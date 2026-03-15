# Structured Handoff with Context Transfer

When agent A finishes a phase and agent B takes over, there's a context transfer problem. Manually extracting A's results and passing them to B loses metadata: what A tried, what worked, what it was uncertain about. Structured handoff formalizes this pattern.

## Why a Primitive

Without structured handoff, context transfer looks like:

```
result_a = agent_a ~>? {task: "review auth"} ^
agent_b ~>? {task: "implement fix" context: result_a.summary} ^
```

This loses: what agent A tried and rejected, what assumptions it made, what it was uncertain about, what files it read, what it recommends the next agent start with. The receiving agent gets a summary blob, not structured knowledge about the preceding work.

## Handoff Record

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

Agents construct handoff records explicitly — the runtime doesn't auto-populate them (that would require introspection of all agent activity, which is `std/introspect`'s job). The structure ensures a consistent shape that receiving agents can query.

## API

```
agent.handoff from to handoff_record   -- a ^ AgentErr
```

`agent.handoff` sends the handoff record to the receiving agent as a structured context message. The receiving agent gets a message with `{type: "handoff" ...handoff_record}` and can access any field.

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

fix_result = agent.handoff reviewer fixer handoff ^
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

## Implementation Status

Planned. `Handoff` Protocol and `agent.handoff` / `agent.as_context` functions in `std/agent`.

## Cross-References

- Agent communication: [agents.md](agents.md)
- Multi-turn dialogue: [agents-dialogue.md](agents-dialogue.md)
- Introspection (auto-populating handoff): [stdlib-introspect.md](stdlib-introspect.md)
- Planner agent (handoff between plan steps): [standard_agents.md](../design/standard_agents.md)
