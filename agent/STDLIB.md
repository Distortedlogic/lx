-- Memory: ISA manual (stdlib). Standard library modules and built-in functions.
-- Update when stdlib modules are added or changed. See also LANGUAGE.md and AGENTS.md.

# lx Standard Library

**Note:** 10 packages in `pkg/`. Import the Class/Trait name:
`use pkg/collection {Collection}`, `use pkg/knowledge {KnowledgeBase}`, `use pkg/tasks {TaskStore}`,
`use pkg/trace {TraceStore}`, `use pkg/memory {MemoryStore}`, `use pkg/context {ContextWindow}`,
`use pkg/circuit {CircuitBreaker}`, `use pkg/introspect {Inspector}`, `use pkg/pool {Pool}`.
`pkg/prompt` remains a pure record builder. 5 collection packages (knowledge, tasks, memory, trace, context)
use `entries: Store ()` + Collection Trait for generic operations. Construct with `ClassName {field: val}`
or `ClassName ()`. Methods via `instance.method args`.

## AI (std/ai)

```lx
use std/ai
resp = ai.prompt "Summarize this code" ^
resp = ai.prompt_with {
  prompt: "Analyze..."
  append_system: "You are a code reviewer."
  tools: ["Read" "Grep" "Bash"]
  max_turns: 10
} ^
resp.text                    -- the response text

result = ai.prompt_structured ScoreProtocol "Rate this" ^
result = ai.prompt_json "Classify this intent" {intent: "" findings: [""]} ^
```

## Store (first-class Value)

`Store` is a first-class value type (`Value::Store { id }`) with dot-access methods:

```lx
s = Store ()
s.set "key" value              s.get "key"
s.keys ()                      s.values ()
s.entries ()                   s.has "key"
s.len ()                       s.remove "key"
s.clear ()                     s.update "key" (v) v + 1
s.filter (k v) condition       s.query {field: "value"}
s.map (k v) transform          s.save "path.json"
s.load "path.json"             s.persist "path.json"
s.reload "path.json"
```

Reference semantics: `a = b` shares the same Store. Store cloning in Class constructors ensures each instance gets its own copy.

## Collection Trait (pkg/collection)

Generic operations for any Class with `entries: Store ()`. Provides 9 default methods: `get`, `keys`, `values`, `remove`, `query`, `len`, `has`, `save`, `load` — all delegating to `self.entries`. Any conforming Class gets these for free; domain-only methods remain on the Class. Used by: KnowledgeBase, TaskStore, TraceStore, MemoryStore, ContextWindow.

## Prompt Assembly (pkg/prompt)

```lx
use pkg/prompt
p = prompt.create ()
  | prompt.system "You are a code auditor"
  | prompt.section "Checklist" audit_text
  | prompt.instruction "Produce a findings report"
  | prompt.constraint "Only report problems"
  | prompt.example "Example finding: ..."
rendered = prompt.render p
```

## Tracing (pkg/trace)

```lx
use pkg/trace {TraceStore}
t = TraceStore ()
t.record {name: "step1" input: x output: y} ^
sum = t.summary ()
rate = t.improvement_rate 3
stop = t.should_stop {min_delta: 2.0 window: 3}
```

## Grading and Auditing

```lx
use std/audit
use std/agents/grader
use std/agents/auditor

rubric = audit.rubric [
  {name: "coverage" description: "covers all items" weight: 50}
  {name: "quality" description: "clear and actionable" weight: 50}
]

grade = grader.grade {work: draft  task: "review doc"  rubric: rubric  threshold: 75}
check = audit.quick_check {output: text  task: "documentation"}
full = auditor.audit {output: text  task: "documentation"}
```

## Deadline (Time Propagation)

```lx
use std/deadline

dl = deadline.create 5000 ^
body = () {
  remaining = deadline.remaining () ^
  expired = deadline.expired () ^
  deadline.check () ^
  sub = deadline.slice 0.3 ^
  remaining
}
result = deadline.scope dl body ^

dl2 = deadline.create_at (time.now().ms + 10000) ^
deadline.extend dl2 5000 ^
```

