# Protocol Format Negotiation

Runtime format negotiation between agents whose output/input Protocols don't match exactly but are structurally compatible.

## Problem

`Protocol` validates message shape at send/receive boundaries. If Agent A sends `{score: 0.8 label: "good"}` but Agent B expects `{value: Float name: Str}`, Protocol validation fails — even though the data is structurally compatible (same types, different field names).

Today the fix is: write a manual `agent.intercept` middleware that knows both schemas and transforms between them, or agree on one Protocol in advance. Neither scales when composing agents from different authors or when agent capabilities evolve.

What's needed: agents declare what they produce and accept, and the runtime (or a lightweight negotiation step) resolves compatible mappings automatically.

## Design

### Protocol Adapter Functions

```lx
use std/agent

adapter = agent.adapter ReviewOutput AnalysisInput {
  score -> confidence
  label -> category
  details -> context
}

adapted = agent.intercept reviewer adapter
result = adapted ~>? task ^
```

### `agent.adapter` — Static Field Mapping

```lx
agent.adapter SourceProtocol TargetProtocol {
  source_field -> target_field
  source_field -> target_field
  ...
}
```

Returns an intercept-compatible function that transforms outgoing messages from `SourceProtocol` shape to `TargetProtocol` shape. Fields not mentioned pass through unchanged. Missing required fields in target cause a compile-time error (when used with `lx check`) or runtime error.

### `agent.negotiate_format` — Runtime Negotiation

For dynamic scenarios where Protocols aren't known at write time:

```lx
mapping = agent.negotiate_format producer consumer ^
mapping ? {
  Ok adapter -> {
    adapted = agent.intercept producer adapter
    result = adapted ~>? task ^
  }
  Err conflicts -> {
    log.err "incompatible: {conflicts}"
  }
}
```

`agent.negotiate_format` inspects both agents' advertised Protocols (via `agent.capabilities`) and attempts to find a compatible mapping:

1. **Exact match** — same Protocol name → identity adapter
2. **Structural match** — same field types, different names → prompt for mapping or use heuristic (Levenshtein distance on field names + type match)
3. **Subset match** — target is a subset of source fields → projection adapter
4. **Incompatible** — returns `Err` with list of unresolvable fields

### `agent.coerce` — One-Shot Transform

For ad-hoc transformations without persistent adapters:

```lx
transformed = agent.coerce msg TargetProtocol {
  score -> confidence
  label -> category
} ^
```

Transforms a single message record. Validates result against `TargetProtocol`.

### Core Functions

| Function | Signature | Purpose |
|---|---|---|
| `agent.adapter` | `(source: Protocol target: Protocol mapping: Record) -> Fn` | Create reusable field-mapping adapter |
| `agent.negotiate_format` | `(producer: Agent consumer: Agent) -> Result Fn Str` | Auto-discover compatible mapping |
| `agent.coerce` | `(msg: Record target: Protocol mapping: Record) -> Result Record Str` | One-shot message transform |

### Integration with Existing Features

- `agent.intercept` — adapters are intercept middleware. Composable with logging/validation interceptors.
- `agent.capabilities` — `negotiate_format` reads advertised protocols from capabilities.
- `Protocol` composition — adapters work with composed protocols (`{..Base extra: Str}`).
- `Protocol` unions — adapter maps per-variant when source and target are both unions.

## Implementation

Agent extensions (sub-module of `std/agent`). No parser changes. `agent.adapter` builds a closure that renames fields. `agent.negotiate_format` calls `agent.capabilities` on both agents, compares Protocol schemas, and returns an adapter or error.

Approximately 120 lines of Rust.

## Priority

Tier 3. Useful for plug-and-play agent composition but not blocking — manual interceptors work today. Benefits multiply once `std/registry` (cross-process discovery) ships, where you can't pre-coordinate Protocol names.
