# Writing lx Programs

This is everything you need to write idiomatic lx programs. lx is an agentic workflow language — it exists so agents can spawn sub-agents, pass messages, invoke tools, and orchestrate multi-step workflows with minimal syntax overhead.

Read this file top-to-bottom. Later sections build on earlier ones.

## Basics

```lx
x = 42                       -- immutable binding
x := 0                       -- mutable binding
x <- x + 1                   -- reassign mutable
name = "world"
greeting = "hello {name}"    -- string interpolation
raw = `no {interpolation}`   -- raw string (backtick)
```

Collections:
```lx
[1 2 3]                      -- list (space-separated, no commas)
{x: 1  y: 2}                 -- record (fixed keys, all fields need key: value)
{:}                          -- empty record
(1 "hello" true)             -- tuple
%{"key": "value"}            -- map (arbitrary keys)
```

Access and update:
```lx
xs.0  xs.-1                  -- index / negative index
xs.1..3                      -- slice (start..end exclusive)
record.name                  -- field access
{..record  x: 5}             -- record update (new record)
[..xs 4 5]                   -- list spread
```

## Functions

```lx
double = (x) x * 2
add = (x y) x + y
greet = (name  greeting = "hello") "{greeting} {name}"
make_adder = (n) (x) x + n  -- closure
```

Type annotations (validated by `lx check`, ignored by `lx run`):
```lx
add = (x: Int y: Int) -> Int x + y
safe_div = (a: Int b: Int) -> Int ^ Str { ... }
```

## Arithmetic

`/` always returns Float (even for two Ints). `//` is integer division:
```lx
7 / 2          -- 3.5 (Float)
7 // 2         -- 3 (Int)
15 / 3         -- 5.0 (Float)
```

Mixed Int/Float auto-promotes to Float:
```lx
3 * 0.5        -- 1.5 (Int * Float → Float)
10 + 2.0       -- 12.0 (Int + Float → Float)
```

## Pipes — The Core Composition Tool

`|` passes the left value as the **last** argument to the right function. This is the primary way to compose operations in lx:

```lx
xs | map (* 2) | filter (> 0) | sum
url | fetch ^ | (.body) | json.parse ^
names | sort | take 5 | join ", "
```

The `(* 2)`, `(> 0)`, `(.body)` are **sections** — partial application syntax:

```lx
(+ 1)      -- (x) x + 1
(* 2)      -- (x) x * 2
(> 0)      -- (x) x > 0
(.name)    -- (x) x.name
(10 -)     -- (x) 10 - x
(?? 0)     -- (x) x ?? 0
```

Sections are the idiomatic way to create short lambdas in pipes. Prefer `map (.name)` over `map (x) x.name`.

## Error Handling

Errors are values, not exceptions. Two families:

```lx
Ok 42          Err "failed"     -- Result
Some "hi"      None             -- Maybe
```

Two operators handle them:

```lx
value = risky_call ^            -- ^ unwraps Ok/Some, propagates Err/None up
fallback = risky_call ?? "default"  -- ?? provides fallback on Err/None
```

These compose in pipes:
```lx
url | fetch ^ | (.body) | json.parse ^      -- any failure propagates
config | (.timeout) ?? 30                     -- missing field → default
```

Predicates: `ok?`, `err?`, `some?` test Result/Maybe values.

`require` converts Maybe to Result: `require "need value" maybe_val`.

## Pattern Matching

```lx
x ? {
  0 -> "zero"
  1 | 2 -> "small"
  n & (n > 100) -> "big {n}"
  _ -> "other"
}

shape ? { Circle r -> r * r  Rect w h -> w * h  Dot -> 0 }
result ? { Ok v -> v  Err e -> handle e }
{name: "alice" ..} -> "found alice"     -- record pattern with rest

x > 0 ? "positive" : "non-positive"     -- ternary
x > 0 ? do_thing x                      -- single-arm (unit if false)
```

