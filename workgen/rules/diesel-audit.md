# Diesel Codebase Audit

Every item below is a binary check — a violation either exists or it does not. The audit checks each item across all `.rs` files in crates that use diesel, diesel-async, or diesel-derive-enum.

---

## Raw SQL

- **Raw SQL for expressible queries** — `diesel::sql_query()` used for a query that can be expressed via diesel's query builder (joins, group_by, aggregates, subqueries). Raw SQL bypasses compile-time schema validation, breaks under column renames, and loses type safety. Fix: rewrite using diesel's query builder with explicit `.inner_join()` / `.left_join()`, `.group_by()`, and aggregate functions (`count()`, `min()`, `max()`, `sum()`, `avg()`). Only keep `sql_query()` when the query uses database-specific syntax that diesel's DSL genuinely cannot represent (CTEs, window functions, lateral joins, custom operators).
  `rg 'sql_query' --type rust crates/`
  `rg 'QueryableByName' --type rust crates/`
  For each hit: read the SQL string and determine whether diesel's query builder can express it. Flag if it can.

- **QueryableByName struct duplicating schema** — a struct with `#[derive(QueryableByName)]` whose fields mirror an existing `Queryable` model or a subset of one. Fix: if the query can be rewritten via the query builder (see above), use the existing `Queryable` model or a tuple. If raw SQL is genuinely necessary, check whether the `QueryableByName` struct duplicates 3+ fields from an existing model — if so, hold the model as a field or merge.
  `rg 'QueryableByName' --type rust crates/`
  Cross-reference field lists with existing model structs in the same crate.

---

## Sequential DB Calls

- **Sequential queries replaceable by joins** — two or more queries in the same function where the second query uses results from the first as a filter, and the two tables have a foreign key relationship or joinable column. Fix: combine into a single query with `.inner_join()` or `.left_join()` and `.select()` the needed columns. Diesel's `joinable!` macro in `schema.rs` declares valid join paths — use them.
  `rg 'joinable!' --type rust crates/`
  For each DB function: read the body. If it issues 2+ queries and the second filters by a value from the first's result, flag it.

- **Sequential deletes on related tables** — multiple `diesel::delete()` calls in the same function targeting different tables filtered by the same key. Fix: if the tables have foreign key constraints, add `ON DELETE CASCADE` to the migration and delete only from the parent table. If CASCADE is inappropriate, batch the deletes into a single transaction (they likely already share a connection, but verify the transaction boundary).
  `rg 'diesel::delete' --type rust crates/`
  For each function with 2+ delete calls: check if they filter by the same key.

- **Loop of individual upserts** — a `for` loop calling `insert_into().on_conflict().do_update()` one row at a time. Diesel supports batch `insert_into().values(&vec_of_rows)` with `.on_conflict().do_update()`. Fix: collect rows into a Vec and issue a single batch upsert.
  `rg 'for .* in ' --type rust crates/trading-db/`
  `rg 'on_conflict' --type rust crates/`
  For each loop containing an insert/upsert: check if it can be batched.

---

## Enum Usage

- **String column for fixed variant set** — a model struct has a `String` field whose values come from a Rust enum (converted via `.to_string()`, `Display`, or manual string literals). This loses type safety, allows invalid values in the database, and requires string matching instead of exhaustive pattern matching. Fix: (1) derive `DbEnum` on the source enum using `diesel-derive-enum`, (2) add the corresponding SQL type to `sql_types.rs`, (3) create a PostgreSQL migration to add the enum type and alter the column, (4) change the model field from `String` to the enum type.
  `rg '\.to_string\(\)' --type rust crates/trading-db/`
  `rg 'String' --type rust crates/trading-db/src/models.rs`
  `rg 'DbEnum' --type rust crates/`
  For each `String` field in a model: trace where values originate. If they come from a Rust enum or a fixed set of literals, flag it.

- **String comparison instead of enum variant** — a diesel `.filter()` or Rust `match`/`if` compares a model's `String` field against a string literal (e.g., `.filter(status.eq("Elite"))`). Fix: after converting the column to a `DbEnum` (see above), compare against the enum variant directly (e.g., `.filter(status.eq(ArchiveStatus::Elite))`).
  `rg '\.eq\("' --type rust crates/trading-db/`
  `rg '\.filter\(.*"' --type rust crates/trading-db/`
  For each string comparison in a diesel filter: check if the column should be an enum.

- **Missing DbEnum derive** — a Rust enum represents a database column's domain but does not derive `DbEnum`. It is instead converted to/from strings at the boundary via `From`/`Into`, `Display`/`FromStr`, or manual mapping. Fix: derive `DbEnum`, define the SQL type, and use the enum directly in the model.
  `rg 'enum ' --type rust crates/trading-types/`
  `rg 'DbEnum' --type rust crates/`
  Cross-reference: for each enum in trading-types that has a corresponding string column in a diesel model, flag if `DbEnum` is not derived.

- **Enum-to-string conversion at DB boundary** — a `From` impl or conversion function that converts an enum to a `String` (or `&str`) for database insertion, or parses a `String` back into an enum after querying. Fix: derive `DbEnum` on the enum so diesel handles the conversion natively. Remove the manual `From`/`Into`/`Display`/`FromStr` impls that exist solely for the DB boundary.
  `rg 'impl From<' --type rust crates/trading-db/src/conversions.rs`
  `rg '\.to_string\(\)' --type rust crates/trading-db/src/conversions.rs`
  For each conversion: check if a field is converted from an enum to a string. Flag it.

---

## Query Builder

- **Select-all instead of explicit select** — a query omits `.select()` and loads all columns when only a subset is needed. This transfers unnecessary data from the database. Fix: add `.select((col_a, col_b, ...))` with only the needed columns, and use a tuple or a dedicated struct deriving `Queryable` for the result.
  `rg '\.load::<' --type rust crates/trading-db/`
  `rg '\.first::<' --type rust crates/trading-db/`
  For each query: check if `.select()` is present. If the consumer uses fewer than half the model's fields, flag it.

- **N+1 query pattern** — a query loads a list of parent rows, then loops over them issuing a query per row for related data. Fix: load the related data in a single query with `.filter(parent_id.eq_any(&parent_ids))` or use a join.
  `rg '\.load::<' --type rust crates/trading-db/`
  For each function that loads a collection: check if a subsequent loop issues per-row queries.

- **Filter after load** — all rows loaded via `.load()` followed by Rust-side `.filter()` / `.iter().filter()` instead of a diesel `.filter()` clause. Fix: move the filter into the diesel query.
  `rg '\.load::<' --type rust crates/trading-db/`
  `rg '\.filter(|' --type rust crates/trading-db/`
  For each load: check if the result is immediately filtered in Rust.

---

## Connection & Transaction

- **Multiple queries without transaction** — a function issues 2+ write queries (insert, update, delete) on the same connection without wrapping them in `conn.transaction()`. If any query fails, the database is left in a partially-updated state. Fix: wrap in `conn.transaction(|conn| { ... })`.
  `rg '\.execute\(' --type rust crates/trading-db/`
  For each function with 2+ `.execute()` calls: check if they are inside a `transaction()` block.

- **Connection acquired per query in a batch** — a function calls `pool.get()` inside a loop instead of acquiring one connection before the loop. Fix: acquire the connection once outside the loop.
  `rg 'pool\.get\(\)' --type rust crates/trading-db/`
  `rg 'get_conn' --type rust crates/trading-db/`
  For each pool access inside a loop: flag it.
