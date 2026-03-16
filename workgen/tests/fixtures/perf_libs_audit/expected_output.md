# Goal

Fix library-specific performance anti-patterns: premature LazyFrame materialization, filter after collect, select all columns, element-wise scalar iteration, Polars-to-Rust-to-Polars round-trip, eager read instead of lazy scan, lock per parallel iteration, and nested parallelism.

# Why

`process_data` collects the LazyFrame three times in sequence defeating predicate and projection pushdown — chain transformations before collecting once. It also selects all columns with `col("*")`. `aggregate_manual` iterates Series element-by-element instead of using Polars vectorized `.sum()`. `extract_and_transform` extracts a column to a Rust Vec, transforms, then pushes back into a DataFrame — a classic Polars-to-Rust-to-Polars round-trip. `read_eager` uses `CsvReader` instead of `LazyCsvReader` for scan with pushdown. `parallel_process` acquires a lock per parallel iteration instead of using Rayon's `.reduce()` or `.fold()`. It also allocates a new Vec inside the parallel closure. `nested_parallel` nests `.par_iter()` inside `.par_iter()` causing thread pool oversubscription.

# What changes

- Chain all LazyFrame transformations in `process_data` and collect once at the end
- Replace `col("*")` with explicit column selection for projection pushdown
- Replace element-wise iteration in `aggregate_manual` with `series.sum()` Polars expression
- Express transform in `extract_and_transform` as a Polars expression to avoid round-trip
- Replace `CsvReader` with `LazyCsvReader` in `read_eager` for predicate/projection pushdown
- Replace lock-per-iteration in `parallel_process` with Rayon `.reduce()` or `.fold()`
- Replace nested `.par_iter()` in `nested_parallel` with flat single-level parallelism

# Files affected

- src/pipeline.rs — premature collect, select all, scalar iteration, round-trip, eager read, lock per iteration, nested parallelism

# Task List

## Task 1: Fix Polars lazy evaluation

Chain transformations in `process_data` and collect once. Replace `col("*")` with explicit select. Use `series.sum()` in `aggregate_manual`.

```
just fmt
git add src/pipeline.rs
git commit -m "perf: fix premature collect, projection pushdown, vectorized aggregation"
```

## Task 2: Fix round-trip and eager read

Express transform as Polars expression. Replace CsvReader with LazyCsvReader.

```
just fmt
git add src/pipeline.rs
git commit -m "perf: eliminate Polars round-trip, use lazy CSV scan"
```

## Task 3: Fix Rayon patterns

Replace lock per iteration with reduce/fold. Remove nested par_iter.

```
just fmt
git add src/pipeline.rs
git commit -m "perf: fix lock contention and nested parallelism in Rayon"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify performance-libs audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- Chain LazyFrame operations, collect once
- Never iterate Series element-by-element — use expressions
- No lock per parallel iteration

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
