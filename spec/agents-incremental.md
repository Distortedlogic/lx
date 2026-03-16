# Incremental / Memoized Plan Execution

Extension to `std/plan`. Input-based cache invalidation so re-running a workflow skips steps whose inputs haven't changed. Build-system semantics for agentic workflows.

## Problem

A 20-step workflow re-runs from scratch every time. Step 14 changed, but steps 1-13 re-execute (burning tokens, time, API calls) and produce identical results. `std/plan` has dependency ordering but no caching.

```
plan.run steps {on_step: execute} ^
// all 20 steps run, even if only step 14's input changed
```

## Solution: `plan.run_incremental`

```
use std/plan

result = plan.run_incremental steps {
  on_step: execute
  cache: "plan_cache.json"
} ^
```

`plan.run_incremental` is a new function in `std/plan`. It:
1. Hashes each step's inputs (the step record minus the handler)
2. Checks the cache for a matching hash
3. On cache hit: returns the cached result, skips execution
4. On cache miss: executes the step, caches the result

### Cache Key

The cache key for a step is a content hash of:
- Step `name` (identity)
- Step `deps` (dependency names, not their results — results are hashed separately)
- Step input data (any fields on the step record besides `name`, `deps`, `handler`)
- Cached results of dependency steps (so a dep change invalidates downstream)

```
key = hash(step.name, step.deps, step.inputs, [dep_results...])
```

### Cache File

```json
{
  "steps": {
    "step_name": {
      "key": "abc123...",
      "result": {"findings": ["..."]},
      "timestamp": 1710000000,
      "elapsed_ms": 450
    }
  }
}
```

JSON file, same pattern as `std/knowledge` and `std/reputation`.

### Invalidation

A step re-executes when:
- Its inputs changed (hash mismatch)
- Any dependency's result changed (transitive invalidation)
- The cache file doesn't exist or is corrupted
- The step is explicitly marked `no_cache: true`
- The caller passes `force: true`

```
result = plan.run_incremental steps {
  on_step: execute
  cache: "plan_cache.json"
  force: true
} ^
// force re-runs all steps, rebuilds cache
```

### Selective Force

```
result = plan.run_incremental steps {
  on_step: execute
  cache: "plan_cache.json"
  force_steps: ["step_14" "step_15"]
} ^
```

Force-run specific steps while using cache for others.

### Cache-Aware Callbacks

```
result = plan.run_incremental steps {
  on_step: execute
  cache: "plan_cache.json"
  on_cache_hit: (step cached) {
    emit {type: "cached" step: step.name elapsed: cached.elapsed_ms}
  }
  on_cache_miss: (step) {
    emit {type: "executing" step: step.name}
  }
} ^
```

### TTL

```
result = plan.run_incremental steps {
  on_step: execute
  cache: "plan_cache.json"
  ttl: 3600
} ^
```

`ttl` (seconds) — cached results older than this are treated as misses. Default: no expiration.

### Non-Deterministic Steps

Steps that depend on external state (API calls, file system, time) may return different results even with the same inputs. Mark them:

```
steps = [
  {name: "fetch_data" handler: fetch  no_cache: true}
  {name: "process" deps: ["fetch_data"] handler: process}
  {name: "report" deps: ["process"] handler: report}
]
```

`fetch_data` always re-runs. `process` and `report` re-run only if `fetch_data`'s result changed.

### With Budget

```
result = plan.run_incremental steps {
  on_step: (step) {
    r = execute step ^
    budget.spend b {tokens: r.usage} ^
    r
  }
  cache: "plan_cache.json"
  on_cache_hit: (step cached) {
    emit "saved {cached.elapsed_ms}ms on {step.name}"
  }
} ^
```

Cached steps cost nothing — budget is preserved for steps that actually need to run.

## Implementation

Extension to existing `std/plan` module. Adds `plan.run_incremental` alongside `plan.run`. Cache logic is a wrapper around the existing plan execution engine — hash inputs, check cache, execute or return cached, save results.

Hashing uses `serde_json::to_string` on the step record (stable serialization) + a simple hash function. Exact hash algorithm TBD (likely FNV or xxhash via crate).

### Dependencies

- `std/plan` (existing plan execution)
- `serde_json` (cache serialization)
- `std/fs` (cache file I/O)
- Hash crate (TBD)

## Cross-References

- Plan execution: [agents-plans.md](agents-plans.md) (base plan.run)
- Budget: [agents-budget.md](agents-budget.md) (cached steps save budget)
- Knowledge: [stdlib-knowledge.md](stdlib-knowledge.md) (same file-backed pattern)
