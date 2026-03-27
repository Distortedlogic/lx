# Library-Specific Performance Audit

Every item below is a binary check — a violation either exists or it does not. Each check targets a performance anti-pattern specific to a library. The audit checks each item across all `.rs` files in all crates.

## Polars — Lazy Evaluation

- **Premature LazyFrame materialization** — `.collect()` called before all transformations are chained.
  Grep: `.collect()`, `LazyFrame`, `.lazy()`
- **Redundant materialization** — same LazyFrame collected more than once in the same scope.
  Grep: `.collect()`, `LazyFrame`
- **Unnecessary LazyFrame/DataFrame clone** — `.clone()` where a reference or continued chain would suffice.
  Grep: `.clone()`, `LazyFrame`, `DataFrame`
- **Manual group-iterate-recombine** — loop instead of Polars `.over()` window expressions.
  Grep: `group_by`, `.over(`, `for .+ in `
- **Filter after collect** — defeats predicate pushdown.
  Grep: `.collect()`, `.filter(`, `LazyFrame`
- **Select all columns** — defeats projection pushdown.
  Grep: `.collect()`, `.select(`, `col("*")`
- **vstack/extend in loop** — DataFrame `.vstack()` or `.extend()` inside a loop instead of collecting into a Vec and calling `concat` once.
  Grep: `.vstack(`, `.extend(`, `concat(`, `for .+ in `

## Polars — Vectorization

- **Element-wise scalar iteration** — over Series or ChunkedArray where a Polars expression would vectorize the operation.
  Grep: `.iter()`, `.get(`, `ChunkedArray`, `Series`, `for .+ in `
- **Manual aggregation loop** — in Rust instead of Polars built-in aggregation expressions (sum, mean, min, max, std).
  Grep: `.sum()`, `.mean()`, `.min()`, `.max()`, `for .+ in `, `fold(`
- **Element-by-element null handling** — in Rust instead of Polars null-aware expressions (fill_null, drop_nulls, is_null).
  Grep: `is_none()`, `is_some()`, `fill_null`, `drop_nulls`, `Option<`
- **Repeated expression evaluation** — same Polars expression evaluated multiple times instead of computing once and binding to a variable.
  Grep: `.select(`, `.with_column(`, `.filter(`

## Polars — Extraction Boundary

- **Full column extraction for simple query** — to a Rust Vec solely to answer a question answerable on the Series directly (len, sum, min, max).
  Grep: `.to_vec()`, `.into_no_null_iter()`, `as_ref()`, `Series`
- **Polars-to-Rust-to-Polars round-trip** — extracting data, transforming in Rust, then pushing back into a DataFrame.
  Grep: `.to_vec()`, `Series::new`, `DataFrame`, `with_column`
- **Copying borrowed Polars slice** — into an owned collection when downstream only needs a reference.
  Grep: `.to_vec()`, `.to_owned()`, `.clone()`, `ChunkedArray`
- **String columns for repeated comparisons** — where Categorical type would reduce memory and speed lookups.
  Grep: `Utf8`, `String`, `Categorical`, `cast(`, `DataType`

## Polars — I/O

- **Eager read instead of lazy scan** — CsvReader/read_parquet into DataFrame where LazyCsvReader/scan_parquet would allow predicate/projection pushdown.
  Grep: `CsvReader`, `read_parquet`, `LazyCsvReader`, `scan_parquet`, `read_ipc`, `scan_ipc`
- **Missing performant feature flag** — in Cargo.toml when performance-critical Polars operations are used.
  Grep: `polars`, `performant`, `features`

## Rayon — Parallelism Coverage

- **Sequential iter over large collection** — `.iter()` with non-trivial per-item work where `.par_iter()` would parallelize.
  Grep: `.iter()`, `.par_iter()`, `for .+ in `, `rayon`
- **par_iter on trivial/small work** — parallelism overhead exceeds benefit.
  Grep: `.par_iter()`
- **Nested parallelism** — `.par_iter()` inside `.par_iter()` causing thread pool oversubscription.
  Grep: `.par_iter(`, `par_bridge`, `par_chunks`
- **Unbalanced work distribution** — without dynamic scheduling or appropriate chunk sizing.
  Grep: `.par_iter()`, `.par_chunks(`, `.with_min_len(`, `.with_max_len(`

## Rayon — Allocation & Synchronization

- **Allocations inside parallel closures** — could be per-thread reusable buffers via `map_init` or `map_with`.
  Grep: `.par_iter(`, `Vec::new()`, `String::new()`, `map_init`, `map_with`
- **Per-thread allocations sized to full dataset** — instead of the thread's chunk.
  Grep: `.par_iter(`, `Vec::with_capacity`, `vec![`, `.len()`
- **Lock per parallel iteration** — instead of batch accumulate-then-insert.
  Grep: `.lock()`, `.write()`, `.par_iter(`, `Mutex`, `RwLock`
- **Sequential merge after parallel map** — where Rayon's parallel `.reduce()` or `.fold()` would work.
  Grep: `.par_iter(`, `.collect::<Vec`, `.reduce(`, `.fold(`