`deadline.scope` establishes a deadline context. `remaining`, `expired`, `check`, and `slice` read the current scope (thread-local stack). `scope` returns `Result Any Str`. When `~>?`/`~>` is called inside a scope, `_deadline_ms` is auto-injected into Record messages.

## Budget (Cost Tracking)

```lx
use std/budget
b = budget.create {total: 10.0 unit: "dollars"}
b = budget.spend 2.5 b ^
budget.remaining b    -- 7.5
budget.used_pct b     -- 25.0
sub = budget.slice 0.3 b ^  -- sub-budget (30% of remaining)
```

## Retry with Backoff

```lx
use std/retry
result = retry.retry flaky_fn
result = retry.retry_with {max_attempts: 5  base_delay_ms: 200} flaky_fn
```

## Pipeline (std/pipeline)

```lx
use std/pipeline

pipe = pipeline.create "my-pipeline" {storage: ".lx/pipelines/"} ^

result1 = pipeline.stage pipe "step1" input_data (input) {
  process input ^
} ^

result2 = pipeline.stage pipe "step2" result1 (input) {
  transform input ^
} ^

pipeline.complete pipe ^

st = pipeline.status pipe
pipeline.invalidate pipe "step1"
pipeline.clean pipe
all = pipeline.list ()
```

`pipeline.stage` caches completed stage outputs. On re-run with the same input, cached results are returned without re-executing the body. If input changes (hash mismatch), the stage re-executes. `invalidate`/`invalidate_from` remove a stage's cache plus all downstream stages.

## Other Stdlib Modules

| Module            | Purpose                                                              |
|-------------------|----------------------------------------------------------------------|
| `std/json`        | `parse`, `encode`, `encode_pretty`                                   |
| `std/fs`          | `read`, `write`, `append`, `exists`, `stat`, `mkdir`, `ls`, `remove` |
| `std/env`         | `get`, `vars`, `args`, `cwd`, `home`                                 |
| `std/http`        | `get`, `post`, `put`, `delete`                                       |
| `std/re`          | `is_match`, `match`, `find_all`, `replace`, `split`                  |
| `std/md`          | `parse`, `sections`, `code_blocks`, `headings`, `render`, builders   |
| `std/math`        | `abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`, `min`, `max`         |
| `std/time`        | `now`, `sleep`, `format`, `parse`                                    |
| `std/git`         | 36 functions: status, log, diff, blame, grep, commit, branch, etc.   |
| `std/ctx`         | Key-value context: `empty`, `set`, `get`, `merge`, `save`, `load`    |
| `std/deadline`    | Time budgets: `create`, `create_at`, `scope`, `remaining`, `expired`, `check`, `slice`, `extend` |
| `std/pipeline`    | Stage caching: `create`, `stage`, `complete`, `status`, `invalidate`, `clean`, `list` |
| `std/plan`        | Plan execution: `run` with `on_step` callback, `replan`, `skip`      |
| `std/saga`        | Compensating transactions: `run`, `define`, `execute`                |
| `std/cron`        | Scheduling: `every`, `after`, `at`, `schedule`, `run`                |
| `std/user`        | Interactive: `confirm`, `choose`, `ask`, `progress`, `table`         |
| `std/profile`     | Persistent identity: `load`, `save`, `learn`, `recall`, `preference` |
| `std/diag`        | Visualization: `extract`, `to_mermaid`                               |
| `std/flow`        | Flow composition: `load`, `run`, `pipe`, `parallel`, `branch`, `with_retry`, `with_timeout`, `with_fallback` |
| `std/taskgraph`   | DAG execution: `create`, `add`, `remove`, `run`, `run_with`, `validate`, `topo`, `status`, `dot` |

## Flow Composition (std/flow)

```lx
use std/flow

f = flow.load "review.lx" ^
result = flow.run f {task: "review"} ^

pipeline = flow.pipe [
  flow.load "extract.lx" ^
  flow.load "transform.lx" ^
]
result = flow.run pipeline input ^

ensemble = flow.parallel [
  flow.load "reviewer1.lx" ^
  flow.load "reviewer2.lx" ^
]
results = flow.run ensemble input ^

resilient = flow.load "flaky.lx" ^
  | flow.with_timeout 300
  | flow.with_retry {max: 3}
  | flow.with_fallback (flow.load "safe.lx" ^)
result = flow.run resilient input ^
```

