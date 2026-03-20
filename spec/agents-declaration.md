# Agent Declarations

`Agent` is a keyword-level declaration — a first-class, named, typed agent definition. It bundles method implementations with trait conformance, MCP bindings, and lifecycle configuration into a single construct that the runtime validates at definition time.

Eliminates the script-with-main boilerplate pattern. Every agent in `flows/agents/` currently repeats: define handlers, build a map, `+main = () { agent.dispatch handlers }`. An Agent declaration replaces all of that with a validated, self-describing definition.

## Problem

Agents have no first-class identity in lx. They're either:

1. **Records with a handler** — `{handler: (msg) ...}`, no contract enforcement, no typing
2. **Scripts with main** — a `.lx` file exporting `+main` that returns a dispatcher, spawned via `agent.spawn {command: "lx" args: ["run" "script.lx"]}`

Problems:
- No definition-time validation that an agent implements its declared traits
- No typed method signatures — handler functions accept and return `any`
- MCP connections are manual ceremony (connect in every handler, close after)
- Every agent file repeats the same dispatch boilerplate
- Agents can't describe themselves for discovery without separate metadata

## `Agent` Declaration

```
use ../mcps/context_engine

Trait Searchable = {
  search: {topic: Str  path: Str = "."} -> {summary: Str  gaps: List}
  retrieve: {query: Str} -> Str
  requires: [:ai]
}

Agent ContextEngine: Searchable = {
  uses: {engine: context_engine}

  search = (msg) {
    engine.search {topic: msg.topic  path: msg.path ?? "."} ^
  }

  retrieve = (msg) {
    engine.retrieve {query: msg.query} ^
  }
}
```

### Syntax

```
Agent Name: TraitList = { body }
Agent Name = { body }
```

`TraitList` is either a single Trait name or a list `[Trait1 Trait2]`. When present, all Trait methods must be implemented in the body. When omitted, the agent is untyped (useful for prototyping).

Export with `+`:

```
+Agent ContextEngine: Searchable = { ... }
```

### Body Fields

#### Methods

Every non-reserved name in the body is a method implementation:

```
search = (msg) { ... }
retrieve = (msg) { ... }
```

Methods receive the incoming message as their argument. Return type is validated against the Trait's declared output type for that method.

#### `uses` — MCP Bindings

```
uses: {engine: context_engine  grit: gritql}
```

Declares MCP dependencies by referencing imported MCP declaration modules. The runtime:
1. Calls `module.connect` for each binding at agent startup
2. Binds the typed MCP client to the declared name in method scope
3. Calls `module.close` for each binding on agent shutdown or kill

Methods access MCP tools through the bound names directly — no manual connect/close.

#### `init` — Initial State

```
init: {count: 0  history: []}
```

Optional initial state record. If present, methods receive state as a second argument and return updated state alongside their result:

```
Agent Counter = {
  init: {count: 0}

  increment = (msg state) {
    new_state = {..state count: state.count + 1}
    {result: new_state.count  state: new_state}
  }

  get = (msg state) {
    {result: state.count  state}
  }
}
```

The runtime threads state between method calls. If `init` is absent, methods receive only the message.

#### `on` — Lifecycle Hooks

```
on: {
  startup: () { emit "agent ready" }
  shutdown: (reason) { emit "shutting down: {reason}" }
  idle: {after: 30  run: () { emit "idle housekeeping" }}
  error: (err msg) { emit "error on {msg}: {err}" }
}
```

Inline lifecycle hooks (see `agents-lifecycle.md`). Registered automatically at agent startup.

### Reserved Field Names

`uses`, `init`, `on` are reserved. Everything else is a method.

## Validation

At definition time, the runtime checks:

1. **Method completeness** — every Trait method has a corresponding implementation
2. **No extra methods** for typed agents — prevents silent misspellings (warn, not error)
3. **Input compatibility** — method parameter accepts the Trait's declared input type
4. **Output compatibility** — method return matches the Trait's declared output type
5. **Resource availability** — Trait's `requires` resources exist in `RuntimeCtx`
6. **MCP bindings** — every `uses` entry references a valid MCP module with `connect`/`close`

Validation failure at definition time is a hard error with diagnostics:

```
Agent error: ContextEngine missing method 'retrieve' required by Searchable
Agent error: ContextEngine.search output type mismatch:
  expected: {summary: Str  gaps: List}
  got: Str
```

