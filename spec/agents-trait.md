# Agent Traits

Behavioral contracts with typed method signatures. A Trait declares what an agent must do — typed methods, resource requirements, and optional metadata for LLM discovery. Agents declare conformance; the runtime validates at definition time.

Supersedes the `Skill` declaration. Skills and Traits covered the same ground — self-describing typed capabilities — but Skills were per-function while Traits were unenforceable name lists. This spec merges them: Trait methods ARE skills, with the rigor of interface enforcement.

## Problem

The current Trait system (`handles: [Protocol]  provides: [skill_name]`) is toothless:

- `provides` is a list of strings — no signatures, no enforcement
- `agent.implements` checks `__traits` string tags, not actual capability
- An agent can claim `__traits: ["Reviewer"]` without handling any review messages
- Skills (`spec/agents-skill.md`) added typed I/O and metadata but as a separate system
- Two overlapping concepts (Traits for contracts, Skills for capabilities) that should be one

## `Trait` Declaration

```
Trait Searchable = {
  search: {topic: Str  path: Str = "."} -> {summary: Str  gaps: List}
  retrieve: {query: Str} -> Str
  requires: [:ai]
}
```

Methods use the same signature syntax as MCP tool declarations: `{input fields} -> output_type`. This is the contract — any agent implementing `Searchable` must provide `search` and `retrieve` with these exact signatures.

### Method Signatures

Short form — just the type contract:

```
Trait Reviewable = {
  review: {task: Str  path: Str} -> {findings: List  score: Float}
  summarize: {findings: List} -> Str
}
```

Long form — signature plus metadata for LLM discovery:

```
Trait Reviewable = {
  review: {
    description: "Review code at the given path"
    input: {task: Str  path: Str}
    output: {findings: List  score: Float}
    examples: [{input: {task: "security" path: "src/"} output: {findings: [] score: 1.0}}]
  }
  summarize: {findings: List} -> Str
}
```

The two forms are unambiguous: if the value contains `->`, it's short form. If it's a record with `input` and `output` fields, it's long form.

### Reserved Fields

These names are trait metadata, not methods:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | Str | No | Natural-language description for LLM discovery |
| `requires` | [Symbol] | No | Resource capabilities needed: `:ai`, `:fs`, `:network`, `:shell` |
| `tags` | [Str] | No | Categorization tags for filtering and matching |

Everything else in the trait body is a method declaration.

### Trait Inputs

Method inputs can reference named Traits instead of inline record types:

```
Trait ReviewRequest = {task: Str  path: Str}

Trait Reviewable = {
  review: ReviewRequest -> {findings: List  score: Float}
}
```

This enables automatic message routing: when an agent receives a `ReviewRequest` via `~>?`, the runtime can match it to the `review` method.

## Conformance Checking

### At Agent Definition

When an Agent declaration lists trait conformance (see `agents-declaration.md`), the runtime validates:

1. The agent provides a method for every method in the Trait
2. Each method's input type is compatible with the Trait's declared input
3. Each method's output type is compatible with the Trait's declared output
4. All `requires` resources are available in the agent's runtime context

Validation happens at definition time — not deferred to first call.

### At Spawn Time

```
reviewer = agent.spawn {
  command: "lx"
  args: ["run" "agents/reviewer.lx"]
  implements: [Reviewable]
} ^
```

The `implements` field triggers spawn-time validation. The runtime sends a capabilities probe. The agent's response must include method signatures matching all declared Traits. Mismatch returns `Err` with diagnostics listing missing or incompatible methods.

### `agent.implements?`

Runtime check: `agent.implements? agent trait -> Bool`. Returns `true` only if the agent's methods match the Trait's signatures. Replaces the current `__traits` string-tag check.

## Trait Composition

Agents implement multiple Traits by listing them:

```
Agent Analyzer: [Reviewable Searchable] = { ... }
```

The agent must satisfy all Traits. Overlapping methods with identical signatures are deduplicated. Conflicting signatures (same method name, different types) are a definition-time error.

