# Goal

Refactor `workgen/tests/fixtures/diesel_audit/src/db.rs` to eliminate raw SQL, sequential queries, loop-based upserts, post-load filtering, string-typed enum columns, and missing transaction boundaries. Every function in this file violates at least one diesel best practice — this work item rewrites each function to use diesel's query builder idiomatically, adds type-safe enum handling, and wraps multi-write operations in transactions.

# Why

- `fetch_report` uses `sql_query` with `format!()` string interpolation, bypassing both compile-time schema validation and parameterized query safety — this is a SQL injection vector
- `get_user_orders` issues two sequential queries where one join suffices, doubling round trips and loading an entire `User` struct only to read `.id` (which the caller already has)
- `batch_upsert` issues one insert per row in a loop instead of a single batch upsert, causing N round trips instead of one
- `get_active_items` loads every row from the `items` table into memory then filters in Rust, transferring unnecessary data from the database
- `items::status` is stored as a `String` and compared against string literals, losing compile-time exhaustiveness checks and allowing invalid values
- `update_status` issues two separate `.execute()` calls without a transaction boundary — a failure on the second leaves the database in a partially-updated state

# What changes

**fetch_report** — Replace `diesel::sql_query(format!(...))` with a diesel query builder expression using `.inner_join(orders::table)`, `.filter(users::status.eq(query))`, `.group_by(users::name)`, and `.select((users::name, count(orders::id)))`. Remove the `ReportRow` `QueryableByName` struct if it exists and use a tuple or typed struct deriving `Queryable` instead.

**get_user_orders** — Remove the first query that loads the `User` struct. Filter orders directly with `orders::table.filter(orders::user_id.eq(user_id)).load::<Order>(conn)`. The `user_id` parameter already contains the value needed, making the user lookup redundant.

**batch_upsert** — Remove the `for` loop. Pass the entire `records` vec to `.values(&records)` in a single `diesel::insert_into(records::table).values(&records).on_conflict(records::key).do_update().set(...)` call.

**get_active_items** — Move the status filter into the diesel query by adding `.filter(items::status.eq("active"))` (or the enum variant once the enum is introduced). Remove the Rust-side `.into_iter().filter()` chain.

**Status enum** — Define an `ItemStatus` enum with variants matching the fixed set (at minimum `Active`). Derive `DbEnum` on it. Change the `items::status` model field from `String` to `ItemStatus`. Update `get_active_items` to filter by `ItemStatus::Active` and `update_status` to accept `ItemStatus` instead of `&str`.

**update_status** — Combine the two separate `.set()` calls into a single update using `.set((items::status.eq(new_status), items::updated_at.eq(diesel::dsl::now)))`. Wrap in `conn.transaction()` if they remain as separate statements.

# Files affected

- `workgen/tests/fixtures/diesel_audit/src/db.rs` — all six functions rewritten; `ItemStatus` enum added; string comparisons replaced with enum variants

# Task List

## Task 1: Rewrite get_user_orders to eliminate sequential queries and redundant select-all

Remove the first query that loads the full `User` struct. Replace the two-query function body with a single query: `orders::table.filter(orders::user_id.eq(user_id)).load::<Order>(conn).unwrap()`. The `user_id` parameter already contains the foreign key value, making the user lookup entirely redundant.

File: `workgen/tests/fixtures/diesel_audit/src/db.rs`, function `get_user_orders`.

Finish with: `just fmt`, `git add -A`, `git commit -m "refactor: eliminate sequential queries in get_user_orders"`.

## Task 2: Rewrite batch_upsert to use single batch operation

Remove the `for` loop that calls `insert_into().on_conflict().do_update()` per row. Replace with a single call: `diesel::insert_into(records::table).values(&records).on_conflict(records::key).do_update().set(...)` passing the entire vec at once. Ensure the `NewRecord` type derives `AsChangeset` if it does not already, so it can be used in `.set()`.

File: `workgen/tests/fixtures/diesel_audit/src/db.rs`, function `batch_upsert`.

Finish with: `just fmt`, `git add -A`, `git commit -m "refactor: batch upsert in single query"`.

## Task 3: Rewrite get_active_items to filter in database instead of Rust

Remove the two-step load-then-filter pattern. Replace with `items::table.filter(items::status.eq("active")).load::<Item>(conn).unwrap()`. Remove the `all_items` binding, the `.into_iter().filter()` chain, and the `active` binding. Return the query result directly.

File: `workgen/tests/fixtures/diesel_audit/src/db.rs`, function `get_active_items`.

Finish with: `just fmt`, `git add -A`, `git commit -m "refactor: move status filter into diesel query"`.

## Task 4: Rewrite update_status to combine updates and add transaction

Merge the two separate `diesel::update().set().execute()` calls into a single update that sets both fields at once: `.set((items::status.eq(new_status), items::updated_at.eq(diesel::dsl::now)))`. This eliminates the need for a transaction boundary since it becomes a single atomic operation. Remove the second `diesel::update()` call entirely.

File: `workgen/tests/fixtures/diesel_audit/src/db.rs`, function `update_status`.

Finish with: `just fmt`, `git add -A`, `git commit -m "refactor: combine update_status into single query"`.

## Task 5: Rewrite fetch_report to use query builder instead of raw SQL

Replace the `diesel::sql_query(format!(...))` call with a diesel query builder expression. The rewritten query should: (1) start from `users::table`, (2) call `.inner_join(orders::table)`, (3) call `.filter(users::status.eq(query))` using a parameterized bind instead of string interpolation, (4) call `.group_by(users::name)`, (5) call `.select((users::name, diesel::dsl::count(orders::id)))`. The return type should change from `Vec<ReportRow>` to `Vec<(String, i64)>` or a tuple-based `Queryable` struct, eliminating any `QueryableByName` dependency.

File: `workgen/tests/fixtures/diesel_audit/src/db.rs`, function `fetch_report`.

Finish with: `just fmt`, `git add -A`, `git commit -m "refactor: replace raw SQL with query builder in fetch_report"`.

## Task 6: Add ItemStatus enum and replace string-typed status column

Define an `ItemStatus` enum with at least an `Active` variant (and other variants as indicated by usage in the codebase). Derive `DbEnum` on it. Change the `status` field in the `Item` model struct from `String` to `ItemStatus`. Update `get_active_items` to filter by `ItemStatus::Active` instead of the string literal `"active"`. Update `update_status` to accept `ItemStatus` instead of `&str` as the `new_status` parameter.

File: `workgen/tests/fixtures/diesel_audit/src/db.rs`.

Finish with: `just fmt`, `git add -A`, `git commit -m "refactor: replace string status with DbEnum ItemStatus"`.

## Task 7: Final verification

Run the full verification suite to confirm all changes compile and pass:

1. `just fmt` — confirm formatting is clean
2. `just diagnose` — confirm no compiler errors or clippy warnings
3. `just test` — confirm all tests pass

---

# CRITICAL REMINDERS

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "work_items/DIESEL_AUDIT.md" })
```

Then call `next_task` to get the first task and begin implementation.