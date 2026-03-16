# Goal

Eliminate all Polars and Rayon performance anti-patterns in `workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs`. The file contains 14 violations across six functions: premature and redundant LazyFrame materialization, defeated predicate and projection pushdown, scalar iteration instead of vectorized expressions, Polars-to-Rust-to-Polars round-trips, eager CSV reads instead of lazy scans, Mutex-serialized parallel accumulation, per-iteration allocations, and nested parallelism on trivial work. Every function in the file requires rewriting to use idiomatic, high-performance Polars and Rayon patterns.

# Why

- **process_data** calls `.collect()` three times with an unnecessary `.clone()`, defeating both predicate pushdown (filter applied after materialization) and projection pushdown (`col("*")` selects everything). A single lazy chain with one terminal collect enables the Polars optimizer to push predicates into the scan and prune unused columns.
- **aggregate_manual** iterates element-by-element over a Series using `.get(i)` in a loop, bypassing Polars' vectorized, null-aware `.sum()` that operates on contiguous memory via Arrow arrays.
- **extract_and_transform** extracts a column to a Rust Vec, transforms in Rust, then pushes back into a new Series and DataFrame — a full round-trip that negates Polars' zero-copy columnar engine. Two redundant DataFrame clones compound the waste. A single `col("price") * lit(2.0)` expression does this entirely in-engine.
- **read_eager** uses `CsvReader` which reads the full CSV into memory upfront, preventing any scan-level optimizations. `LazyCsvReader` enables predicate and projection pushdown at the I/O layer.
- **parallel_process** acquires a `Mutex` lock on every `par_iter` iteration and allocates a throwaway `Vec::new()` per iteration, serializing the accumulation and wasting memory. Rayon's `.map().collect()` eliminates both the lock and the per-iteration allocation.
- **nested_parallel** nests `par_iter` inside `par_iter` for a trivial scalar multiply, oversubscribing the Rayon thread pool where the parallelism overhead vastly exceeds the computation cost.

# What changes

## process_data

Remove the `.clone()` on the LazyFrame. Remove all intermediate `.collect()` calls. Chain `.filter(col("value").gt(lit(0)))` directly on the original LazyFrame. Remove the `.select([col("*")])` call entirely since selecting all columns is a no-op that defeats projection pushdown. Call `.collect().unwrap()` once at the end of the chain. The function body becomes a single chained expression: `df.lazy().filter(...).collect().unwrap()`.

## aggregate_manual

Delete the manual `for` loop, the mutable accumulator, the `.get(i)` indexing, and the `try_extract` calls. Replace the entire function body with a call to the Series' built-in vectorized sum. Cast the series to `f64` type via `.f64().unwrap()` and call `.sum().unwrap_or(0.0)` which handles nulls automatically.

## extract_and_transform

Delete the `into_no_null_iter().collect()` extraction to Vec, the Rust-side `.map(|v| v * 2.0)` transform, the `Series::new` reconstruction, and both `.clone()` calls on the DataFrame. Replace with a lazy chain: `df.clone().lazy().with_column((col("price") * lit(2.0)).alias("doubled")).collect().unwrap()`. This keeps the entire operation in the Polars engine. The single remaining `df.clone()` is needed because the function receives `&DataFrame` but `.lazy()` requires ownership.

## read_eager

Replace `CsvReader::from_path(path).unwrap().finish().unwrap()` with `LazyCsvReader::new(path).finish().unwrap().collect().unwrap()`. Change the return type to `LazyFrame` if callers support it, otherwise keep `DataFrame` return with a terminal `.collect()`. This enables predicate and projection pushdown when downstream operations are also lazy.

## parallel_process

Delete the `Mutex`, the `.for_each` closure, the per-iteration `Vec::new()` allocation, and the `results.into_inner().unwrap()` drain. Replace with `items.par_iter().map(|chunk| chunk.iter().sum::<f64>()).collect()` which uses Rayon's parallel collect to build the result Vec without any shared mutable state or per-iteration allocations.

## nested_parallel

Replace the inner `row.par_iter()` with `row.iter()` to eliminate nested parallelism. The inner operation is a single scalar multiply per element — trivial work where parallelism overhead dominates. The outer `par_iter` over matrix rows provides sufficient parallelism.

# Files affected

- **workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs** — Rewrite all six functions as described above. Remove `use std::sync::Mutex` since Mutex is no longer needed after the `parallel_process` fix.

# Task List

## Task 1: Rewrite process_data to use single lazy chain

**Subject:** Rewrite process_data to eliminate premature materialization

**Files:** `workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs`

**Changes:**
- In `process_data`, remove `lazy.clone().collect().unwrap()` and the two subsequent `.lazy()...collect()` round-trips
- Remove the `.select([col("*")])` call entirely — it defeats projection pushdown and is a no-op
- Chain `.filter(col("value").gt(lit(0)))` directly on `df.lazy()`
- Call `.collect().unwrap()` once at the end
- The function body should be a single expression: `df.lazy().filter(col("value").gt(lit(0))).collect().unwrap()`

