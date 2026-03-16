# Goal

Eliminate twelve performance anti-patterns in `workgen/tests/fixtures/perf_audit/src/main.rs` spanning unnecessary heap allocations, suboptimal data structure usage, lock contention, redundant lookups, and algorithmic inefficiency. Each fix applies the idiomatic Rust solution for the specific anti-pattern.

# Why

- `aggregate` collects into an intermediate Vec solely to sum it, doubling memory usage for a single-pass operation
- `build_index` uses a default-capacity HashMap that rehashes multiple times despite the size being known, and clones every string key on insert
- `process_batch` acquires the global Mutex on every loop iteration, causing lock contention proportional to batch size instead of constant
- The global `CACHE` uses Mutex where RwLock would allow concurrent readers
- `format_results` allocates a temporary String via `format!` on every iteration instead of using `join`
- `top_scores` fully sorts the input to extract the top-k, performing O(n log n) work where O(n) suffices, and uses stable sort on primitives where unstable sort is faster
- `lookup_all` hashes and looks up the same key twice per iteration and starts with a zero-capacity Vec despite knowing the upper bound

# What changes

- **CACHE static**: Change `Mutex<HashMap<String, Vec<f64>>>` to `RwLock<HashMap<String, Vec<f64>>>`, update the import from `std::sync::Mutex` to `std::sync::RwLock`
- **aggregate**: Remove the `collected` Vec entirely, replace with `data.iter().copied().sum()`
- **build_index**: Change `HashMap::new()` to `HashMap::with_capacity(items.len())`, change the parameter from `&[String]` to `Vec<String>` and use `into_iter()` to move owned strings into the map instead of cloning
- **process_batch**: Collect all entries into a local `Vec<(String, f64)>` buffer first, then acquire the write lock once and batch-insert all entries. Use `.write().unwrap()` since CACHE is now RwLock
- **format_results**: Replace the manual loop with `items.join(", ")`, removing both the `String::new()` and the per-iteration `format!` allocation
- **top_scores**: Replace `sort_by` with `select_nth_unstable_by` to partition the top-n in O(n) average time, then sort only the selected slice with `sort_unstable_by`
- **lookup_all**: Change `Vec::new()` to `Vec::with_capacity(keys.len())`, replace the double `map.get(key)` with a single `if let Some(val) = map.get(key)` binding

# Files affected

- `workgen/tests/fixtures/perf_audit/src/main.rs` — all changes are in this single file

# Task List

## Task 1: Replace Mutex with RwLock on CACHE static

**File:** `workgen/tests/fixtures/perf_audit/src/main.rs`

Change the import from `std::sync::Mutex` to `std::sync::RwLock`. Change the CACHE static type from `Mutex<HashMap<String, Vec<f64>>>` to `RwLock<HashMap<String, Vec<f64>>>`. Change the initializer from `Mutex::new(HashMap::new())` to `RwLock::new(HashMap::new())`.

**Verification:** File compiles with `just diagnose` after updating all lock call sites in subsequent tasks.

**Run:** `just fmt && git add -A && git commit -m "perf: replace Mutex with RwLock on CACHE static"`

## Task 2: Eliminate intermediate Vec in aggregate

**File:** `workgen/tests/fixtures/perf_audit/src/main.rs`

Replace the body of `aggregate` — remove the `collected` variable and its `.iter().sum()` call. Replace with a single expression: `data.iter().copied().sum()`.

**Verification:** The function body is a single line returning the sum directly from the slice iterator.

**Run:** `just fmt && git add -A && git commit -m "perf: eliminate intermediate Vec in aggregate"`

## Task 3: Optimize build_index with capacity and owned keys

**File:** `workgen/tests/fixtures/perf_audit/src/main.rs`

Change `build_index` parameter from `items: &[String]` to `items: Vec<String>`. Change `HashMap::new()` to `HashMap::with_capacity(items.len())`. Change the loop to use `items.into_iter().enumerate()` and insert the owned `item` directly instead of calling `item.clone()`.

**Verification:** No `.clone()` call remains in `build_index`. The HashMap is initialized with capacity.

**Run:** `just fmt && git add -A && git commit -m "perf: use with_capacity and owned keys in build_index"`

## Task 4: Batch lock acquisition in process_batch

**File:** `workgen/tests/fixtures/perf_audit/src/main.rs`

Restructure `process_batch` to first collect all key-value pairs into a local `Vec<(&str, f64)>` buffer by iterating over `keys.iter().zip(values)`. After the collection loop, acquire the CACHE write lock once with `CACHE.write().unwrap()` and iterate over the local buffer to insert all entries. Each insert uses `.entry(key.to_string()).or_insert_with(Vec::new).push(val)` on the locked cache.

**Verification:** The lock is acquired exactly once, outside of any loop. No `.lock()` call remains — it uses `.write()` since CACHE is now RwLock.

**Run:** `just fmt && git add -A && git commit -m "perf: batch lock acquisition in process_batch"`

## Task 5: Replace loop concatenation with join in format_results

**File:** `workgen/tests/fixtures/perf_audit/src/main.rs`

Replace the entire body of `format_results` with `items.join(", ")`. Remove the `String::new()`, the loop, the `push_str`, and the `format!` call.

**Verification:** The function body is a single expression. No `String::new()`, no `push_str`, no `format!`.

**Run:** `just fmt && git add -A && git commit -m "perf: replace loop concatenation with join in format_results"`

## Task 6: Use partial sort in top_scores

**File:** `workgen/tests/fixtures/perf_audit/src/main.rs`

Replace the `sort_by` call in `top_scores` with two steps: first call `scores.select_nth_unstable_by(n - 1, |a, b| b.partial_cmp(a).unwrap())` to partition the top-n elements to the front in O(n) average time. Then call `scores[..n].sort_unstable_by(|a, b| b.partial_cmp(a).unwrap())` to sort only the selected slice. Keep the final `scores[..n].to_vec()` return.

**Verification:** No `.sort_by(` call remains. Both `select_nth_unstable_by` and `sort_unstable_by` are present.

**Run:** `just fmt && git add -A && git commit -m "perf: use select_nth_unstable for top-k in top_scores"`

## Task 7: Deduplicate lookups and add capacity in lookup_all

**File:** `workgen/tests/fixtures/perf_audit/src/main.rs`

Change `Vec::new()` to `Vec::with_capacity(keys.len())`. Replace the double-lookup pattern — remove the `if map.get(key).is_some()` check and the second `map.get(key).unwrap()`. Replace with `if let Some(val) = map.get(key) { results.push(*val); }`.

**Verification:** Only one `map.get(key)` call per iteration. Vec is initialized with capacity. No `.is_some()` or `.unwrap()` on a get call.

**Run:** `just fmt && git add -A && git commit -m "perf: deduplicate lookups and add capacity in lookup_all"`

## Task 8: Final verification

Run the full verification suite to confirm all changes compile and pass.

**Run:** `just test && just diagnose && just fmt`

---

# CRITICAL REMINDERS

Re-read before starting each task:

1. **Run the fmt/commit commands at the end of each task.** They are listed explicitly in each task.
2. **Execute tasks in order.** Do not skip, reorder, or combine tasks.
3. **Do not add tasks.** Execute the task list exactly as written.
4. **The final task is verification.** It must pass cleanly before the work item is considered complete.

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "work_items/PERF_AUDIT_FIXTURE_FIXES.md" })
```

Then call `mcp__workflow__next_task` to get the first task and begin implementation.