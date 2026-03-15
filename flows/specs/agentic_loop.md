# Agentic Loop

Core ReAct agent loop with doom loop detection and circuit breakers.

## Target Goal

Implement the Thought → Action → Observation → loop-or-terminate cycle that every agent runs. The loop must detect when it's stuck (doom loop), enforce hard limits (circuit breaker), and optionally verify observations before continuing. The plan-and-execute variant decomposes tasks into subtasks, each with its own ReAct loop, and replans when assumptions are invalidated.

## Scenarios

### Scenario 1: Simple Task Completion

Agent receives "fix the typo in README.md". Thinks → reads file → edits file → verifies edit → done. Completes in 3 turns.

**Success:** Task completes in <5 turns. Final verification confirms the fix.

### Scenario 2: Doom Loop — Stuck

Agent receives "fix the flaky test". Reads the test file, runs it, reads it again, runs it again — same 3 actions repeating. Doom loop detector sees last 3 actions are identical.

**Success:** Circuit breaker fires after detecting 3 identical actions. Agent stops with `Err "doom loop detected"` instead of burning 25 turns.

### Scenario 3: Doom Loop — Stagnating

Agent is oscillating: edit file → test fails → revert edit → edit differently → test fails → revert. Actions alternate but make no progress.

**Success:** Detector classifies as "stagnating" (actions repeat with period 2). Agent receives a hint to try a different approach. If still stagnating after hint, circuit breaker fires.

### Scenario 4: Circuit Breaker — Turn Limit

Complex task requires many steps. Agent reaches turn 25 without completing.

**Success:** Hard stop at turn 25. Returns `Err "circuit breaker: max turns"` with the partial work done so far. Does not silently continue.

### Scenario 5: Circuit Breaker — Timeout

Agent spawns a slow shell command that hangs. 300 seconds elapse.

**Success:** Timeout fires. Agent killed. Error propagated to caller.

### Scenario 6: Verification Failure

Agent edits a file, runs tests, tests fail. Verification loop catches this and asks the agent to diagnose.

**Success:** Agent gets one retry opportunity. If verification fails again, the agent reports failure with diagnostic info rather than looping.

### Scenario 7: Plan-and-Execute

Agent receives "refactor the error handling module". Decomposes into 4 subtasks: (1) audit current patterns, (2) design new pattern, (3) apply to each file, (4) run tests. Each subtask runs its own ReAct loop. After subtask 2, the plan is re-evaluated.

**Success:** Each subtask completes independently. Replan gate catches if subtask 1 reveals the scope is larger than expected.
