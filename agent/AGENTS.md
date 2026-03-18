-- Memory: ISA manual (extensions). Agent system — protocols, traits, agents, messaging.
-- Update when agent features change. See also LANGUAGE.md and STDLIB.md.

# lx Agent System

## Protocols (Message Contracts)

Protocols validate message shapes at runtime. Returns Err on validation failure (catchable with `??`).

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

## Agent Declarations

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

## Agent Messaging

```lx
worker = agent.spawn {command: "lx" args: ["run" "worker.lx"]} ^

result = worker ~>? {task: "analyze" file: "main.rs"} ^
worker ~> {status: "done"}

worker ~>? {task: "review"} ^ | (.findings) | filter (.critical)

agent.kill worker
```

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

## Yield and Emit

```lx
yield {kind: "approval" data: changes}   -- pause for orchestrator input
emit "Status update"                      -- fire-and-forget to human (strings → stdout)
emit {progress: 50 stage: "analyzing"}   -- structured emit (records → JSON)
```

## Receive (Agent Message Handler)

`receive` replaces the yield/loop/match boilerplate for agent message handlers:

```lx
receive {
  analyze -> (msg) analyze_fn msg
  compare -> (msg) compare_fn msg
  _ -> (msg) Err "unknown action"
}
```

Desugars to: yield `{kind: "ready"}`, enter loop, dispatch on `msg.action`, yield `{kind: "result" data: result}`, break on None.

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

### Other Extensions

```lx
caps = agent.capabilities worker ^
agent.advertise {protocols: [...] domains: [...] tools: [...]}
gate = agent.gate "deploy" {show: {diff: changes} timeout: 300 on_timeout: "abort"}
use std/agent {Handoff}
context_str = agent.as_context handoff_record
result = agent.negotiate agents {topic: "approach" max_rounds: 5 strategy: "convergence"}
mock = agent.mock [
  {match: {task: "review"} respond: {approved: true}}
  {match: "any" respond: {error: "unexpected"}}
]
agent.mock_assert_called mock {task: "review"} ^
```
