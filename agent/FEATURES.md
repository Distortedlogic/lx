# lx Language Features

Quick reference for writing lx programs. Organized by category with syntax examples.

## Literals & Primitives

```lx
42                     -- Int (arbitrary precision)
1_000_000              -- Int with separators
0xff  0b1010  0o77     -- Hex, binary, octal
3.14  1.5e2  1.5E-2    -- Float (IEEE 754 f64)
true  false            -- Bool
()                     -- Unit (zero-tuple)
"hello {name}"         -- String with interpolation
`raw {no interp}`      -- Raw string (backtick)
r/\d+/  r/[a-z]+/i     -- Regex literal with flags
```

## Collections

```lx
[1 2 3]                -- List (space-separated)
{x: 1  y: 2}          -- Record (fixed ident keys)
{x  y}                 -- Record shorthand (x: x  y: y)
(1 "hello" true)       -- Tuple (no commas)
%{"key": "value"}      -- Map (arbitrary expression keys)
```

**Access & Slicing:**
```lx
xs.0                   -- Index (zero-based)
xs.-1                  -- Negative index (last element)
xs.1..3                -- Slice (start inclusive, end exclusive)
xs.2..                 -- Slice from index to end
xs...3                 -- Slice from start to index
record.name            -- Field access
map."key"              -- Map string key access
```

**Spread & Update:**
```lx
[..xs 4 5]             -- List spread
[..xs ..ys]            -- List concat via spread
{..record  x: 5}      -- Record update
{..a  ..b}             -- Record merge (b wins)
```

## Bindings & Assignment

```lx
x = 5                  -- Immutable
x := 5                 -- Mutable
x <- 10                -- Reassign mutable
x: Int = 5             -- With type annotation
(a b c) = (1 2 3)      -- Tuple destructure
{name  age} = record   -- Record destructure
[first ..rest] = list   -- List destructure with rest
record.field <- value   -- Mutable record field update
```

## Operators (by precedence, high to low)

