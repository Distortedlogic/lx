-- Memory: ISA manual (stdlib). Standard library modules and built-in functions.
-- Update when stdlib modules are added or changed. See also LANGUAGE.md and AGENTS.md.

# lx Standard Library

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

## Prompt Assembly (std/prompt)

```lx
use std/prompt
p = prompt.create ()
  | prompt.system "You are a code auditor"
  | prompt.section "Checklist" audit_text
  | prompt.instruction "Produce a findings report"
  | prompt.constraint "Only report problems"
  | prompt.example "Example finding: ..."
rendered = prompt.render p
```

## Tracing (std/trace)

```lx
use std/trace
session = trace.create "/tmp/trace.json" ^
trace.record {name: "step1" input: x output: y} session ^
rate = trace.improvement_rate session
stop = trace.should_stop {min_delta: 2.0 window: 3} session
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

## Budget (Cost Tracking)

```lx
use std/budget
b = budget.create {total: 10.0 unit: "dollars"}
b = budget.spend 2.5 b ^
budget.remaining b    -- 7.5
budget.used_pct b     -- 25.0
sub = budget.slice 0.3 b ^  -- sub-budget (30% of remaining)
```

## Context Windows

```lx
use std/context
ctx = context.create {max_tokens: 4000}
ctx = context.add {role: "user" content: msg} ctx
context.pressure ctx       -- 0.0 to 1.0
ctx = context.pin "system" ctx
ctx = context.evict_until 0.5 ctx
```

## Circuit Breakers

```lx
use std/circuit
cb = circuit.create {threshold: 3 window: 60 cooldown: 30}
cb = circuit.record true cb
circuit.is_tripped cb
```

## Retry with Backoff

```lx
use std/retry
result = retry.retry flaky_fn
result = retry.retry_with {max_attempts: 5  base_delay_ms: 200} flaky_fn
```

## Worker Pools

```lx
use std/pool
p = pool.create {size: 4 workers: [w1 w2 w3 w4]}
results = pool.fan_out p tasks ^
results = pool.map p items process ^
```

## Tasks

```lx
use std/tasks
ts = tasks.empty ()
ts = tasks.create ts "review" {priority: "high" parent: "audit"}
ts = tasks.start ts "review" ^
ts = tasks.submit ts "review" {output: findings} ^
ts = tasks.pass ts "review" ^
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
| `std/memory`      | Tiered memory: `create`, `store`, `recall`, `promote`, `consolidate` |
| `std/knowledge`   | File-backed KB: `create`, `store`, `get`, `query`, `merge`, `expire` |
| `std/pipeline`    | Stage caching: `create`, `stage`, `complete`, `status`, `invalidate`, `clean`, `list` |
| `std/plan`        | Plan execution: `run` with `on_step` callback, `replan`, `skip`      |
| `std/saga`        | Compensating transactions: `run`, `define`, `execute`                |
| `std/cron`        | Scheduling: `every`, `after`, `at`, `schedule`, `run`                |
| `std/introspect`  | Self-observation: `elapsed`, `actions`, `is_stuck`, `strategy_shift`  |
| `std/user`        | Interactive: `confirm`, `choose`, `ask`, `progress`, `table`         |
| `std/profile`     | Persistent identity: `load`, `save`, `learn`, `recall`, `preference` |
| `std/diag`        | Visualization: `extract`, `to_mermaid`                               |

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

**Prompt composition:** Use `std/prompt` builder, not string concatenation.

**Scoped resources:** Always `with ... as` for connections needing cleanup.

**Fan-out + reconcile:**
```lx
results = par { a ~>? msg ^; b ~>? msg ^; c ~>? msg ^ }
merged = agent.reconcile [results.0 results.1 results.2] {strategy: "merge_fields"}
```
