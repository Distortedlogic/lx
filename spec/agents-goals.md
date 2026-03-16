# Goal-Level vs Task-Level Communication

Standard Protocol definitions for distinguishing between goals (intent — agent decides how) and tasks (instructions — agent executes directly). These are conventions enforced by the type system, not separate API functions.

## Problem

Currently all agent messages are structural — `Protocol` validates shape but doesn't distinguish intent level. An agent receiving a message must inspect content to decide whether to plan or execute:

```
handler = (msg) {
  msg.type ? {
    "search"  -> do_search msg.query
    "analyze" -> {
      plan = figure_out_how msg
      execute plan
    }
  }
}
```

Every agent hand-rolls goal detection.

## Standard Protocols

Two Protocol definitions in `std/agent`, establishing a convention:

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

## Usage

Send goals and tasks using normal `~>?` with Protocol validation:

```
result = worker ~>? Goal {
  intent: "find and fix all SQL injection vulnerabilities"
  constraints: {scope: "src/api/" exclude: ["test files"]}
  budget: 500
  acceptance: {min_score: 90 rubric: security_rubric}
} ^

result = worker ~>? Task {
  action: "grep"
  params: {pattern: "execute\\(" path: "src/api/"}
} ^
```

No wrapper functions needed — `~>?` with a Protocol value provides validation.

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

Goal handlers use `std/plan` for decomposition, task handlers execute directly.

## Goal Decomposition

When an agent receives a Goal, it decomposes using `std/agents/planner`:

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

Two Protocol definitions added to `std/agent` in `stdlib/agent.rs`. No functions — just protocol shapes. The Goal/Task distinction is a convention enforced by Protocol validation on `~>?`.

## Cross-References

- Plan execution: [agents-plans.md](agents-plans.md) (goals decompose into plans)
- Planner agent: stdlib_roadmap (`std/agents/planner` — LLM-based decomposition)
- Capability discovery: [agents-capability.md](agents-capability.md) (can this agent handle my goal?)
- Refinement: [agents-refine.md](agents-refine.md) (acceptance verification uses refine)
- Handoff: [agents-handoff.md](agents-handoff.md) (goal forwarding uses handoff conventions)
