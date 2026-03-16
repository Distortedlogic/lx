# Goal

Fix Diesel usage violations: sequential queries replaceable by joins, raw SQL for expressible queries, loop of individual upserts, filter after load, string column comparisons, and multiple writes without transaction.

# Why

`get_user_orders` issues two sequential queries when a single join would suffice. `fetch_report` uses raw `sql_query` for a query expressible via diesel's query builder (join + group_by + count). `batch_upsert` loops individual upserts instead of batch inserting. `get_active_items` loads all rows then filters in Rust instead of using a diesel `.filter()` clause. `update_status` issues two update queries without a transaction wrapper. String comparison `.status == "active"` should use a DbEnum.

# What changes

- Combine sequential queries in `get_user_orders` into a single join query
- Replace `sql_query` in `fetch_report` with diesel query builder using `.inner_join()`, `.group_by()`, `.select()`
- Replace loop of individual upserts in `batch_upsert` with single batch `insert_into().values(&records)`
- Move Rust-side filter in `get_active_items` into diesel `.filter()` clause — filter after load defeats the database
- Wrap multiple writes in `update_status` in a `conn.transaction()` block
- Replace string comparison `"active"` with a DbEnum variant

# Files affected

- src/db.rs — sequential queries, raw SQL, loop upserts, filter after load, missing transaction, string enum comparison

# Task List

## Task 1: Fix sequential queries and raw SQL

Combine `get_user_orders` into a single join. Replace `fetch_report` sql_query with diesel query builder.

```
just fmt
git add src/db.rs
git commit -m "fix: use joins instead of sequential queries, replace raw SQL"
```

## Task 2: Fix batch upsert and filter after load

Batch the upsert loop into a single insert. Move filter into diesel query in `get_active_items`.

```
just fmt
git add src/db.rs
git commit -m "fix: batch upsert, move filter to diesel query"
```

## Task 3: Add transaction and fix string enum

Wrap `update_status` writes in transaction. Replace string status comparison with DbEnum.

```
just fmt
git add src/db.rs
git commit -m "fix: add transaction wrapper, use DbEnum for status"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify diesel audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No sequential queries when a join works
- No raw SQL for expressible queries
- No filter after load — push filters to the database

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
