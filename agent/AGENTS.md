-- Memory: ISA manual (extensions). Agent system — protocols, traits, agents, messaging.
-- Update when agent features change. See also LANGUAGE.md and STDLIB.md.

# lx Agent System

## Traits (Message Contracts)

Traits are `Value::Trait` with non-empty `fields`. Callable as constructor, runtime validation. Display as `<Trait X>`. Returns Err on validation failure (catchable with `??`).

```lx
Trait ReviewRequest = {file: Str  depth: Str = "standard"}
Trait ReviewResult = {approved: Bool  findings: [Str]}
Trait AgentMsg = ReviewRequest | ReviewResult   -- union (auto-injects _variant)
Trait Score = {
  value: Float where value >= 0.0 && value <= 1.0  -- field constraint
}
```

Composition: `Trait Extended = {..Base  extra: Str}`.

## Traits (Behavioral Contracts)

Typed method signatures using MCP syntax (`{input} -> output`), plus default method implementations. Behavioral Traits have empty `fields` and display as `<Trait X>`. Traits with non-empty `fields` are Traits (see above).

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

Methods can reference named Traits as input: `review: ReviewRequest -> {findings: List}`.
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

Subprocess wire protocol: JSON-lines with `{"type":"stream","value":...}`, `{"type":"stream_end"}`, `{"type":"stream_error","error":"msg"}`.

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

## Ambient Context (`with context`)

Scoped ambient state that flows through call chains without explicit parameter threading:

```lx
with context deadline: 30 budget: 500 request_id: "abc-123" {
  dl = context.deadline         -- 30
  all = context.current ()      -- {deadline: 30 budget: 500 request_id: "abc-123"}
  found = context.get "budget"  -- Some 500
  missing = context.get "nope"  -- None
}
```

Nesting merges with outer context — inner values override, outer restored on exit:
```lx
with context budget: 1000 deadline: 60 {
  with context budget: 200 {
    context.budget     -- 200 (overridden)
    context.deadline   -- 60 (inherited)
  }
  context.budget       -- 1000 (restored)
}
```

Empty context clears ambient:
```lx
with context {
  context.current ()   -- {} (empty)
}
```

Context is visible in called functions (flows through the thread-local ambient snapshot):
```lx
read_ctx = () { context.current () }
with context env: "staging" {
  read_ctx ()          -- {env: "staging"}
}
```

Outside any `with context` block, `context.current ()` returns `{}` and `context.get key` returns `None`.

## MCP Declarations (Tool Contracts)

```lx
MCP Tools = {
  read_file { path: Str } -> { content: Str }
  list_dir { path: Str } -> [{ name: Str  kind: Str }]
}
```

## Yield, Emit, Receive

`yield {kind: "approval" data: changes}` — pause for orchestrator input. `emit "Status update"` — fire-and-forget to human (strings → stdout, records → JSON). `receive { action -> (msg) handler }` — agent message loop sugar (desugars to yield/loop/match on `msg.action`).

### Typed Yield Variants (std/yield)

5 Traits for structured orchestrator communication:

```lx
use std/yield {YieldApproval YieldReflection YieldInformation YieldDelegation YieldProgress}

plan = yield YieldApproval {
  action: "deploy"
  details: {env: "prod"  risk: "low"}
}

guidance = yield YieldReflection {
  task: "analyze"
  attempt: {score: 15}
  question: "What should I change?"
}

data = yield YieldInformation {query: "API key for service X"}

yield YieldDelegation {task: {name: "review"  files: ["a.rs"]}}

yield YieldProgress {stage: "parsing"  pct: 0.45}
```

Each Trait has a `kind` field with a default ("approval", "reflection", "information", "delegation", "progress"). The orchestrator receives `{"__yield": {"kind": "approval", ...}}` and dispatches on `kind`. Untyped `yield expr` still works — backwards compatible. Response shapes are conventions, not enforced.

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

## Meta (Strategy-Level Iteration)

```lx
result = meta task {
  strategies: ["bottom_up" "top_down" "decompose"]
  attempt: (strategy task) execute_with strategy task
  evaluate: (result strategy) {
    viable: result.score > 30
    quality: result.score
    reason: result.feedback
  }
  select: "sequential"
  on_switch: (from to reason) log.info "switching from {from} to {to}"
}
```

