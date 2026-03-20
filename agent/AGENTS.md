-- Memory: ISA manual (extensions). Agent system — protocols, traits, agents, messaging.
-- Update when agent features change. See also LANGUAGE.md and STDLIB.md.

# lx Agent System

## Protocols (Message Contracts)

Protocols are `Value::Trait` with non-empty `fields`. Callable as constructor, runtime validation. Display as `<Protocol X>`. Returns Err on validation failure (catchable with `??`).

```lx
Protocol ReviewRequest = {file: Str  depth: Str = "standard"}
Protocol ReviewResult = {approved: Bool  findings: [Str]}
Protocol AgentMsg = ReviewRequest | ReviewResult   -- union (auto-injects _variant)
Protocol Score = {
  value: Float where value >= 0.0 && value <= 1.0  -- field constraint
}
```

Composition: `Protocol Extended = {..Base  extra: Str}`.

## Traits (Behavioral Contracts)

Typed method signatures using MCP syntax (`{input} -> output`), plus default method implementations. Behavioral Traits have empty `fields` and display as `<Trait X>`. Traits with non-empty `fields` are Protocols (see above).

```lx
Trait Reviewer = {
  description: "Code review agent"
  review: {file: Str  depth: Str = "normal"} -> {approved: Bool  findings: List}
  summarize: {findings: List} -> Str
  requires: [:ai :fs]
  tags: ["code" "review"]
  summary = () { "Summary: " ++ (self.describe ()) }
}
```

Methods can reference named Protocols as input: `review: ReviewRequest -> {findings: List}`.
Default methods (with `=`) are auto-injected into conforming Agent/Class if not overridden.

## Classes (Stateful Objects)

Both Class and Agent produce `Value::Class { name, traits, defaults, methods }`. Agent is a Trait defined in `pkg/agent.lx` — the `Agent` keyword auto-imports it and auto-adds "Agent" to the traits list. `Class Worker : [Agent] = { ... }` also works. Shared trait injection via `inject_traits` helper. Object fields backed by STORES (same DashMap as Store values).

```lx
Class Counter : [Checkable] = {
  count: 0
  max: 10
  tick = () { self.count <- self.count + 1 }
  check = () { self.count >= self.max ? (Err "limit") : (Ok ()) }
}

c = Counter {max: 5}
c.tick ()
c.check () ^
```

- Fields use `:` (name: default), methods use `=` (name = closure)
- Constructor: `ClassName {field: val}` or `ClassName ()`. Store fields are cloned per instance.
- `self.field` reads, `self.field <- val` writes (in-place via STORES)
- Reference semantics: `a = b` shares same object. Both see mutations
- Trait conformance + defaults work the same as Agent
- Export with `+Class Name = { ... }`
- Display: checks traits list for "Agent" → `<Agent X>` if present, `<Class X>` otherwise

Discovery via `std/trait`:

```lx
use std/trait
methods = trait.methods Reviewer
best = trait.match Reviewer "find issues"
```

## Agent Declarations

Agent is a Trait defined in `pkg/agent.lx`. The `Agent` keyword auto-imports it and auto-adds "Agent" to the traits list. All agents get the Agent Trait's defaults: init, perceive, reason, act, reflect, handle (auto-dispatch by `msg.action` via `method_of`), run (yield/loop message loop), think/think_with/think_structured (AI), use_tool/tools (tool hooks), describe (self-description via `methods_of`), ask/tell (inter-agent communication). Override only the methods you need:

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
`init`/`on` go into the methods map. `uses` dropped (use `with mcp.connect` instead).
Display: `<Agent CodeReviewer>`.

## Agent Messaging

```lx
worker = agent.spawn {command: "lx" args: ["run" "worker.lx"]} ^

result = worker ~>? {task: "analyze" file: "main.rs"} ^
worker ~> {status: "done"}

worker ~>? {task: "review"} ^ | (.findings) | filter (.critical)

agent.kill worker
```

## Streaming Ask (`~>>?`)

`~>>?` returns a `Stream` — a lazy sequence of values from a long-running agent:

```lx
stream = analyzer ~>>? {task: "review" path: "src/"}

stream | each (item) emit "reviewed: {item.file}"
results = stream | collect
first_five = stream | take 5
critical = stream | filter (.severity == "critical")
sum = stream | fold 0 (acc x) acc + x
```

All HOFs work on streams: `map`, `filter`, `each`, `take`, `fold`, `flat_map`, etc. `collect` materializes the entire stream into a list.

Agent-side (subprocess): use `agent.emit_stream` and `agent.end_stream`:
```lx
use std/agent
items | each (item) {
  result = process item ^
  agent.emit_stream result
}
agent.end_stream ()
```

Subprocess protocol: JSON-lines with `{"type":"stream","value":...}`, `{"type":"stream_end"}`, `{"type":"stream_error","error":"msg"}`.

## Scoped Resources (with ... as)

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

## MCP Declarations (Tool Contracts)

```lx
MCP Tools = {
  read_file { path: Str } -> { content: Str }
  list_dir { path: Str } -> [{ name: Str  kind: Str }]
}
```

## Yield, Emit, Receive