Destructuring in bindings:
```lx
(a b c) = (1 2 3)           -- tuple
{name  age} = person         -- record
[first ..rest] = items       -- list with rest
```

## Type Definitions

```lx
Point = {x: Float  y: Float}
Shape = | Circle Float | Rect Float Float | Dot
Tree a = | Leaf a | Node (Tree a) (Tree a)
```

Constructors work as functions: `Circle 5.0`, `Node (Leaf 1) (Leaf 2)`.

## Modules

```lx
use std/json                   -- whole module (json.parse, json.encode)
use std/json : j               -- aliased (j.parse, j.encode)
use std/json {parse encode}    -- selective (parse, encode as bare names)
use ./util                     -- relative import
+exported_fn = (x) x * 2      -- + prefix = exported
```

## Control Flow

```lx
1..10 | each (n) { process n }         -- iteration
loop { condition ? break value }        -- loop with break
par { fetch url1 ^; fetch url2 ^ }      -- parallel (returns tuple)
sel { fetch url -> it; timeout 5 -> Err "slow" }  -- race
xs | pmap fetch                          -- parallel map
xs | pmap_n 10 fetch                     -- rate-limited parallel map
```

## Shell Integration

```lx
r = $echo "hello {name}"     -- Result {out err code} ShellErr
s = $^pwd | trim              -- $^ extracts stdout string directly
block = ${                    -- multi-line session (commands share state)
  cd /tmp
  pwd
}
```

`|` inside `$` is a shell pipe. To chain to lx: `($^ls src) | lines`.

## Concurrency

```lx
(a b c) = par {
  fetch url1 ^
  fetch url2 ^
  fetch url3 ^
}

winner = sel {
  fetch url   -> it          -- `it` = result of completed arm
  timeout 5   -> Err "slow"
}
```

`par` runs all arms, returns tuple. `sel` races, first wins. `^` in any arm cancels siblings on error.

Mutable bindings (`:=`) cannot be captured in `par`/`sel`/`pmap` bodies.

## Agent System — The Core of lx

### Protocols (Message Contracts)

Protocols validate message shapes at runtime:

```lx
Protocol ReviewRequest = {file: Str  depth: Str = "standard"}
Protocol ReviewResult = {approved: Bool  findings: [Str]}
Protocol AgentMsg = ReviewRequest | ReviewResult   -- union (auto-injects _variant)
Protocol Score = {
  value: Float where value >= 0.0 && value <= 1.0  -- field constraint
}
```

Composition: `Protocol Extended = {..Base  extra: Str}`.

### Traits (Behavioral Contracts)

Typed method signatures using MCP syntax (`{input} -> output`):

```lx
Trait Reviewer = {
  description: "Code review agent"
  review: {file: Str  depth: Str = "normal"} -> {approved: Bool  findings: List}
  summarize: {findings: List} -> Str
  requires: [:ai :fs]
  tags: ["code" "review"]
}
```

Methods can reference named Protocols as input: `review: ReviewRequest -> {findings: List}`.

Discovery via `std/trait`:

```lx
use std/trait
methods = trait.methods Reviewer
best = trait.match Reviewer "find issues"
```

### Agent Declarations

```lx
Agent CodeReviewer: Reviewer = {
  review = (msg) {
    analysis = ai.prompt "Review {msg.file}" ^
    {approved: true  findings: [analysis.text]}
  }
  summarize = (msg) msg.findings | join "\n"
}
```

Trait conformance validated at definition time — missing methods halt execution.
Access methods via `.`: `CodeReviewer.review {file: "main.rs"}`.
Reserved fields: `uses` (MCP connections), `init` (startup logic), `on` (lifecycle hooks).

### Agent Messaging

```lx
-- Spawn a subprocess agent
worker = agent.spawn {command: "lx" args: ["run" "worker.lx"]} ^

-- Ask (request-response) and send (fire-and-forget)
result = worker ~>? {task: "analyze" file: "main.rs"} ^
worker ~> {status: "done"}

-- Pipeline: spawn → ask → process result
worker ~>? {task: "review"} ^ | (.findings) | filter (.critical)

agent.kill worker
```

