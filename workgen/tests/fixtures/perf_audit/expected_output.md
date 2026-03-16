# Goal

Fix performance anti-patterns: unnecessary collect into Vec, allocation inside hot loop, missing with_capacity, string concatenation in loop, clone where borrow suffices, full sort for partial result, repeated HashMap lookup, and lock per parallel iteration.

# Why

The codebase has multiple performance issues: `aggregate` collects into an intermediate Vec just to sum it (iterator would be direct), `build_index` clones strings for HashMap keys unnecessarily, `process_batch` acquires the lock on every iteration instead of batch-inserting, `format_results` concatenates strings in a loop without pre-sized capacity (should use join), `top_scores` fully sorts when only top-N is needed (should use select_nth_unstable or a heap), `lookup_all` calls `map.get(key)` twice per iteration instead of using the entry API or a single let binding, and `or_insert_with(Vec::new)` should use `with_capacity` when the size is estimable.

# What changes

- In aggregate: remove intermediate collect, use `data.iter().copied().sum()` directly
- In build_index: avoid clone by using `&str` keys or accepting owned strings
- In process_batch: collect into a local buffer, lock once, batch insert — do not lock per iteration
- In format_results: use `items.join(", ")` instead of string concatenation in a loop, or pre-allocate with with_capacity
- In top_scores: use `select_nth_unstable` for partial sort instead of full sort
- In lookup_all: use a single `map.get(key)` with let binding instead of repeated HashMap lookup
- Add with_capacity where collection sizes are known

# Files affected

- src/main.rs — intermediate collect, clone for HashMap, lock per iteration, string concat in loop, full sort for top-k, repeated HashMap lookup, missing with_capacity

# Task List

## Task 1: Remove intermediate collect in aggregate

Replace `data.iter().copied().collect::<Vec>()` followed by `.iter().sum()` with `data.iter().copied().sum()`.

```
just fmt
git add src/main.rs
git commit -m "perf: remove unnecessary intermediate collect in aggregate"
```

## Task 2: Fix lock contention in process_batch

Collect results into a local HashMap, then lock once and merge. Do not acquire lock per iteration.

```
just fmt
git add src/main.rs
git commit -m "perf: batch insert under single lock in process_batch"
```

## Task 3: Fix string concat, sort, and repeated lookup

Use `join` instead of loop concatenation in format_results. Use `select_nth_unstable` in top_scores. Use single `let` binding for HashMap get in lookup_all. Add with_capacity where sizes are known.

```
just fmt
git add src/main.rs
git commit -m "perf: fix string concat, partial sort, repeated lookup"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify performance audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No intermediate collect for single-pass aggregates
- No lock acquisition inside loops — batch and lock once
- Use with_capacity when collection size is known

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