`yield {kind: "approval" data: changes}` — pause for orchestrator input. `emit "Status update"` — fire-and-forget to human (strings → stdout, records → JSON). `receive { action -> (msg) handler }` — agent message loop sugar (desugars to yield/loop/match on `msg.action`).

## Refine (Iterative Improvement)

```lx
result = refine initial_draft {
  grade: (work) {score: evaluate work  feedback: "..."}
  revise: (work feedback) improve work feedback
  threshold: 85
  max_rounds: 5
  on_round: (round work score) log.info "round {round}: {score}"
}
```

Returns Ok {work rounds final_score} or Err {work rounds final_score reason}.

## Agent Communication Extensions

All under `use std/agent`:

### Dialogue (Multi-Turn Sessions)

```lx
session = agent.dialogue worker {role: "reviewer" context: "..." max_turns: 10} ^
r1 = agent.dialogue_turn session "review the auth module" ^
r2 = agent.dialogue_turn session "what about the error handling?" ^
agent.dialogue_end session
```

### Dialogue Branching (Fork/Compare/Merge)

```lx
(fork_a fork_b) = agent.dialogue_fork session ["Try JWT" "Try sessions"] ^
a1 = agent.dialogue_turn fork_a "Implement JWT" ^
b1 = agent.dialogue_turn fork_b "Implement sessions" ^
comparison = agent.dialogue_compare [fork_a fork_b] {
  grade: (s) { h = agent.dialogue_history s; {score: 0.9  summary: "..."} }
} ^
agent.dialogue_merge session comparison.best ^
branches = agent.dialogue_branches session
```

Fork shares parent history. Parent suspended while forks active. Nestable. `dialogue_merge` appends winner's post-fork history, cleans up all forks recursively. `dialogue_compare` returns `{rankings: [{session score summary}] best spread}`.

### Dispatch (Pattern-Based Routing)

```lx
dispatcher = agent.dispatch [
  {match: {domain: "security"} to: sec_agent}
  {match: (msg) msg.priority == "critical" to: fast_agent}
  {match: "default" to: general_agent}
]
dispatcher ~>? msg ^
agent.dispatch_multi dispatcher msg ^
```

### Reconciliation (Merge Parallel Results)

```lx
decision = agent.reconcile results {
  strategy: "vote"  quorum: "majority"  deliberate: 2
}
```

Strategies: "union", "intersection", "vote", "highest_confidence", "max_score", "merge_fields", or custom Fn.

### Supervision (Erlang-Style)

```lx
sup = agent.supervise {
  strategy: "one_for_one"  max_restarts: 5  window: 60
  children: [{id: "worker" spawn: () agent.spawn {...} ^ restart: "permanent"}]
}
```

### Message Middleware

```lx
traced = agent.intercept worker (msg next) {
  log.debug "msg: {msg | to_str}"
  next msg
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

### Capability Routing

```lx
agent.register reviewer {traits: ["Reviewer"] domains: ["code" "security"]} ^
agent.register worker {protocols: ["TaskRequest"] max_concurrent: 5} ^

result = agent.route msg {trait: "Reviewer"} ^
result = agent.route msg {trait: "Reviewer" prefer: "round_robin" fallback: backup} ^
results = agent.route_multi msg {trait: "Reviewer"} ^
reconciled = agent.route_multi msg {
  trait: "Reviewer"
  reconcile: {strategy: "vote" vote_field: "approved"}
} ^

agents = agent.registered {trait: "Reviewer"} ^
agent.unregister reviewer ^
```

Selection: `"least_busy"` (default), `"round_robin"`, `"random"`, or custom `(agents) -> Agent`.

### Pipeline — `agent.pipeline [stages] opts ^`, then `pipeline_send/collect/batch/stats/on_pressure/pause/resume/drain/close/add_worker`. Overflow: block/drop_oldest/drop_newest/sample. Stages: `{name: "x" handler: fn}` or spawned agents.

### Format Negotiation

`agent.adapter SourceProto TargetProto {src_field: "tgt_field"}` — reusable field-mapping function. Unmapped fields pass through. Missing required target fields → `Err` (catchable with `??`). `agent.coerce msg TargetProto {mapping}` — one-shot transform, returns `Ok record` or `Err`. `agent.negotiate_format producer consumer` — inspects capabilities, finds compatible mapping (exact name → identity, structural/Levenshtein → mapping adapter, incompatible → `Err`).

### Other Extensions

`agent.capabilities worker ^` / `agent.advertise {protocols: [...]}` — capability discovery. `agent.gate "deploy" {show: data}` — human-in-the-loop approval. `agent.as_context handoff` — context transfer. `agent.negotiate agents {topic: ... max_rounds: 5}` — multi-party consensus. `agent.mock [{match: {task: "review"} respond: {approved: true}}]` — mock agents with call tracking.

## AgentErr (Structured Error Variants)

11 tagged error variants: Timeout, RateLimited, BudgetExhausted, ContextOverflow, Incompetent, Upstream, PermissionDenied, ProtocolViolation, Unavailable, Cancelled, Internal. Import: `use std/agent {Timeout Upstream ...}`. Match: `Err e -> e ? { Timeout info -> ... ; Upstream info -> ... }`