### Scoped Resources (with ... as)

Auto-cleanup with LIFO close order:
```lx
with mcp.connect {command: "npx" args: ["server"]} ^ as conn {
  tools = mcp.list_tools conn ^
  result = mcp.call conn "read_file" {path: "src/main.rs"} ^
}  -- conn auto-closed here, even on error
```

Multiple resources, scoped bindings:
```lx
with conn1 as c1, conn2 as c2 { use_both c1 c2 }
with x = compute_value (), y = other () { x + y }
with mut counter = 0 { counter <- counter + 1; counter }
```

### MCP Declarations (Tool Contracts)

```lx
MCP Tools = {
  read_file { path: Str } -> { content: Str }
  list_dir { path: Str } -> [{ name: Str  kind: Str }]
}
```

### Yield and Emit

```lx
yield {kind: "approval" data: changes}   -- pause for orchestrator input
emit "Status update"                      -- fire-and-forget to human (strings → stdout)
emit {progress: 50 stage: "analyzing"}   -- structured emit (records → JSON)
```

### Receive (Agent Message Handler)

`receive` replaces the yield/loop/match boilerplate for agent message handlers:

```lx
receive {
  analyze -> (msg) analyze_fn msg
  compare -> (msg) compare_fn msg
  _ -> (msg) Err "unknown action"
}
```

Desugars to: yield `{kind: "ready"}`, enter loop, dispatch on `msg.action`, yield `{kind: "result" data: result}`, break on None.

### Refine (Iterative Improvement)

The `refine` expression replaces manual grade/revise loops:

```lx
result = refine initial_draft {
  grade: (work) {score: evaluate work  feedback: "..."}
  revise: (work feedback) improve work feedback
  threshold: 85
  max_rounds: 5
  on_round: (round work score) log.info "round {round}: {score}"
}
-- Returns Ok {work rounds final_score} or Err {work rounds final_score reason}
```

Use `refine` instead of manually coding `loop` + grade + break + revise.

## Agent Communication Extensions

All under `use std/agent`:

### Dialogue (Multi-Turn Sessions)

```lx
session = agent.dialogue worker {role: "reviewer" context: "..." max_turns: 10} ^
r1 = agent.dialogue_turn session "review the auth module" ^
r2 = agent.dialogue_turn session "what about the error handling?" ^
agent.dialogue_end session
```

### Dispatch (Pattern-Based Routing)

```lx
dispatcher = agent.dispatch [
  {match: {domain: "security"} to: sec_agent}
  {match: (msg) msg.priority == "critical" to: fast_agent}
  {match: "default" to: general_agent}
]
dispatcher ~>? msg ^                           -- routes to first match
agent.dispatch_multi dispatcher msg ^          -- fan-out to ALL matching
```

### Reconciliation (Merge Parallel Results)

```lx
decision = agent.reconcile results {
  strategy: "vote"  quorum: "majority"  deliberate: 2
}
-- Strategies: "union", "intersection", "vote", "highest_confidence",
--             "max_score", "merge_fields", or custom Fn
```

### Supervision (Erlang-Style)

```lx
sup = agent.supervise {
  strategy: "one_for_one"  max_restarts: 5  window: 60
  children: [
    {id: "worker" spawn: () agent.spawn {...} ^ restart: "permanent"}
  ]
}
```

### Message Middleware

```lx
traced = agent.intercept worker (msg next) {
  log.debug "msg: {msg | to_str}"
  next msg    -- call next to continue, or return early to short-circuit
}
```

### Pub/Sub

```lx
t = agent.topic "updates"
agent.subscribe t worker
agent.subscribe_filtered t worker (msg) msg.priority == "critical"
agent.publish t {kind: "status" data: "running"}
responses = agent.publish_collect t msg ^
```

