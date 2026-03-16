# Structured Handoff — Reference

## Handoff Protocol

```
Protocol Handoff = {
  result: Any  tried: List = []  assumptions: List = []  uncertainties: List = []
  recommendations: List = []  files_read: List = []  tools_used: List = []  duration_ms: Int = 0
}
```

Import via `use std/agent {Handoff}`. Agents construct handoff records explicitly.

## Sending a Handoff

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

Transforms a handoff into a prompt-friendly Markdown string:

```
agent.as_context handoff   -- Str
```

## Pipeline Example

```
use std/agent
review_result = reviewer ~>? {task: "review auth module"} ^
handoff = Handoff {
  result: review_result
  tried: ["checked token refresh"]
  recommendations: ["start with src/auth/token.rs:45"]
  files_read: ["src/auth/token.rs"]
}
fix_result = fixer ~>? handoff ^
```

### With Dialogue

```
session = agent.dialogue fixer {role: "implementer"  context: agent.as_context handoff} ^
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
    result = stage.agent ~>? {task: stage.task  context: agent.as_context acc.handoff} ^
    {result  handoff: Handoff {result  tried: result.approaches ?? []  recommendations: result.next_steps ?? []}}
  }
}
```
