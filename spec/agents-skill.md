# Skill Declarations

`Skill` is a keyword-level declaration — a self-describing, discoverable, composable unit of capability. It bundles a function with metadata (description, input/output schema, required permissions) so that LLMs and other agents can discover and select skills at runtime.

Completes a trinity of typed declarations: `Protocol` validates messages, `MCP` wraps external tools, `Skill` defines internal capabilities.

## Problem

Functions have no metadata. An LLM planner can't ask "what can this agent do?" and get back descriptions + schemas to reason over. Every planning/routing step requires ad-hoc description strings.

```
router ~>? {task: "security audit" prompt: desc} ^
// router uses LLM to guess which agent handles "security audit"
// no catalog of what agents actually can do
```

MCP tools are self-describing but external only. Internal lx functions are invisible to LLM reasoning.

## `Skill` Declaration

```
Skill summarize = {
  description: "Condense a document into key points"
  input: {text: Str  max_points: Int = 5}
  output: {points: [Str]  confidence: Float}
  requires: [:ai]
  handler: (input) {
    ai.prompt_structured SummaryResult
      "Summarize into {input.max_points} points: {input.text}" ^
  }
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | Str | Yes | Natural-language description for LLM discovery |
| `input` | Record type | Yes | Input schema — validated before handler runs |
| `output` | Record type | Yes | Output schema — validated after handler returns |
| `requires` | [Symbol] | No | Capabilities needed: `:ai`, `:fs`, `:network`, `:shell` |
| `handler` | Fn | Yes | The implementation function |
| `examples` | [{input output}] | No | Example I/O pairs for few-shot LLM context |
| `tags` | [Str] | No | Categorization tags for filtering |

### Validation

Input is validated against the declared schema before the handler runs. Output is validated after. Schema violations return `Err {type: "schema" field: ... expected: ... got: ...}`.

Default values in input schema (like `max_points: Int = 5`) are applied when the field is missing.

### Export

```
+Skill summarize = { ... }
```

Exported skills are available to importing modules.

## `std/skill` — Runtime Registry and Discovery

### Registry

```
use std/skill

registry = skill.registry [summarize search_code analyze_deps]
```

Creates a registry from a list of Skill declarations.

### Discovery

```
available = skill.list registry
// => [
//   {name: "summarize"  description: "Condense..."  input: {text: Str ...}  output: {...}  tags: [...]}
//   {name: "search_code"  description: "Find code..."  ...}
// ]
```

`skill.list` returns metadata only — no handler functions. Safe to send to an LLM for selection.

### Matching

```
best = skill.match registry "I need to find all functions that handle auth" ^
// => {name: "search_code"  score: 0.85  reason: "matches 'find' + 'code'"}
```

`skill.match` uses keyword matching against descriptions and tags. If `std/ai` is available, can optionally use LLM for semantic matching:

```
best = skill.match_semantic registry prompt ^
```

### Execution

```
result = skill.run registry "summarize" {text: doc} ^
```

`skill.run` looks up the skill by name, validates input, calls handler, validates output. Returns `Result output SkillErr`.

### Skill Info

```
info = skill.get registry "summarize" ^
// => {name: "summarize"  description: ...  input: ...  output: ...  requires: ...  tags: ...}
```

### With Planner

```
available = skill.list registry
plan = planner ~>? {
  task: "audit this codebase for security issues"
  available_skills: available
} ^
// planner returns ordered list of skill names to execute

plan.steps | each (step) {
  skill.run registry step.skill step.input ^
}
```

The planner sees skill descriptions and input/output schemas, then generates a plan using skill names. Execution is type-safe because input/output are validated.

### With Router

```
match = skill.match registry incoming_task ^
match.score > 0.7 ? {
  true -> skill.run registry match.name {text: incoming_task} ^
  false -> router ~>? {task: incoming_task} ^  // fallback to LLM routing
}
```

### With Reputation

```
result = skill.run registry skill_name input ^
grade = auditor ~>? {output: result task: skill_name} ^
reputation.record rep {
  agent: skill_name
  task_type: "skill_execution"
  passed: grade.passed
  score: grade.score
} ^
```

### Combining Skills

```
pipeline = skill.compose registry ["fetch_data" "analyze" "summarize"]
result = pipeline {url: "https://..."} ^
```

`skill.compose` chains skills — output of each feeds as input to the next. Type-checked: output schema of step N must be compatible with input schema of step N+1.

## Relationship to MCP

| | Skill | MCP |
|---|---|---|
| Defined in | lx code | External server |
| Discovery | `skill.list` / `skill.match` | MCP `tools/list` |
| Execution | `skill.run` (in-process) | `mcp.call` (RPC) |
| Typed I/O | Protocol-based | JSON Schema |
| Self-describing | Yes (description + schema) | Yes (schema + description) |

A unified view is possible:

```
all_capabilities = skill.list registry ++ mcp.tools client
```

Both return records with `name`, `description`, `input`, `output`. The planner doesn't need to know whether a capability is internal (Skill) or external (MCP).

## Implementation

### Parser

`Skill` is a new keyword, parsed like `Protocol` and `MCP`. The parser validates that required fields (`description`, `input`, `output`, `handler`) are present.

### AST Node

```
SkillDecl {
  name: String
  exported: bool
  description: Expr
  input: Record
  output: Record
  requires: Option<Vec<Expr>>
  handler: Expr
  examples: Option<Vec<Expr>>
  tags: Option<Vec<Expr>>
}
```

### Runtime

A `Skill` declaration evaluates to a `Value::Skill` variant (like `Value::Protocol` and `Value::McpDecl`). The skill holds its metadata and handler function. `std/skill` registry functions operate on these values.

### std/skill Module

New stdlib module following standard pattern. Functions: `registry`, `list`, `match`, `match_semantic`, `run`, `get`, `compose`.

## Cross-References

- Protocol declarations: [agents-protocol.md](agents-protocol.md) (message validation)
- MCP declarations: [agents-advanced.md](agents-advanced.md) (external tool contracts)
- Structured output: [agents-structured-output.md](agents-structured-output.md) (skills use structured output)
- Router agent: ROADMAP (`std/agents/router`) — skill matching as alternative to LLM routing
- Planner agent: ROADMAP (`std/agents/planner`) — plans over available skills
- Reputation: [agents-reputation.md](agents-reputation.md) — skill execution outcomes feed reputation