### Other Extensions

```lx
-- Capability discovery
caps = agent.capabilities worker ^
agent.advertise {protocols: [...] domains: [...] tools: [...]}

-- Human-in-the-loop gate
gate = agent.gate "deploy" {show: {diff: changes} timeout: 300 on_timeout: "abort"}

-- Handoff context for LLM prompts
use std/agent {Handoff}
context_str = agent.as_context handoff_record

-- Multi-agent negotiation
result = agent.negotiate agents {topic: "approach" max_rounds: 5 strategy: "convergence"}

-- Mock agents for testing
mock = agent.mock [
  {match: {task: "review"} respond: {approved: true}}
  {match: "any" respond: {error: "unexpected"}}
]
agent.mock_assert_called mock {task: "review"} ^
```

## Stdlib Tour

### AI (std/ai)

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

-- Protocol-validated structured output (auto-retries on schema violation)
result = ai.prompt_structured {prompt: "Rate this"  protocol: ScoreProtocol} ^

-- Lightweight structured JSON output (no Protocol needed, shape from example record)
result = ai.prompt_json "Classify this intent" {intent: "" findings: [""]} ^
```

### Prompt Assembly (std/prompt)

Build prompts compositionally instead of string concatenation:

```lx
use std/prompt
p = prompt.create ()
  | prompt.system "You are a code auditor"
  | prompt.section "Checklist" audit_text
  | prompt.section "Findings" findings
  | prompt.instruction "Produce a findings report"
  | prompt.constraint "Only report problems, not things that are correct"
  | prompt.example "Example finding: ..."
rendered = prompt.render p
```

### Tracing (std/trace)

```lx
use std/trace
session = trace.create "/tmp/trace.json" ^
trace.record {name: "step1" input: x output: y} session ^
trace.record {name: "step2" input: y output: z score: 0.85} session ^

-- Diminishing returns detection (for refine loops)
rate = trace.improvement_rate session
stop = trace.should_stop {min_delta: 2.0 window: 3} session
```

### Grading and Auditing

```lx
use std/audit
use std/agents/grader
use std/agents/auditor

rubric = audit.rubric [
  {name: "coverage" description: "covers all items" weight: 50}
  {name: "quality" description: "clear and actionable" weight: 50}
]

grade = grader.grade {work: draft  task: "review doc"  rubric: rubric  threshold: 75}
-- grade.score, grade.passed, grade.feedback, grade.categories

check = audit.quick_check {output: text  task: "documentation"}
-- check.passed, check.reasons