## Discovery

Traits absorb the `std/skill` discovery functionality. Since Trait methods are typed and optionally annotated with descriptions, they serve as the catalog for LLM-driven routing and planning.

### Listing Methods

```
methods = trait.methods Reviewable
-- [{name: "review"  input: {task: Str  path: Str}  output: {findings: List  score: Float}}
--  {name: "summarize"  input: {findings: List}  output: Str}]
```

Returns method metadata only (no handler functions). Safe to send to an LLM for selection.

### Matching

```
best = trait.match Reviewable "I need to find security issues in auth code" ^
-- {method: "review"  score: 0.85}
```

Keyword matching against method names, descriptions, and tags. With `:ai` available, `trait.match_semantic` uses LLM for semantic matching.

### Unified Capability View

```
internal = trait.methods Reviewable
external = mcp.tools grit_server
all_capabilities = internal ++ external
```

Both return records with `name`, `input`, `output`. A planner doesn't need to distinguish internal methods from external MCP tools.

## Trait-Based Routing

```
route_by_trait = (task agents trait) {
  capable = agents | filter (a) agent.implements? a trait
  capable | empty? ? {
    true  -> Err "no agent implements {trait.name}"
    false -> (first capable) ~>? task ^
  }
}
```

### Message Routing via Trait Matching

When a method's input type is a named Trait, the runtime can auto-route incoming messages:

```
reviewer ~>? ReviewRequest {task: "audit" path: "src/"}
```

The runtime checks which method accepts `ReviewRequest` and dispatches to it. If multiple methods accept the same Trait, the first match wins.

### Direct Method Calls

When an agent is imported as a module, methods are callable directly:

```
use ./agents/reviewer
reviewer.review {task: "audit" path: "src/"} ^
```

No message passing — direct function call with input validation against the Trait signature.

## Trait-Based Pools

```
pool = pool.create {
  agent: "agents/reviewer.lx"
  size: 3
  trait: Reviewable
}
```

All pool workers must implement the declared Trait. Workers that fail the Trait check are rejected at spawn time.

## Implementation

### Parser

`Trait` keyword already exists. Parser changes: method entries use `{input} -> output` or `{description, input, output}` syntax (same as MCP tool declarations). Reserved field names (`description`, `requires`, `tags`) parsed as metadata.

### AST Node

```
TraitDecl {
    name: String,
    methods: Vec<TraitMethod>,
    requires: Vec<Expr>,
    description: Option<Expr>,
    tags: Option<Vec<Expr>>,
    exported: bool,
}

TraitMethod {
    name: String,
    input: McpInputDef,
    output: McpOutputDef,
    description: Option<String>,
    examples: Option<Vec<Expr>>,
}
```

Reuses `McpInputDef` and `McpOutputDef` from MCP declarations — same type representation.

### Runtime Value

`Value::Trait { name, methods, requires, description, tags }`. Methods carry their typed signatures. `trait.methods` extracts them as records.

### Validation

`agent.implements?` performs structural checking: for each Trait method, the agent must have a corresponding method with compatible input/output types. Type compatibility uses the existing subtyping rules from the type checker.

### Migration from Current Traits

Current `handles`/`provides` syntax continues to parse but is deprecated. The `handles` list is derivable from methods with Trait-typed inputs. The `provides` name list is replaced by actual method declarations.

## Cross-References

- Agent declarations: [agents-declaration.md](agents-declaration.md) — Traits are enforced on Agent definitions
- MCP declarations: [agents-advanced.md](agents-advanced.md) — method syntax mirrors MCP tool syntax
- Trait system: [agents-protocol.md](agents-protocol.md) — Trait-typed inputs enable message routing
- Agent pools: [agents-pool.md](agents-pool.md) — Trait-constrained worker pools
- Cross-process discovery: [agents-discovery.md](agents-discovery.md) — registry queries by Trait
- Eliminated: [agents-skill.md](agents-skill.md) — Skill functionality merged into Traits
