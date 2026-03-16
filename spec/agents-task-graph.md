# Task Graph — DAG-Aware Subtask Decomposition

Dependency-ordered subtask decomposition, assignment, and result aggregation for multi-agent workflows.

## Problem

`std/plan` treats plans as linear step lists with `on_step` callbacks. `std/tasks` tracks task state but has no dependency edges. `agent.reconcile` merges parallel results post-hoc. `std/pool` fans out independent work.

None of these handle the common pattern: **"task C depends on tasks A and B completing first."** Today you manually `agent.spawn` N agents, manually track which depends on which, manually `~>?` in the right order, and manually merge results. Every non-trivial multi-agent workflow reinvents this DAG scheduling.

What's needed: a first-class task graph where you declare tasks + dependencies, assign agents, and the runtime executes in dependency order with maximum parallelism.

## Design

### Module: `std/taskgraph`

```lx
use std/taskgraph

g = taskgraph.create "code-review" ^

taskgraph.add g "parse" {
  agent: parser_agent
  input: {files: changed_files}
} ^

taskgraph.add g "lint" {
  agent: lint_agent
  input: {files: changed_files}
} ^

taskgraph.add g "review" {
  agent: reviewer_agent
  depends: ["parse" "lint"]
  input_from: (results) {
    ast: results.parse.ast
    warnings: results.lint.issues
  }
} ^

taskgraph.add g "summarize" {
  agent: summary_agent
  depends: ["review"]
  input_from: (results) results.review
} ^

results = taskgraph.run g ^ -- executes parse+lint in parallel, then review, then summarize
```

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `taskgraph.create` | `(name: Str) -> Result TaskGraph Str` | Create empty graph |
| `taskgraph.add` | `(g: TaskGraph id: Str opts: Record) -> Result () Str` | Add task node |
| `taskgraph.remove` | `(g: TaskGraph id: Str) -> Result () Str` | Remove task (and edges) |
| `taskgraph.run` | `(g: TaskGraph) -> Result Record Str` | Execute graph, return `{task_id: result}` |
| `taskgraph.run_with` | `(g: TaskGraph opts: Record) -> Result Record Str` | Execute with options |
| `taskgraph.validate` | `(g: TaskGraph) -> Result () Str` | Check for cycles, missing deps |
| `taskgraph.topo` | `(g: TaskGraph) -> Result [Str] Str` | Return topological ordering |
| `taskgraph.status` | `(g: TaskGraph) -> Record` | Current execution status per task |
| `taskgraph.dot` | `(g: TaskGraph) -> Str` | Export as DOT graph |

### Task Options

```lx
taskgraph.add g "review" {
  agent: reviewer          -- agent to send work to (via ~>?)
  input: {files: fs}       -- static input (used if no depends)
  depends: ["parse" "lint"] -- task IDs that must complete first
  input_from: (results) .. -- transform dependency results into input
  timeout: 30000           -- per-task timeout in ms
  retry: 2                 -- retry count on failure
  on_fail: "skip"          -- "fail" (default), "skip", or custom (Fn)
}
```

### Execution Model

`taskgraph.run` performs topological sort, then executes tasks in waves — all tasks whose dependencies are satisfied run in parallel (via `par`). Each task sends its input to its assigned agent via `~>?`. Results are collected in a `{task_id: result}` record.

If a task fails:
- `on_fail: "fail"` (default) — entire graph fails, returns `Err {task: id error: e completed: partial_results}`
- `on_fail: "skip"` — downstream tasks that depend on the failed task are also skipped, rest continue
- `on_fail: custom_fn` — `(task_id error) -> PlanAction` using same control vocabulary as `std/plan`

### `run_with` Options

```lx
results = taskgraph.run_with g {
  on_complete: (id result) log.info "done: {id}"
  on_fail: (id err) log.err "failed: {id}: {err}"
  budget: my_budget       -- std/budget instance, checked before each task
  max_parallel: 4          -- limit concurrent tasks
} ^
```

### Relationship to Existing Modules

- `std/plan` — linear step execution. Task graph is for DAG-shaped work. Plans can contain a `taskgraph.run` as a step.
- `std/tasks` — task state tracking (create/start/submit/pass/fail). Task graph uses tasks internally for tracking but adds dependency ordering and parallel execution.
- `std/pool` — worker pools for homogeneous fan-out. Task graph is for heterogeneous tasks with dependencies.
- `agent.reconcile` — post-hoc result merging. Task graph's `input_from` handles result threading between dependent tasks.
- `std/pipeline` (planned) — linear stage-by-stage with checkpoint/resume. Task graph is DAG-shaped. Pipelines could use task graph internally for stages that have sub-dependencies.

## Implementation

Pure stdlib module. No parser changes. Core data structure: adjacency list (`Map<String, Vec<String>>` for deps), task config map, result accumulator. Topological sort via Kahn's algorithm. Cycle detection during `validate` and at start of `run`.

Approximately 200 lines of Rust. Execution uses existing `par` infrastructure for parallel waves.

## Priority

Tier 2. Fills a structural gap that every non-trivial multi-agent flow hits. No parser changes. No dependencies on unimplemented features (uses existing `~>?`, `par`, `std/tasks`).
