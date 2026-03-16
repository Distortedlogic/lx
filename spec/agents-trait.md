# Agent Traits

Behavioral contracts for agents. A Trait declares what Protocol messages an agent handles and what Skills it provides. Agents declare conformance; the runtime validates.

## Problem

`agent.capabilities` (#27) is runtime discovery — "what CAN you do right now?" That's dynamic and useful for load-aware routing. But there's no way to express static contracts — "agents of this kind MUST handle these message types." Without contracts:

- Routers guess via LLM or ad-hoc domain tags
- Mock agents for testing might not match the real agent's interface
- Agent pools can't guarantee all workers are interchangeable
- No compile-time or spawn-time validation that an agent handles the messages it'll receive

## `Trait` Declaration

```
Protocol ReviewRequest = {task: Str  path: Str}
Protocol AuditRequest = {severity: Str  scope: Str}

Trait Reviewer = {
  handles: [ReviewRequest AuditRequest]
  provides: [summarize_findings]
}
```

### Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `handles` | [Protocol] | Yes | Protocol messages this agent must accept |
| `provides` | [Skill] | No | Skills this agent must expose |
| `requires` | [Symbol] | No | Capabilities needed: `:ai`, `:fs`, `:network` |

`handles` is a list of Protocol names (resolved at evaluation time). `provides` is a list of Skill names. Both are validated when an agent declares conformance.

## Agent Conformance

### At Spawn Time

```
reviewer = agent.spawn {
  command: "lx"
  args: ["run" "agents/reviewer.lx"]
  implements: [Reviewer]
} ^
```

The `implements` field is a list of Trait values. At spawn time, the runtime sends a capabilities probe to the agent. The agent's response must include all Protocols listed in `handles` and all Skills listed in `provides`. If validation fails, `agent.spawn` returns `Err`.

### In Handler Definitions

For in-process agents (record-based handlers), validation is structural:

```
reviewer = {
  implements: [Reviewer]
  handler: (msg) {
    msg._variant ? {
      "ReviewRequest" -> do_review msg
      "AuditRequest"  -> do_audit msg
    }
  }
}
```

The runtime checks that the handler can receive all Protocol messages listed in the Trait's `handles` field. For record-based agents, this is a best-effort check at definition time.

## Trait Composition

Traits compose via list concatenation:

```
Trait Auditable = {handles: [AuditRequest]  provides: [audit_report]}
Trait Reviewable = {handles: [ReviewRequest]  provides: [summarize]}

-- Agent implements both
analyzer = agent.spawn {
  command: "lx" args: ["run" "agents/analyzer.lx"]
  implements: [Auditable Reviewable]
} ^
```

The agent must satisfy all Traits. Overlapping Protocols are deduplicated.

## Trait-Based Routing

```
use std/agent

route_by_trait = (task agents trait) {
  capable = agents | filter (a) agent.implements? a trait
  capable | empty? ? {
    true  -> Err "no agent implements {trait.name}"
    false -> (first capable) ~>? task ^
  }
}
```

`agent.implements?` checks if an agent's declared traits include the given Trait. This replaces ad-hoc domain-tag matching in the router.

## Trait-Based Pools

Traits combine naturally with Agent Pools (see [agents-pool.md](agents-pool.md)):

```
pool = pool.create {
  agent: "agents/reviewer.lx"
  size: 3
  trait: Reviewer
}
```

All pool workers must implement the declared Trait. Workers that fail the Trait check are rejected at spawn time.

## Implementation

### Parser

`Trait` is a new keyword (like `Protocol`, `MCP`, `Skill`). Parsed as: `Trait Name = { handles: [...] provides: [...] requires: [...] }`. Returns `Stmt::Trait { name, fields, exported }`.

### AST Node

```
Stmt::Trait {
    name: String,
    handles: Vec<String>,
    provides: Vec<String>,
    requires: Vec<String>,
    exported: bool,
}
```

### Runtime Value

`Value::Trait { name, handles, provides, requires }`. Evaluated at definition time — Protocol/Skill names are resolved in the current environment.

### Validation

`agent.spawn` with `implements` field triggers trait validation:
1. Send `{type: "capabilities"}` to the agent
2. Check response's `protocols` list includes all `handles` Protocols
3. Check response's skills include all `provides` Skills
4. Return `Err` on mismatch with diagnostic listing missing capabilities

### `agent.implements?`

New builtin in `std/agent`: `agent.implements? agent trait -> Bool`. Checks the agent's stored Trait list (from `implements` at spawn time).

## Cross-References

- Protocol system: [agents-protocol.md](agents-protocol.md)
- Protocol unions (Traits reference union Protocols): [agents-protocol-ext.md](agents-protocol-ext.md)
- Capability discovery (runtime complement): [agents-capability.md](agents-capability.md)
- Skill declarations: [agents-skill.md](agents-skill.md)
- Agent pools: [agents-pool.md](agents-pool.md)