Returns Ok {result strategy attempts} on first viable attempt, or Err {reason attempts best} if all strategies exhausted. Fields: `strategies` (list), `attempt` (curried fn: strategy -> task -> result), `evaluate` (curried fn: result -> strategy -> {viable quality reason}), optional `select` ("sequential" default, "random"), optional `on_switch` (callback: from -> to -> reason -> ()).

`meta` is a contextual keyword — `Ident("meta")` in the lexer, detected by lookahead in the parser. `meta` can still be used as an identifier (variable name, record field, parameter name).

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

### Hot Reload

```lx
worker = {name: "w" handler: old_fn}
worker = agent.reload worker {handler: new_fn} ^
worker = agent.reload worker {
  handler: better_fn
  on_reload: (old_h new_h) { log.info "reloaded" }
} ^
```

`agent.reload` returns `Ok(agent)` with `__handler_id` referencing a global mutable handler store. Subsequent `~>?`/`~>` calls resolve the handler from the store (not the `handler` field), enabling hot-swap without rebinding at every call site. Subprocess agents return `Err "cannot reload subprocess agent"`.

```lx
handler = (msg) {
  result = approach msg
  score = evaluate result
  (score < 0.7) ? {
    true -> agent.evolve {handler: better_approach} ^
    false -> ()
  }
  result
}
worker = agent.reload worker {handler: handler} ^
```

`agent.evolve` is callable from within a handler — sets a thread-local pending flag. The interpreter applies the evolve after the current message completes. Takes effect on the NEXT message.

```lx
worker = agent.update_traits worker {add: ["Reviewer"] remove: ["Basic"]} ^
```

`agent.update_traits` adds/removes traits on agent Records. Interceptors are preserved — the interceptor chain's `next` function dynamically resolves the current handler from the store, so a reload automatically updates what interceptors delegate to.

### Dialogue Persistence

```lx
session = agent.dialogue worker {role: "reviewer" context: "auth module"} ^
agent.dialogue_turn session "check error handling" ^

agent.dialogue_save session "review-auth-2026-03" ^

session = agent.dialogue_load "review-auth-2026-03" worker ^
agent.dialogue_turn session "any final concerns?" ^

saved = agent.dialogue_list () ^
agent.dialogue_delete "review-auth-2026-03" ^
```

`dialogue_save` persists session state (config + turn history) to `.lx/dialogues/{id}.json`. Overwrites if id exists. `dialogue_load` restores and binds to a (possibly different) agent — only conversation state transfers, not process handle. `dialogue_list` returns `[{id role turns created updated context_preview}]`. `dialogue_delete` removes saved file, returns `Err` if not found.

### Lifecycle Hooks

```lx
me = {name: "worker" handler: (msg) msg}
me = agent.on me "startup" () { log.info "started" } ^
me = agent.on me "shutdown" (reason) { flush_state () } ^
me = agent.on me "error" (err) (msg) { log.err "error: {err}" } ^
me = agent.on me "idle" 30 () { memory.compact () } ^
me = agent.on me "message" (msg) { trace.record "incoming" msg } ^
me = agent.on me "signal" (s) { s.action == "stop" ? should_stop <- true } ^

agent.startup me ^
agent.shutdown me "done" ^
agent.signal me {action: "pause"} ^
agent.on_remove me "idle" ^
hooks = agent.idle_hooks me
```

6 events: startup (before first message), shutdown (on kill/exit), error (unhandled handler error, curried `err -> msg`), idle (after N seconds silence), message (pre-handler hook, Err rejects), signal (user interrupt). Multiple hooks per event fire in registration order. `agent.on_remove` clears all hooks for an event. `agent.kill` auto-fires shutdown hooks. Global `HOOKS` DashMap keyed by auto-assigned `__lifecycle_id`.

### Other Extensions

`agent.capabilities worker ^` / `agent.advertise {protocols: [...]}` — capability discovery. `agent.gate "deploy" {show: data}` — human-in-the-loop approval. `agent.as_context handoff` — context transfer. `agent.negotiate agents {topic: ... max_rounds: 5}` — multi-party consensus. `agent.mock [{match: {task: "review"} respond: {approved: true}}]` — mock agents with call tracking.

## AgentErr (Structured Error Variants)

11 tagged error variants: Timeout, RateLimited, BudgetExhausted, ContextOverflow, Incompetent, Upstream, PermissionDenied, TraitViolation, Unavailable, Cancelled, Internal. Import: `use std/agent {Timeout Upstream ...}`. Match: `Err e -> e ? { Timeout info -> ... ; Upstream info -> ... }`
