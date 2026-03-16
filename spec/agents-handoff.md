# Structured Handoff with Context Transfer

When agent A finishes a phase and agent B takes over, there's a context transfer problem. Manually extracting A's results and passing them to B loses metadata: what A tried, what worked, what it was uncertain about. Structured handoff is a Protocol convention (not a function) that standardizes this.

## Why a Convention

Without structured handoff, context transfer looks like:

```
result_a = agent_a ~>? {task: "review auth"} ^
agent_b ~>? {task: "implement fix" context: result_a.summary} ^
```

This loses: what agent A tried and rejected, what assumptions it made, what it was uncertain about, what files it read, what it recommends the next agent start with. The receiving agent gets a summary blob, not structured knowledge about the preceding work.

## Handoff Protocol

A handoff is a record with a known structure:

```
Protocol Handoff = {
  result: Any
  tried: List = []
  assumptions: List = []
  uncertainties: List = []
  recommendations: List = []
  files_read: List = []
  tools_used: List = []
  duration_ms: Int = 0
}
```

Agents construct handoff records explicitly — the runtime doesn't auto-populate them. The structure ensures a consistent shape that receiving agents can query. Send a handoff using normal `~>?`:

```
fixer ~>? Handoff {
  result: review_result
  tried: ["checked token refresh" "checked session management"]
  assumptions: ["auth module is the only entry point"]
  uncertainties: ["rate limiting may also be affected"]
  recommendations: ["start with src/auth/token.rs:45"]
  files_read: ["src/auth/token.rs" "src/auth/session.rs"]
} ^
```

## `as_context` Helper

`agent.as_context` transforms a handoff record into a prompt-friendly string that an LLM-backed agent can use:

```
agent.as_context handoff   -- Str
```

Output format:

```
## Previous Agent Handoff
**Result:** <result summary>
**Tried:** <bullet list of approaches>
**Assumptions:** <bullet list>
**Uncertainties:** <bullet list>
**Recommendations:** <bullet list>
**Files examined:** <list>
```

## Usage Patterns

### Sequential Pipeline

```
use std/agent

review_result = reviewer ~>? {task: "review auth module"} ^

handoff = Handoff {
  result: review_result
  tried: ["checked token refresh" "checked session management"]
  assumptions: ["auth module is the only entry point"]
  uncertainties: ["rate limiting may also be affected"]
  recommendations: ["start with src/auth/token.rs:45"]
  files_read: ["src/auth/token.rs" "src/auth/session.rs"]
}

fix_result = fixer ~>? handoff ^
```

### With Dialogue

Handoff can initialize a dialogue session's context:

```
session = agent.dialogue fixer {
  role: "implementer"
  context: agent.as_context handoff
} ^
agent.dialogue_turn session "implement the fix based on the review" ^
```

### Chain of Specialists

```
use std/agent

pipeline = [
  {agent: researcher task: "gather requirements"}
  {agent: architect task: "design solution"}
  {agent: implementer task: "write code"}
  {agent: reviewer task: "verify implementation"}
]

run_pipeline = (stages) {
  stages | fold {result: () handoff: Handoff {result: ()}} (acc stage) {
    result = stage.agent ~>? {
      task: stage.task
      context: agent.as_context acc.handoff
    } ^
    {
      result
      handoff: Handoff {
        result
        tried: result.approaches ?? []
        recommendations: result.next_steps ?? []
      }
    }
  }
}
```

### Escalation Pattern

When an agent fails, escalation is a fold over a chain with handoff context:

```
escalation_chain = [junior_reviewer senior_reviewer lead_reviewer]

result = escalation_chain | fold_until (Err {}) (agent, prev) {
  attempt = agent ~>? {task prev_attempts: prev.attempts ?? []} ^
  attempt ? {
    Ok v -> Break v
    Err e -> Continue {attempts: (prev.attempts ?? []) ++ [{agent: agent error: e}]}
  }
}
```

## Implementation Status

Implemented. `crates/lx/src/stdlib/agent_handoff.rs`. Tests: `tests/48_agent_handoff.lx`. `Handoff` Protocol exposed via `use std/agent {Handoff}` (selective import needed since parser doesn't allow uppercase after `.`). `agent.as_context` formats as Markdown with section headers.

## Cross-References

- Agent communication: [agents.md](agents.md)
- Multi-turn dialogue: [agents-dialogue.md](agents-dialogue.md)
- Introspection (auto-populating handoff): [stdlib-introspect.md](stdlib-introspect.md)