`flow.load` reads and parses a .lx file, returning a Flow record. `flow.run` executes in an isolated interpreter with shared RuntimeCtx. Flows must export `+run` or `+main`. `flow.branch` takes a router function that receives input and returns a Flow.

## Task Graphs (std/taskgraph)

```lx
use std/taskgraph

g = taskgraph.create "code-review" ^
taskgraph.add g "parse" {handler: parse_fn  input: {files: changed}} ^
taskgraph.add g "lint" {handler: lint_fn  input: {files: changed}} ^
taskgraph.add g "review" {
  depends: ["parse" "lint"]
  input_from: (results) {ast: results.parse.ast  warnings: results.lint.issues}
  handler: review_fn
} ^
results = taskgraph.run g ^
```

Task options: `handler` (function), `input` (static), `depends` (task ID list), `input_from` (transform dep results), `timeout` (ms), `retry` (count), `on_fail` ("fail"|"skip"). `taskgraph.validate` checks cycles + unknown deps. `taskgraph.topo` returns topological order. `taskgraph.dot` exports DOT graph. `taskgraph.run_with` adds `on_complete`/`on_fail` callbacks and `max_parallel`.

## Standard Agents

Six pre-built agents under `std/agents/`:

| Module                | Functions                | Use for                          |
|-----------------------|--------------------------|----------------------------------|
| `std/agents/auditor`  | `quick_audit`, `audit`   | Output quality checking          |
| `std/agents/grader`   | `quick_grade`, `grade`   | Rubric-based scoring             |
| `std/agents/router`   | `quick_route`, `route`   | Message routing decisions        |
| `std/agents/planner`  | `quick_plan`, `plan`     | Task decomposition               |
| `std/agents/reviewer` | `quick_review`, `review` | Code/document review             |
| `std/agents/monitor`  | `scan_actions`, `check`  | Behavioral monitoring            |

## Built-in Functions (No Import Needed)

**Transform:** `map`, `flat_map`, `scan`, `fold`, `sum`, `product`
**Filter:** `filter`, `take`, `drop`, `take_while`, `drop_while`
**Search:** `find`, `find_index`, `first`, `last`, `get`
**Predicates:** `any?`, `all?`, `none?`, `count`, `empty?`, `contains?`, `has_key?`, `sorted?`
**Sort:** `sort`, `sort_by`, `rev`, `min`, `max`, `min_by`, `max_by`
**Reshape:** `chunks`, `windows`, `partition`, `group_by`, `zip`, `enumerate`
**Flatten:** `flatten`, `intersperse`, `uniq`
**Convert:** `to_list`, `to_map`, `to_record`, `keys`, `values`, `entries`, `merge`, `remove`
**String:** `len`, `chars`, `lines`, `split`, `join`, `trim`, `upper`, `lower`, `replace`, `starts?`, `ends?`, `pad_left`, `pad_right`, `repeat`
**Numeric:** `even?`, `odd?`, `parse_int`, `parse_float`, `to_int`, `to_float`
**Type:** `type_of`, `to_str`, `ok?`, `err?`, `some?`
**Effects:** `each` (returns unit), `tap` (returns original), `dbg` (debug print), `print` (stdout)
**Control:** `identity`, `not`, `require`, `timeout`, `step`, `collect`
**Logging:** `log.info`, `log.warn`, `log.err`, `log.debug`

## Idioms

**Pipe-first design:**
```lx
audit_items
  | map (item) investigate item root ^
  | filter (.severity == "critical")
  | sort_by (.confidence) | rev
  | take 10
```

**Sections over lambdas:** `items | map (.name) | filter (starts? "test") | sort`

**`^` early in pipes:** `url | fetch ^ | (.body) | json.parse ^ | (.data) | map process`

**Prompt composition:** Use `pkg/prompt` builder, not string concatenation.

**Scoped resources:** Always `with ... as` for connections needing cleanup.

**Fan-out + reconcile:**
```lx
results = par { a ~>? msg ^; b ~>? msg ^; c ~>? msg ^ }
merged = agent.reconcile [results.0 results.1 results.2] {strategy: "merge_fields"}
```