| Operator                    | Description                           |
| --------------------------- | ------------------------------------- |
| `.`                         | Field/index access                    |
| juxtaposition               | Function application                  |
| `-` `!` `not`               | Unary prefix                          |
| `*` `/` `%` `//`            | Multiplicative (// = integer div)     |
| `+` `-`                     | Additive                              |
| `..` `..=`                  | Range (exclusive, inclusive)          |
| `++` `~>` `~>?`             | Concat, agent send, agent ask         |
| `\|`                        | Pipe                                  |
| `==` `!=` `<` `>` `<=` `>=` | Comparison                            |
| `&&`                        | Logical and                           |
| `\|\|`                      | Logical or                            |
| `??`                        | Coalesce (default on Err/None)        |
| `^`                         | Error propagation (postfix)           |
| `&`                         | Pattern guard (in ? arms)             |
| `->`                        | Arm body / type arrow                 |
| `?`                         | Match / ternary                       |
| `=` `:=` `<-`               | Binding / assignment                  |

## Functions

```lx
double = (x) x * 2
add = (x y) x + y
greet = (name  greeting = "hello") "{greeting} {name}"

add = (x: Int y: Int) -> Int x + y
safe_div = (a: Int b: Int) -> Int ^ Str { ... }

make_adder = (n) (x) x + n    -- closure
add3 = add 3                   -- auto-curry

greet "alice" greeting: "hi"   -- named arg
```

## Sections (Partial Application)

```lx
(+ 1)      -- (x) x + 1
(* 2)      -- (x) x * 2
(> 0)      -- (x) x > 0
(.name)    -- (x) x.name
(10 -)     -- (x) 10 - x
(?? 0)     -- (x) x ?? 0
```

## Pipes

```lx
xs | map (* 2) | filter (> 0) | sum
url | fetch ^ | (.body) | json.parse ^
```

`|` passes left value as **last** argument to right function.

## Pattern Matching

```lx
x ? {
  0 -> "zero"
  1 | 2 -> "small"
  n & (n > 100) -> "big"
  _ -> "other"
}

x > 0 ? "positive" : "non-positive"    -- ternary
x > 0 ? do_thing x                     -- single-arm (unit if false)

result ? { Ok v -> v  Err e -> handle e }
shape ? { Circle r -> r * r  Rect w h -> w * h  Dot -> 0 }
[first ..rest] ? { [x] -> "one"  [x ..] -> "many"  [] -> "none" }
{name: "alice" ..} -> "found alice"     -- record pattern with rest
```

## Error Handling

```lx
Ok 42              -- Result success
Err "failed"       -- Result error
Some "hi"          -- Maybe value
None               -- Maybe empty

value = Ok 42 ^             -- unwrap or propagate
fallback = read "f" ?? ""   -- coalesce on error
require "needed" maybe_val  -- Maybe -> Result

ok? result    -- Bool predicates
err? result
some? maybe
```

## Control Flow

```lx
loop {
  condition ? break value
}

1..10 | each (n) { process n }
1..=10 | step 2 | collect

assert (x == 42)             -- panics if false (test mode: caught per-test)
assert (x == 42) "message"   -- with optional message
```

## Concurrency

```lx
(a b c) = par {       -- parallel, all run, returns tuple
  fetch url1 ^
  fetch url2 ^
  fetch url3 ^
}

winner = sel {         -- race, first wins
  fetch url   -> it    -- `it` = result of completed arm
  timeout 5   -> Err "slow"
}

xs | pmap fetch        -- parallel map
xs | pmap_n 10 fetch   -- rate-limited parallel map
```

**Restrictions:** Capturing mutable bindings (`:=`) in `par`/`sel`/`pmap` bodies is a compile error. Mutables defined inside the body are fine.

**Cancellation:** First error with `^` cancels siblings. Shell commands get SIGTERM, HTTP requests aborted, nested par/sel recursively cancelled.

## Type Definitions

```lx
Point = {x: Float  y: Float}
Shape = | Circle Float | Rect Float Float | Dot
Tree a = | Leaf a | Node (Tree a) (Tree a)
Pair a b = {fst: a  snd: b}
```

## Type Annotations

```lx
x: Int = 42
sum_list = (xs: [Int]) xs | fold 0 (+)
get_name = (r: {name: Str}) r.name
apply_fn = (f: (Int -> Int)  x: Int) f x
safe_div = (a: Int b: Int) -> Int ^ Str { ... }
maybe_val: Maybe Int = Some 42
mapping: %{Str: Int} = %{"a": 1}
```

Validated by `lx check`, ignored by `lx run`.

## Protocols

```lx
Protocol Greeting = {name: Str  message: Str}
Protocol Config = {host: Str  port: Int  debug: Bool = false}

Protocol PerfFinding = {..Base  location: Str}           -- composition
Protocol AgentMsg = ReviewRequest | AuditRequest          -- union (auto-injects _variant)
Protocol Score = {
  value: Float where value >= 0.0 && value <= 1.0         -- field constraint
}
```

Types: `Str`, `Int`, `Float`, `Bool`, `List`, `Record`, `Map`, `Tuple`, `Any`.

## MCP Declarations

```lx
MCP Tools = {
  read_file { path: Str } -> { content: Str }
  list_dir { path: Str } -> [{ name: Str  kind: Str }]
}
```

Typed tool contracts with input/output validation and wrapper generation.

## Traits

```lx
Trait Reviewer = {
  handles: [ReviewRequest AuditRequest]
  provides: [summarize_findings]
  requires: [:ai :fs]               -- :name syntax only in Trait requires lists
}

agent.implements reviewer Reviewer    -- runtime check
```

Agents declare trait membership via `__traits` field: `{name: "reviewer" __traits: ["Reviewer"] handler: ...}`.
Trait-based filtering: `agents | filter (a) agent.implements a Reviewer`.

## Modules

```lx
use std/json                   -- whole module
use std/json : j               -- aliased (j.parse, j.encode)
use std/json {parse encode}    -- selective (direct names)
use ./util                     -- relative import
use ../shared/types            -- parent-relative import

+exported_fn = (x) x * 2      -- + prefix = exported
private_fn = (x) x + 1        -- no prefix = private
```

Importing a module with tagged unions brings constructors into scope. On conflict, use qualified: `module.Constructor`.

## Shell Integration

```lx
r = $echo "hello {name}"      -- Result {out: Str  err: Str  code: Int} ShellErr
s = $^pwd | trim               -- direct Str (exit 0) or propagate error
block = ${                     -- multi-line session (commands share state)
  cd /tmp
  pwd
}
```

`|` inside `$` line is a **shell pipe**. To transition to lx pipe, use `$^` or wrap: `($ls src) ^ | (.out) | split "\n"`.

## Agents & Messaging

```lx
echo = {handler: (msg) msg}
result = echo ~>? {task: "review"} ^   -- ask (request-response)
echo ~> {task: "notify"}               -- send (fire-and-forget)

worker = agent.spawn {command: "lx" args: ["run" "worker.lx"]} ^
response = worker ~>? {task: "analyze"} ^
agent.name worker                      -- get agent name
agent.status worker                    -- query agent state
agent.kill worker
```

## Agent Communication Extensions

```lx
use std/agent

-- Multi-turn dialogue
session = agent.dialogue worker {role: "reviewer"  context: "..."  max_turns: 10} ^
r1 = agent.dialogue_turn session "check auth" ^
history = agent.dialogue_history session ^
agent.dialogue_end session

-- Message middleware (composable, outside-in)
traced = agent.intercept worker (msg next) {
  log.debug "msg: {msg | to_str}"
  next msg
}

-- Pattern-based routing
dispatcher = agent.dispatch [
  {match: {domain: "security"} to: sec_agent}
  {match: (msg) msg.priority == "critical" to: fast_agent}
  {match: "default" to: general_agent}
]
agent.dispatch_multi dispatcher msg ^   -- fan-out to ALL matching
agent.dispatch_add dispatcher {match: ... to: ...} ^
agent.dispatch_remove dispatcher "domain" ^
agent.dispatch_rules dispatcher ^       -- inspect routing table

-- Multi-agent reconciliation
decision = agent.reconcile results {
  strategy: "vote"  quorum: "majority"  deliberate: 2
}
-- Strategies: "union", "intersection", "vote", "highest_confidence", "max_score", "merge_fields", Custom Fn

-- Supervision (Erlang-style)
sup = agent.supervise {
  strategy: "one_for_one"  max_restarts: 5  window: 60
  children: [
    {id: "worker" spawn: () agent.spawn {...} ^ restart: "permanent"}
  ]
}
-- Strategies: "one_for_one", "one_for_all", "rest_for_one"
-- Restart: "permanent", "transient", "temporary"
child = agent.child sup "worker"
agent.supervise_stop sup

-- Capability discovery
caps = agent.capabilities worker ^
agent.advertise {protocols: [...] domains: [...] tools: [...]}

-- Human-in-the-loop gate
gate = agent.gate "deploy" {show: {diff: changes} timeout: 300 on_timeout: "abort"}
-- on_timeout: "abort", "approve", "reject", "escalate"

-- Structured handoff
use std/agent {Handoff}
context_str = agent.as_context handoff   -- Handoff -> markdown string for prompts

-- Multi-agent negotiation
result = agent.negotiate agents {
  topic: "approach"  max_rounds: 5  strategy: "convergence"
}

-- Pub/sub messaging
t = agent.topic "updates"
agent.subscribe t worker
agent.subscribe_filtered t worker (msg) msg.priority == "critical"
agent.publish t {kind: "status" data: "running"}
responses = agent.publish_collect t msg ^   -- publish and collect all responses
agent.unsubscribe t worker
subs = agent.subscribers t
all_topics = agent.topics ()
```

## Mock Agents (Testing)

```lx
mock = agent.mock [
  {match: {task: "review"} respond: {approved: true}}
  {match: (msg) msg.priority == "critical" respond: {fast: true}}
  {match: "any" respond: {error: "unexpected"}}
]
calls = agent.mock_calls mock ^
agent.mock_assert_called mock {task: "review"} ^
agent.mock_assert_not_called mock {task: "delete"} ^
```

## Scoped Resources

```lx
-- Auto-cleanup (LIFO close order, cleanup on error)
with mcp.connect {command: "npx" args: ["server"]} ^ as conn {
  tools = mcp.list_tools conn ^
  mcp.call conn "read_file" {path: "src/main.rs"} ^
}

-- Multiple resources
with resource1 as r1, resource2 as r2 {
  use_both r1 r2
}

-- Scoped bindings
with x = 10, y = 20 {
  x + y
}

-- Mutable scoped binding
with mut counter = 0 {
  counter <- counter + 1
}
```

## Yield & Emit

```lx
yield {kind: "approval" data: changes}    -- pause, get orchestrator response
emit "Status update for humans"           -- fire-and-forget output (strings -> stdout)
emit {progress: 50 stage: "analyzing"}    -- structured emit (records -> JSON)
```

`yield` uses JSON-line orchestrator protocol. `emit` uses `EmitBackend`.

## Refine (Iterative Improvement)

```lx
result = refine initial_work {
  grade: (work) {score: evaluate work  feedback: "..."}
  revise: (work feedback) improve work feedback
  threshold: 85
  max_rounds: 5
  on_round: (round work score) log.info "round {round}: {score}"
}
-- Returns Ok {work rounds final_score} or Err {work rounds final_score reason: "max_rounds"}
```

## Standard Library Modules

### Data Processing

| Module     | Key Functions                                                                                                        |
| ---------- | -------------------------------------------------------------------------------------------------------------------- |
| `std/json` | `parse`, `encode`, `encode_pretty`                                                                                   |
| `std/md`   | `parse`, `sections`, `code_blocks`, `headings`, `links`, `to_text`, `render`, `doc`, `h1`-`h3`, `para`, `code`, `list`, `ordered`, `table`, `link`, `blockquote`, `hr`, `raw` |
| `std/re`   | `is_match`, `match`, `find_all`, `replace`, `replace_all`, `split`                                                   |
| `std/math` | `abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`, `min`, `max`, `pi`, `e`, `inf`                                       |
| `std/time` | `now`, `sleep`, `format`, `parse`                                                                                    |

### System

| Module     | Key Functions                                                        |
| ---------- | -------------------------------------------------------------------- |
| `std/fs`   | `read`, `write`, `append`, `exists`, `stat`, `mkdir`, `ls`, `remove` |
| `std/env`  | `get`, `vars`, `args`, `cwd`, `home`                                 |
| `std/http` | `get`, `post`, `put`, `delete`                                       |

### AI & MCP

| Module    | Key Functions                                                                                             |
| --------- | --------------------------------------------------------------------------------------------------------- |
| `std/ai`  | `prompt`, `prompt_with`, `prompt_structured`, `prompt_structured_with`                                    |
| `std/mcp` | `connect`, `close`, `list_tools`, `call`, `list_resources`, `read_resource`, `list_prompts`, `get_prompt` |

`ai.prompt_structured` validates LLM output against a Protocol schema with auto-retry on schema violation.

### Agent Infrastructure

| Module          | Key Functions                                                                                                                                                                                                                                                          |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `std/agent`     | `spawn`, `ask`, `send`, `kill`, `name`, `status`, `dialogue`, `dialogue_turn`, `dialogue_history`, `dialogue_end`, `intercept`, `dispatch`, `dispatch_multi`, `dispatch_add`, `dispatch_remove`, `dispatch_rules`, `reconcile`, `supervise`, `child`, `supervise_stop`, `mock`, `mock_calls`, `mock_assert_called`, `mock_assert_not_called`, `gate`, `capabilities`, `advertise`, `as_context`, `implements`, `negotiate`, `topic`, `subscribe`, `subscribe_filtered`, `unsubscribe`, `publish`, `publish_collect`, `subscribers`, `topics` |
| `std/ctx`       | `empty`, `set`, `get`, `remove`, `merge`, `keys`, `save`, `load`                                                                                                                                                                                                      |
| `std/tasks`     | `empty`, `create`, `get`, `list`, `children`, `update`, `start`, `submit`, `audit`, `pass`, `fail`, `revise`, `complete`, `save`, `load`                                                                                                                              |
| `std/memory`    | `create`, `store`, `recall`, `tier`, `all`, `promote`, `demote`, `forget`, `consolidate`                                                                                                                                                                               |
| `std/knowledge` | `create`, `store`, `get`, `keys`, `query`, `remove`, `merge`, `expire`                                                                                                                                                                                                |

Protocols exposed by `std/agent`: `Handoff`, `Capabilities`, `GateResult` (import selectively: `use std/agent {Handoff}`).

### Orchestration

| Module           | Key Functions                                                                                                         |
| ---------------- | --------------------------------------------------------------------------------------------------------------------- |
| `std/plan`       | `run`, `continue`, `abort`, `replan`, `insert_after`, `skip`                                                          |
| `std/saga`       | `run`, `define`, `execute`, `run_with`                                                                                |
| `std/audit`      | `is_empty`, `is_too_short`, `is_repetitive`, `is_hedging`, `is_refusal`, `references_task`, `files_exist`, `has_diff`, `rubric`, `evaluate`, `quick_check` |
| `std/circuit`    | `create`, `status`, `tick`, `record`, `check`, `is_tripped`, `reset`                                                  |
| `std/trace`      | `create`, `record`, `score`, `spans`, `summary`, `filter`, `export`, `improvement_rate`, `should_stop`                |
| `std/introspect` | `self`, `elapsed`, `turn_count`, `tick_turn`, `budget`, `actions`, `actions_since`, `mark`, `record`, `is_stuck`, `strategy_shift`, `similar_actions` |
| `std/cron`       | `every`, `after`, `at`, `schedule`, `cancel`, `active`, `list`, `next`, `next_n`, `run`                               |

### Cost, Prompts & Pools

| Module        | Key Functions                                                                                                            |
| ------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `std/budget`  | `create`, `spend`, `remaining`, `used`, `used_pct`, `project`, `status`, `slice`                                         |
| `std/pool`    | `create`, `fan_out`, `map`, `submit`, `status`, `shutdown`                                                               |
| `std/prompt`  | `create`, `system`, `section`, `constraint`, `instruction`, `example`, `compose`, `render`, `render_within`, `estimate`, `sections`, `without` |
| `std/context` | `create`, `add`, `usage`, `pressure`, `estimate`, `pin`, `unpin`, `evict`, `evict_until`, `items`, `get`, `remove`, `clear` |

### Standard Agents

| Module                | Key Functions            |
| --------------------- | ------------------------ |
| `std/agents/auditor`  | `quick_audit`, `audit`   |
| `std/agents/router`   | `quick_route`, `route`   |
| `std/agents/grader`   | `quick_grade`, `grade`   |
| `std/agents/planner`  | `quick_plan`, `plan`     |
| `std/agents/reviewer` | `quick_review`, `review` |
| `std/agents/monitor`  | `scan_actions`, `check`  |

### Visualization

| Module     | Key Functions                           |
| ---------- | --------------------------------------- |
| `std/diag` | `extract`, `extract_file`, `to_mermaid` |

## Built-in Functions (No Import)

**Collection Transform:** `map`, `flat_map`, `scan`, `fold`, `sum`, `product`
**Collection Filter:** `filter`, `take`, `drop`, `take_while`, `drop_while`
**Collection Search:** `find`, `find_index`, `first`, `last`, `get`
**Collection Predicates:** `any?`, `all?`, `none?`, `count`, `empty?`, `contains?`, `has_key?`, `sorted?`
**Collection Sort:** `sort`, `sort_by`, `rev`, `min`, `max`, `min_by`, `max_by`
**Collection Reshape:** `chunks`, `windows`, `partition`, `group_by`, `zip`, `enumerate`
**Collection Flatten:** `flatten`, `intersperse`, `uniq`
**Collection Convert:** `to_list`, `to_map`, `to_record`, `keys`, `values`, `entries`, `merge`, `remove`
**String:** `len`, `byte_len`, `chars`, `lines`, `split`, `join`, `trim`, `trim_start`, `trim_end`, `upper`, `lower`, `replace`, `replace_all`, `starts?`, `ends?`, `pad_left`, `pad_right`, `repeat`
**Numeric:** `even?`, `odd?`, `parse_int`, `parse_float`, `to_int`, `to_float`
**Type:** `type_of`, `to_str`, `ok?`, `err?`, `some?`
**Side Effects:** `each` (returns unit), `tap` (returns original), `dbg` (debug print), `print` (stdout)
**Control:** `identity`, `not`, `require`, `timeout`, `step`, `collect`
**Logging:** `log.info`, `log.warn`, `log.err`, `log.debug`
**Concurrency:** `pmap`, `pmap_n`

## Runtime Semantics

**Number widening:** Int -> Float is automatic in arithmetic. Float -> Int requires `floor`/`ceil`/`round`.
**Equality:** Structural and order-independent for records. Functions are not comparable (runtime error).
**No truthiness:** Ternary `?` requires `Bool`. `0 ? ...` and `None ? ...` are type errors. Multi-arm `?` accepts any type.
**Closures:** Capture lexical scope by reference. Mutable captures shared across closures.
**Tail-call optimization:** In function body, `?{}` arm bodies, ternary branches, last expr in block. NOT in function args, left of pipe, inside `^`, inside `par`/`sel`/`pmap`.
**Forward references:** Top-level mutual recursion works (any order). Within blocks: sequential only.
**Shadowing:** Allowed. Closures that captured original keep it.
**Tuple auto-spread:** N-param function + single N-tuple argument = auto-spread.
**Block evaluation:** Returns last expression value. Intermediate `Err` in function bodies returns immediately.

## CLI Subcommands

| Command              | Purpose                              |
| -------------------- | ------------------------------------ |
| `lx run file.lx`     | Execute a program                    |
| `lx test`            | Run all test suites                  |
| `lx check`           | Type check (bidirectional inference) |
| `lx agent`           | Run as subprocess agent              |
| `lx diagram file.lx` | Generate Mermaid diagram             |

## RuntimeCtx Backends

All I/O builtins receive `&Arc<RuntimeCtx>`. Embedders swap backends for testing/deployment/sandboxing:

| Backend         | Default Implementation       | Purpose                    |
| --------------- | ---------------------------- | -------------------------- |
| `AiBackend`     | `ClaudeCodeAiBackend`        | LLM calls                 |
| `EmitBackend`   | `StdoutEmitBackend`          | Agent-to-human output      |
| `HttpBackend`   | `ReqwestHttpBackend`         | HTTP requests              |
| `ShellBackend`  | `ProcessShellBackend`        | Shell command execution    |
| `YieldBackend`  | `StdinStdoutYieldBackend`    | Coroutine orchestration    |
| `LogBackend`    | `StderrLogBackend`           | Logging                    |

## Design Principles

1. **Errors are values** — `Result`/`Maybe`, no exceptions
2. **No truthiness** — ternary `?` requires explicit `Bool`
3. **Data-last arguments** — enables piping: `xs | map f`
4. **Auto-currying** — all-positional functions
5. **Structural subtyping** — records with extra fields match
6. **Pipe-centric composition** — `|` is the primary composition tool
7. **Structured concurrency** — only `par`/`sel`/`pmap`, no unstructured spawn
8. **No bitwise operators** — `|` `&` `^` reserved for pipes/guards/errors
9. **Eager evaluation** — ranges produce lists, pipelines operate eagerly
10. **Nominal tagged unions** — variant tags distinguish types