**Verification:** The function has exactly one `.collect()` call, zero `.clone()` calls, no `col("*")`, and no intermediate DataFrame bindings.

**Post-task:** `just fmt` then `git add -A` then `git commit -m "fix: eliminate premature LazyFrame materialization in process_data"`

## Task 2: Replace manual aggregation with vectorized sum

**Subject:** Rewrite aggregate_manual to use Series built-in sum

**Files:** `workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs`

**Changes:**
- Delete the entire `for` loop body: the mutable `total` accumulator, the `for i in 0..series.len()` loop, the `.get(i).unwrap()`, and the `try_extract::<f64>().unwrap()` calls
- Replace with: cast the series to f64 via `series.f64().unwrap()`, then call `.sum().unwrap_or(0.0)`
- The function body becomes a single expression

**Verification:** The function contains no `for` loop, no `.get(`, no `try_extract`, and no mutable variable.

**Post-task:** `just fmt` then `git add -A` then `git commit -m "fix: replace element-wise iteration with vectorized sum in aggregate_manual"`

## Task 3: Eliminate Polars-to-Rust round-trip in extract_and_transform

**Subject:** Rewrite extract_and_transform to use in-engine Polars expression

**Files:** `workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs`

**Changes:**
- Delete the `into_no_null_iter().collect()` extraction to `Vec<f64>`
- Delete the Rust-side `vals.iter().map(|v| v * 2.0).collect()` transform
- Delete the `Series::new("doubled".into(), doubled)` reconstruction
- Delete the trailing `.clone()` on the result (the second clone)
- Replace the entire body with a lazy chain: `df.clone().lazy().with_column((col("price") * lit(2.0)).alias("doubled")).collect().unwrap()`
- The single `df.clone()` is required because the function takes `&DataFrame` and `.lazy()` requires ownership

**Verification:** The function contains no `into_no_null_iter`, no `Vec<f64>`, no `Series::new`, and at most one `.clone()`. It uses `col("price") * lit(2.0)` as an in-engine expression.

**Post-task:** `just fmt` then `git add -A` then `git commit -m "fix: replace Polars-to-Rust round-trip with in-engine expression in extract_and_transform"`

## Task 4: Replace eager CSV read with lazy scan

**Subject:** Replace CsvReader with LazyCsvReader in read_eager

**Files:** `workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs`

**Changes:**
- Replace `CsvReader::from_path(path).unwrap().finish().unwrap()` with `LazyCsvReader::new(path).finish().unwrap().collect().unwrap()`
- This enables predicate and projection pushdown at the I/O layer

**Verification:** The function contains no `CsvReader`. It uses `LazyCsvReader`.

**Post-task:** `just fmt` then `git add -A` then `git commit -m "fix: replace eager CsvReader with LazyCsvReader in read_eager"`

## Task 5: Eliminate Mutex and per-iteration allocation in parallel_process

**Subject:** Rewrite parallel_process to use Rayon map-collect pattern

**Files:** `workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs`

**Changes:**
- Delete the `Mutex::new(Vec::new())` initialization
- Delete the `.for_each` closure containing the per-iteration `Vec::new()`, `.push()`, `.lock().unwrap().extend()` calls
- Delete `results.into_inner().unwrap()`
- Replace the entire body with: `items.par_iter().map(|chunk| chunk.iter().sum::<f64>()).collect()`
- Remove `use std::sync::Mutex` from the top-level imports since Mutex is no longer used anywhere in the file

**Verification:** The file contains no `Mutex`, no `.lock()`, no `.for_each`, and no `Vec::new()` inside a parallel closure. The `use std::sync::Mutex` import is gone.

**Post-task:** `just fmt` then `git add -A` then `git commit -m "fix: replace Mutex accumulation with parallel map-collect in parallel_process"`

## Task 6: Remove nested parallelism in nested_parallel

**Subject:** Replace inner par_iter with sequential iter in nested_parallel

**Files:** `workgen/tests/fixtures/perf_libs_audit/src/pipeline.rs`

**Changes:**
- In the `nested_parallel` function, change `row.par_iter()` to `row.iter()` inside the outer `matrix.par_iter().flat_map()` closure
- Keep the outer `matrix.par_iter()` unchanged — row-level parallelism is appropriate
- The inner scalar multiply is trivial work where parallelism overhead dominates

**Verification:** The function contains exactly one `par_iter` call (the outer one on `matrix`). The inner loop uses `.iter()`, not `.par_iter()`.

**Post-task:** `just fmt` then `git add -A` then `git commit -m "fix: remove nested parallelism for trivial scalar multiply in nested_parallel"`

## Task 7: Final verification

**Subject:** Run full test suite and diagnostics to verify all changes

**Steps:**
- Run `just test` — all tests must pass
- Run `just diagnose` — zero warnings, zero errors
- Run `just fmt` — no formatting changes needed

**Verification:** All three commands exit with status 0 and produce no warnings or errors.

---

# CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "work_items/PERF_LIBS_AUDIT_PIPELINE.md" })
```

Then call `mcp__workflow__next_task()` to receive the first task.