## Usage Patterns

### Direct Import (In-Process)

```
use ./agents/context_engine

result = context_engine.ContextEngine.search {topic: "auth" path: "src/"} ^
```

When imported, Agent methods are callable as module functions. Input is validated against the Trait signature. The `uses` MCPs are connected on first call and cached.

### Spawn as Subprocess

```
agent = agent.spawn ContextEngine {} ^
result = agent ~>? {_method: "search" topic: "auth"} ^
```

`agent.spawn` accepts an Agent declaration directly (no script path needed). The runtime:
1. Starts a subprocess running the Agent's message loop
2. Validates trait conformance via capabilities probe
3. Connects `uses` MCPs in the subprocess
4. Routes incoming messages to methods by `_method` field or Trait matching

### Spawn by Module Path

```
agent = agent.spawn {
  agent: "agents/context_engine.ContextEngine"
  implements: [Searchable]
} ^
```

For spawning agents defined in other modules. The runtime resolves the module path, loads the Agent declaration, and validates conformance.

### Message Routing

When a message arrives via `~>?`, routing follows this order:

1. **Explicit method** — if `msg._method` is set, route to that method
2. **Trait match** — if the Trait declares a method with a Trait input type, and the message validates against that Trait, route to that method
3. **Single method** — if the Agent has exactly one method, route there
4. **No match** — return `Err {type: "no_route" message: msg}`

### Inline Agent (Anonymous)

For quick one-off agents without a named declaration:

```
echo = Agent: Echoable = {
  echo = (msg) { msg }
}
```

## Interaction with Existing Systems

Agent declarations integrate directly with pools, supervision, registry, and dispatch:

```
pool = pool.create {agent: ContextEngine  size: 3  trait: Searchable}

agent.supervise (agent.spawn ContextEngine {}) {strategy: :one_for_one  max_restarts: 3}

registry.register conn {agent: ContextEngine  domains: ["codebase"]}

dispatcher = agent.dispatch [
  {match: {domain: "code"} to: ContextEngine}
  {match: {domain: "security"} to: SecurityAgent}
]
```

`pool.create` and `registry.register` extract trait and method metadata from the Agent declaration automatically. Trait validation is implicit.

## Implementation

### Parser

`Agent` is a new keyword. Grammar:

```
agent_decl := "Agent" IDENT (":" trait_list)? "=" "{" agent_body "}"
trait_list  := IDENT | "[" IDENT+ "]"
agent_body  := (reserved_field | method_def)*
reserved_field := ("uses" | "init" | "on") ":" expr
method_def := IDENT "=" expr
```

### AST Node

```
AgentDecl {
    name: String,
    traits: Vec<String>,
    uses: Option<Vec<(String, String)>>,
    init: Option<Expr>,
    on: Option<Record>,
    methods: Vec<AgentMethod>,
    exported: bool,
}

AgentMethod {
    name: String,
    handler: Expr,
}
```

### Runtime Value

`Value::Agent` holds:
- Name, trait references, method implementations
- MCP binding specs (module paths)
- Init state template
- Lifecycle hook closures

When used in-process, methods are called directly with input validation. When spawned, the runtime generates a message loop that dispatches to methods.

### Message Loop Generation

For subprocess agents, the runtime generates the equivalent of:

```
loop {
  msg = read_json_line stdin
  method = route msg
  result = method msg
  write_json_line stdout result
}
```

This replaces the manual `+main = () { agent.dispatch handlers }` boilerplate.

### MCP Lifecycle

`uses` bindings are managed by the runtime:
- **In-process**: lazy connect on first method call, close when agent value is dropped
- **Subprocess**: connect at startup (after spawn, before first message), close at shutdown

## Cross-References

- Trait contracts: [agents-trait.md](agents-trait.md) — Traits define what Agents must implement
- MCP declarations: [agents-advanced.md](agents-advanced.md) — `uses` references MCP modules
- Lifecycle hooks: [agents-lifecycle.md](agents-lifecycle.md) — `on` field integrates hooks
- Agent pools: [agents-pool.md](agents-pool.md) — pools accept Agent declarations
- Discovery: [agents-discovery.md](agents-discovery.md) — registry extracts Agent metadata
- Scoped resources: [scoped-resources.md](scoped-resources.md) — `uses` is scoped resource management
- Eliminated: `agents-skill.md` — Skill declarations merged into Trait methods
