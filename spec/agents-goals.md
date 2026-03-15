# Goal-Level vs Task-Level Communication

Standard protocols for distinguishing between goals (intent — agent decides how) and tasks (instructions — agent executes directly). Enables agents to autonomously decompose goals while faithfully executing tasks.

## Problem

Currently all agent messages are structural — `Protocol` validates shape but doesn't distinguish intent level. An agent receiving a message must inspect content to decide whether to plan or execute:

```
handler = (msg) {
  msg.type ? {
    "search"  -> do_search msg.query    // task: explicit instruction
    "analyze" -> {                       // goal: requires decomposition
      plan = figure_out_how msg
      execute plan
    }
  }
}
```

This means every agent hand-rolls goal detection. There's no standard way for a sender to signal "I'm telling you WHAT to achieve, not HOW to do it" vs "I'm telling you exactly what to do."

## Standard Protocols

Two new protocols in `std/agent`, establishing a convention:

```
Protocol Goal = {
  type: Str = "goal"
  intent: Str
  constraints: Any = {}
  budget: Int = -1
  acceptance: Any = {}
}

Protocol Task = {
  type: Str = "task"
  action: Str
  params: Any = {}
}
```

### Goal Fields

| Field | Description |
|-------|-------------|
| `intent` | Natural language description of desired outcome. |
| `constraints` | Bounds: time, budget, scope, excluded approaches. |
| `budget` | Max budget units for the agent to spend achieving this. -1 = unlimited. |
| `acceptance` | How the sender will evaluate success. Can be a rubric, threshold, or description. |

### Task Fields

| Field | Description |
|-------|-------------|
| `action` | Specific action to perform. |
| `params` | Parameters for the action. |

## `agent.send_goal` / `agent.send_task`

Convenience functions that validate against the protocol and set the `type` field:

```
use std/agent

// Send a goal — agent decides how
result = agent.send_goal worker {
  intent: "find and fix all SQL injection vulnerabilities"
  constraints: {scope: "src/api/" exclude: ["test files"]}
  budget: 500
  acceptance: {min_score: 90 rubric: security_rubric}
} ^

// Send a task — agent executes directly
result = agent.send_task worker {
  action: "grep"
  params: {pattern: "execute\\(" path: "src/api/"}
} ^
```

These are thin wrappers around `~>?` that add Protocol validation.

## Handler Pattern

```
handler = (msg) -> msg.type ? {
  "goal" -> {
    plan = plan.create (decompose msg.intent msg.constraints) ^
    plan.run plan {
      on_step: (step ctx) {
        result = execute_step step ctx ^
        plan.continue {..ctx (step.id): result}
      }
    } ^
  }
  "task" -> execute_action msg.action msg.params ^
  _ -> Err "unknown message type: {msg.type}"
}
```

The key insight: goal handlers use `std/plan` for decomposition, task handlers execute directly. This integrates naturally with the existing planning infrastructure.

## Goal Decomposition

When an agent receives a Goal, it decomposes using `std/agents/planner` (or its own logic):

```
use std/agents/planner

handler = (msg) -> msg.type ? {
  "goal" -> {
    steps = planner ~>? {
      intent: msg.intent
      constraints: msg.constraints
      my_capabilities: introspect.capabilities ()
    } ^
    plan.run steps {on_step: execute_step} ^
  }
  "task" -> execute_action msg ^
}
```

### Goal Forwarding

An agent that can't handle a goal can forward it:

```
handler = (msg) -> msg.type ? {
  "goal" -> {
    can_handle = assess_capability msg.intent
    can_handle ? {
      true  -> decompose_and_execute msg
      false -> {
        better = find_capable_agent msg.intent ^
        agent.send_goal better msg ^
      }
    }
  }
}
```

### Sub-Goals

Goal decomposition can produce sub-goals, not just tasks:

```
decompose = (intent constraints) {
  [
    {id: "understand" type: "goal" intent: "understand the codebase structure"}
    {id: "find"       type: "task" action: "grep" params: {pattern: "sql"}}
    {id: "fix"        type: "goal" intent: "fix identified vulnerabilities"
                      depends: ["understand" "find"]}
  ]
}
```

This creates a tree: top-level goal → mix of sub-goals and tasks → leaf tasks.

## Acceptance Verification

When a goal includes `acceptance`, the agent self-evaluates before returning:

```
execute_goal = (goal) {
  result = do_work goal ^
  goal.acceptance | empty? ? {
    true  -> result
    false -> {
      grade = evaluate result goal.acceptance ^
      grade.passed ? {
        true  -> result
        false -> refine result {grade: (w) evaluate w goal.acceptance  revise: improve  threshold: 85  max_rounds: 3} ^
      }
    }
  }
}
```

## Implementation

Two new Protocol definitions added to `std/agent` in `stdlib/agent.rs`. `agent.send_goal` and `agent.send_task` are thin wrappers around `~>?` with Protocol validation. No new syntax — these are conventions enforced by the type system.

## Cross-References

- Plan execution: [agents-plans.md](agents-plans.md) (goals decompose into plans)
- Planner agent: stdlib_roadmap (`std/agents/planner` — LLM-based decomposition)
- Capability discovery: [agents-capability.md](agents-capability.md) (can this agent handle my goal?)
- Refinement: [agents-refine.md](agents-refine.md) (acceptance verification uses refine)
- Handoff: [agents-handoff.md](agents-handoff.md) (goal forwarding is a form of handoff)
