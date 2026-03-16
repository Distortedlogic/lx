# Dynamic Plan Revision — Reference

## API

```
use std/plan

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

## Plan State

The `plan_state` argument to `on_step`:

```
{
  completed: [{id: Str  result: a}]
  remaining: [{id: Str  action: Str  depends: [Str]}]
  current: {id: Str  action: Str}
}
```

## Execution

`plan.run` executes steps in topological order (respecting `depends`). For each step:

1. `executor` is called with the step and accumulated context (results of all completed steps)
2. `on_step` is called with the step, its result, and the current plan state
3. Based on the `PlanAction` returned, the runtime continues, revises, or aborts

## Example

```
use std/plan

steps = [
  {id: "gather" action: "collect data" depends: []}
  {id: "analyze" action: "run analysis" depends: ["gather"]}
  {id: "fix" action: "apply fixes" depends: ["analyze"]}
  {id: "verify" action: "run tests" depends: ["fix"]}
]

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
