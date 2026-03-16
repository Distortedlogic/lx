# Pipeline Checkpoint/Resume

Stage-boundary persistence for multi-stage pipelines, enabling resume-from-failure without re-executing completed stages.

## Problem

`std/durable` (specced) handles expression-level workflow persistence — checkpointing and resuming individual expressions. `std/plan` runs step lists with `continue`/`abort` control.

Neither addresses pipeline-level save/resume. In `software_diffusion.lx`, if stage 4 (implement) fails after stages 0-3 succeeded (possibly taking 10+ minutes and real budget), you restart from scratch. There's no way to say "stages 0-3 produced these artifacts, resume from stage 4."

This is distinct from `std/durable` (which checkpoints within an expression) and `std/plan` (which tracks step completion but doesn't persist step outputs across process restarts).

## Design

### Module: `std/pipeline`

```lx
use std/pipeline

pipe = pipeline.create "software-diffusion" {
  resume: true
  storage: ".lx/pipelines/"
} ^

specs = pipeline.stage pipe "elicit" () {
  stage_elicit prompt ^
} ^

skeleton = pipeline.stage pipe "scaffold" specs {
  stage_scaffold specs ^
} ^

stubs = pipeline.stage pipe "stub" skeleton.module_specs {
  stage_stub skeleton.module_specs ^
} ^

verified = pipeline.stage pipe "refine" stubs {
  stage_refine stubs skeleton ^
} ^

result = pipeline.stage pipe "implement" verified {
  stage_implement verified skeleton.module_specs ^
} ^

pipeline.complete pipe ^
```

### How It Works

`pipeline.stage` checks if this stage has a cached result from a prior run:
- **Cache hit**: return cached result, skip body execution
- **Cache miss**: execute body, persist result, return it

Cache key = pipeline name + stage name + hash of input arguments. If inputs change (e.g., specs were modified), the cache invalidates and the stage re-executes.

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `pipeline.create` | `(name: Str opts: Record) -> Result Pipeline Str` | Create or resume pipeline |
| `pipeline.stage` | `(pipe: Pipeline name: Str input: Any body: Fn) -> Result Any Str` | Execute or resume stage |
| `pipeline.complete` | `(pipe: Pipeline) -> Result () Str` | Mark pipeline complete, optionally clean cache |
| `pipeline.status` | `(pipe: Pipeline) -> {stages: [{name: Str status: Str duration_ms: Int}]}` | Pipeline progress |
| `pipeline.invalidate` | `(pipe: Pipeline stage: Str) -> ()` | Force re-execution of stage + all downstream |
| `pipeline.invalidate_from` | `(pipe: Pipeline stage: Str) -> ()` | Invalidate this stage and all after it |
| `pipeline.clean` | `(pipe: Pipeline) -> ()` | Remove all cached data |
| `pipeline.list` | `() -> [{name: Str stages: Int status: Str}]` | List all pipelines with cached state |

### Storage Format

Directory structure:
```
.lx/pipelines/software-diffusion/
  meta.json          -- pipeline metadata, stage order, status
  elicit.json        -- cached output of elicit stage
  elicit.hash        -- input hash for cache invalidation
  scaffold.json      -- cached output of scaffold stage
  scaffold.hash
  ...
```

`meta.json`:
```json
{
  "name": "software-diffusion",
  "created": "2026-03-16T10:00:00Z",
  "updated": "2026-03-16T10:15:00Z",
  "stages": [
    {"name": "elicit", "status": "complete", "started": "...", "finished": "..."},
    {"name": "scaffold", "status": "complete", "started": "...", "finished": "..."},
    {"name": "stub", "status": "complete", "started": "...", "finished": "..."},
    {"name": "refine", "status": "failed", "started": "...", "error": "..."}
  ]
}
```

### Cache Invalidation

Input hashing: `pipeline.stage` hashes the `input` argument. If the hash differs from the stored `.hash` file, the stage re-executes. Downstream stages are automatically invalidated (cascade).

Manual invalidation: `pipeline.invalidate pipe "scaffold"` forces scaffold and all subsequent stages to re-run.

### Visualization

`pipeline.status` returns structured data compatible with `std/diag`:

```lx
pipe_status = pipeline.status pipe
emit pipe_status.stages | map (s) "{s.name}: {s.status}" | join " → "
```

### Integration with `std/trace`

Each `pipeline.stage` creates a trace span. Resumed stages get a span with `resumed: true` annotation. Cache hits get `cached: true`.

### Integration with `std/budget`

When a stage is cached, its budget cost is zero. `pipeline.status` reports per-stage budget usage for completed stages, enabling cost analysis of re-runs.

## Implementation

Pure stdlib module. No parser changes. Core: JSON serialization of stage outputs (reuses `std/json`), file I/O (reuses `std/fs` patterns), SHA-256 hashing of inputs for cache keys.

Approximately 180 lines of Rust. The `pipeline.stage` function is the main complexity: check cache → validate hash → execute or return cached → persist result.

## Priority

Tier 2. Directly prevents the most frustrating failure mode in multi-stage flows: losing 30 minutes of completed work because stage N+1 fails. No parser changes. No dependencies beyond `std/json` and `std/fs` (both implemented).