full = auditor.audit {output: text  task: "documentation"}
-- full.passed, full.score, full.feedback
```

### Budget (Cost Tracking)

```lx
use std/budget
b = budget.create {total: 10.0 unit: "dollars"}
b = budget.spend 2.5 b ^
budget.remaining b    -- 7.5
budget.used_pct b     -- 25.0
budget.project b 3    -- projected cost for 3 more ops
sub = budget.slice 0.3 b ^  -- sub-budget (30% of remaining)
```

### Context Windows

```lx
use std/context
ctx = context.create {max_tokens: 4000}
ctx = context.add {role: "user" content: msg} ctx
context.pressure ctx       -- 0.0 to 1.0
ctx = context.pin "system" ctx    -- protect from eviction
ctx = context.evict_until 0.5 ctx -- evict until 50% usage
```

### Circuit Breakers

```lx
use std/circuit
cb = circuit.create {threshold: 3 window: 60 cooldown: 30}
cb = circuit.record true cb    -- record success
cb = circuit.record false cb   -- record failure
circuit.is_tripped cb          -- true if failures >= threshold
```

### Retry with Backoff

```lx
use std/retry
result = retry.retry flaky_fn              -- 3 attempts, exponential backoff
result = retry.retry_with {
  max_attempts: 5  base_delay_ms: 200
} flaky_fn
-- Ok value on success, Err Exhausted {attempts last_error elapsed_ms} on exhaustion
```

### Worker Pools

```lx
use std/pool
p = pool.create {size: 4 workers: [w1 w2 w3 w4]}
results = pool.fan_out p tasks ^     -- distribute tasks across workers
results = pool.map p items process ^ -- parallel map over pool
```

### Tasks

```lx
use std/tasks
ts = tasks.empty ()
ts = tasks.create ts "review" {priority: "high" parent: "audit"}
ts = tasks.start ts "review" ^
ts = tasks.submit ts "review" {output: findings} ^
ts = tasks.pass ts "review" ^
```

### Other Stdlib

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
| `std/plan`        | Plan execution: `run` with `on_step` callback, `replan`, `skip`      |
| `std/saga`        | Compensating transactions: `run`, `define`, `execute`                |
| `std/cron`        | Scheduling: `every`, `after`, `at`, `schedule`, `run`                |
| `std/introspect`  | Self-observation: `elapsed`, `actions`, `is_stuck`, `strategy_shift`  |
| `std/user`        | Interactive: `confirm`, `choose`, `ask`, `progress`, `table`         |
| `std/profile`     | Persistent identity: `load`, `save`, `learn`, `recall`, `preference` |
| `std/diag`        | Visualization: `extract`, `to_mermaid`                               |

### Standard Agents

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

## Idioms and Patterns

### Pipe-First Design

Structure programs as data flowing left-to-right:
```lx
audit_items
  | map (item) investigate item root ^
  | filter (.severity == "critical")
  | sort_by (.confidence) | rev
  | take 10
  | map (item) {..item  fix: generate_fix item ^}
```

### Sections Over Lambdas

```lx
-- Prefer
items | map (.name) | filter (starts? "test") | sort
-- Over
items | map (x) x.name | filter (x) starts? "test" x | sort
```

### `^` Early in Pipes

Propagate errors as early as possible:
```lx
url | fetch ^ | (.body) | json.parse ^ | (.data) | map process
```

### Prompt Composition Pattern

Build prompts with `std/prompt` instead of string concatenation:
```lx
p = prompt.create ()
  | prompt.system "You are a {role}"
  | prompt.section "Context" context_text
  | prompt.section "Input" input_text
  | prompt.instruction "Do X"
  | prompt.constraint "Do not Y"
resp = ai.prompt_with {prompt: (prompt.render p)  tools: [...]} ^
```

### Refine Over Manual Loops

Replace manual grade/revise loops with `refine`:
```lx
-- Instead of manually coding loop + grade + break + revise:
result = refine initial_work {
  grade: (work) grader.grade {work: work  task: task  rubric: rubric  threshold: 80}
  revise: (work feedback) revise_with_ai work feedback ^
  threshold: 80
  max_rounds: 5
}
```

### Scoped Resources

Always use `with ... as` for connections that need cleanup:
```lx
with mcp.connect {command: "npx" args: ["-y" "@server/tools"]} ^ as tools {
  result = mcp.call tools "read_file" {path: "main.rs"} ^
}
-- tools.close() called automatically, even on error
```

### Fan-Out + Reconcile

Parallel work with merged results:
```lx
results = par {
  reviewer ~>? {task: "review" file: f} ^
  auditor ~>? {task: "audit" file: f} ^
  checker ~>? {task: "check" file: f} ^
}
merged = agent.reconcile [results.0 results.1 results.2] {
  strategy: "merge_fields"
}
```

## Gotchas

See `agent/GOTCHAS.md` for non-obvious behaviors and temporary workarounds.

## Operator Precedence (high to low)

`.` > juxtaposition > unary > `*/%//` > `+-` > `..` > `++ ~> ~>?` > `|` > comparisons > `&&` > `||` > `??` > `^` > `&` > `->` > `?` > `= := <-`

Key consequence: `data | sort | len > 5` = `((data | sort) | len) > 5` — pipe binds tighter than comparison.
