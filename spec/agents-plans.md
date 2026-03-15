# Dynamic Plan Revision

`yield` creates executable plans where an orchestrator fills in holes. `checkpoint`/`rollback` lets you undo. Neither lets you revise the remaining plan forward when intermediate steps reveal that the plan is wrong. `std/plan` provides plan-as-data execution with runtime revision.

## Why Not Just `yield`

`yield` pauses for orchestrator input and resumes. The control flow is baked into the lx program — if step 3 reveals that steps 4-5 are wrong, the program can't restructure itself:

```
plan = () {
  data = yield {action: "gather"}
  analysis = yield {action: "analyze" data}
  yield {action: "report" findings: analysis}
}
```

This plan always runs gather → analyze → report. If "analyze" discovers a critical issue, the plan can't insert "hotfix" before "report" or replace "report" with "alert oncall."

## Plan-as-Data Model

Plans are lists of step records. Steps have dependencies. The runtime executes steps in dependency order, calling a handler after each step to decide whether to continue, revise, or abort.

```
use std/plan

steps = [
  {id: "gather" action: "collect data" depends: []}
  {id: "analyze" action: "run analysis" depends: ["gather"]}
  {id: "fix" action: "apply fixes" depends: ["analyze"]}
  {id: "verify" action: "run tests" depends: ["fix"]}
]
```

## API

```
plan.run steps executor on_step    -- a ^ PlanErr
                                   --   steps: [{id: Str  action: Str  depends: [Str]  ...extra}]
                                   --   executor: (step context) -> a ^ PlanErr
                                   --   on_step: (step result plan_state) -> PlanAction
plan.replan new_steps              -- PlanAction (replace remaining steps)
plan.skip                          -- PlanAction (skip current step's successors)
plan.abort reason                  -- PlanAction (stop plan execution)
plan.continue                      -- PlanAction (proceed normally)
plan.insert_after step_id new_steps -- PlanAction (add steps after a completed step)
```

`PlanAction` is a tagged union that the `on_step` callback returns to control plan execution.

## Execution Model

`plan.run` executes steps in topological order (respecting `depends`). For each step:

1. `executor` is called with the step and accumulated context (results of all completed steps)
2. `on_step` is called with the step, its result, and the current plan state
3. Based on the `PlanAction` returned, the runtime continues, revises, or aborts

```
use std/plan

result = plan.run steps
  (step ctx) {
    step.action ? {
      "collect data" -> gather_data step ctx
      "run analysis" -> analyze step ctx
      "apply fixes"  -> fix_issues step ctx
      "run tests"    -> run_tests step ctx
      _              -> Err "unknown action: {step.action}"
    }
  }
  (step result state) {
    step.id == "analyze" & result.severity == "critical" ? {
      true -> plan.replan [
        {id: "hotfix" action: "apply critical hotfix" depends: ["analyze"]}
        {id: "verify" action: "run critical-path tests only" depends: ["hotfix"]}
        {id: "alert" action: "notify oncall" depends: ["verify"]}
      ]
      false -> plan.continue
    }
  }
^
```

## Plan State

The `plan_state` argument to `on_step` contains:

```
{
  completed: [{id: Str  result: a}]
  remaining: [{id: Str  action: Str  depends: [Str]}]
  current: {id: Str  action: Str}
}
```

This lets the handler reason about what's done, what's left, and decide whether to revise.

## Patterns

### Progressive Refinement

```
steps = [
  {id: "draft" action: "write first draft" depends: []}
  {id: "review" action: "review draft" depends: ["draft"]}
  {id: "revise" action: "revise based on feedback" depends: ["review"]}
]

plan.run steps executor (step result state) {
  step.id == "review" & !result.passed ? {
    true -> plan.insert_after "review" [
      {id: "revise_2" action: "major revision" depends: ["review"]}
      {id: "review_2" action: "re-review" depends: ["revise_2"]}
    ]
    false -> plan.continue
  }
}
```

### With std/agents/planner

The planner agent generates the initial plan. `std/plan` executes it with revision capability:

```
use std/agents/planner
use std/plan

p = planner.spawn ^
initial_steps = p ~>? {task: complex_task_description} ^
plan.run initial_steps.steps executor on_step ^
```

### With Handoff

Each plan step can produce a handoff for the next step:

```
plan.run steps
  (step ctx) {
    prev_handoff = ctx.completed | last | (.result.handoff) ?? Handoff {result: ()}
    execute_with_context step (agent.as_context prev_handoff)
  }
  (step result _) plan.continue
```

## Implementation Status

Planned. `std/plan` module with `plan.run`, `plan.replan`, `plan.continue`, `plan.abort`, `plan.skip`, `plan.insert_after`.

## Cross-References

- Yield (single-point pause/resume): [agents-advanced.md](agents-advanced.md#yield)
- Checkpoint/rollback (undo): [agents-advanced.md](agents-advanced.md#checkpoint-and-rollback)
- Planner agent: [standard_agents.md](../design/standard_agents.md#stdagentsplanner)
- Task state machine: [stdlib_roadmap.md](../design/stdlib_roadmap.md#stdtasks)
- Handoff: [agents-handoff.md](agents-handoff.md)
