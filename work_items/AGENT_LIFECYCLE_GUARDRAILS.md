# Goal

Implement the missing `refine` function in `std/workflow.lx` that `pkg/agent/quality.lx` already calls. Add composable stop conditions as a new `pkg/guard/conditions.lx` module. Create an agent harness runner in `pkg/agents/harness.lx` that composes budget tracking, stop conditions, and quality gating around any Agent's run loop — living in pkg/ where it can import other pkg/ modules, not in std/ where it cannot.

# Why

- `pkg/agent/quality.lx` line 72 calls `refine` but the function does not exist anywhere in the codebase — not as a Rust builtin, not as an exported lx function. The `refine_work`, `refine_response`, and `refine_code` functions are all broken because they depend on this missing function.
- The CircuitBreaker in `pkg/guard/circuit.lx` checks max_turns/max_actions/max_time/repetition as a config blob, but conditions cannot be composed algebraically. You cannot write "stop after 25 turns OR 60 seconds OR budget exhaustion" as a single composable value.
- Brain's `main.lx` and `orchestrator.lx` manually call `monitor.guard()`, `budget.spend()`, `quality.grade()` at every step. This boilerplate pattern would be repeated by any future agent program. A reusable harness runner in pkg/ eliminates this.

# Why Not Wire Into std/agent.lx

The Agent trait lives in `crates/lx/std/agent.lx`, which is embedded in the Rust binary via `include_str!()` in `crates/lx/src/stdlib/mod.rs`. Stdlib files go through a different module resolution path than normal .lx files — they cannot import pkg/ modules. Adding `use pkg/agent/budget` or `use pkg/guard/conditions` to `std/agent.lx` would fail at runtime because the stdlib loader does not resolve pkg/ paths.

The correct architecture: the Agent trait in std/ stays minimal and dependency-free. The harness composition layer lives in pkg/ where it can freely import budget, conditions, quality, and any other pkg/ module. Simple agents use the harness; complex agents (like brain/) continue to wire their own monitoring.

# What Changes

**1. Implement `refine` in `std/workflow.lx`**

Add an exported `refine` function after the existing `topo_sort` function at line 85. It takes two arguments: `initial` (the starting work value) and `opts` (a record with fields `grade`, `revise`, `threshold`, `max_rounds`). The function loops: call grade on current work, if score meets threshold return Ok, if max rounds reached return Err, otherwise call revise with current work and feedback and continue.

The return value matches the existing call site in `pkg/agent/quality.lx` line 94: `result ? { Ok r -> r.work; Err r -> r.work }`. Both Ok and Err carry `{work: ...; rounds: ...; score: ...}`.

**2. Add composable stop conditions in `pkg/guard/conditions.lx`**

New file exporting constructor functions. Each constructor returns a record with a `check` field — a closure taking a state record and returning Bool. The state record has fields: `turns`, `elapsed_ms`, `budget_pct`, `last_score`, `action_count`. Constructors: `max_turns`, `timeout_ms`, `budget_at`, `score_above`, `max_actions`. Combinators: `any_of` (any condition true → true), `all_of` (all conditions true → true).

**3. Create agent harness runner in `pkg/agents/harness.lx`**

New file exporting a `run_with` function that wraps any Agent with guardrail checking. Takes an agent handle (spawned subprocess) and an opts record with optional fields: `budget` (record passed to `budget.create`), `stop_when` (condition record from conditions.lx), `quality_gate` (record with `grader` function and `threshold`). The function runs the yield/handle/yield loop with automatic checks: before each turn, check budget status; after each turn, check stop conditions; optionally grade results. Returns when a stop condition fires, budget exhausts, or the agent breaks naturally.

This does NOT modify `std/agent.lx`. The harness is a wrapper that coordinates with the agent via the existing `~>?` ask protocol. An lx program uses it like:

```
use pkg/agents/harness
use pkg/guard/conditions {any_of max_turns timeout_ms}

handle = agent.spawn {command: "lx" args: ["agent" "worker.lx"]}
result = harness.run_with handle {
  budget: {tokens: 100000; cost_usd: 0.50}
  stop_when: any_of [max_turns 25; timeout_ms 60000]
}
```

# How It Works

The `refine` function is a standalone loop — no trait required. It receives work, grades it, optionally revises, and returns the best version. `quality.refine_work` at line 72 already expects this: it calls `refine initial { grade: ...; revise: ...; threshold; max_rounds: 3 }` and handles the result at line 94 with `result ? { Ok r -> r.work; Err r -> r.work }`.

The stop conditions module is purely functional — each constructor returns a record with a `check` closure. Combining with `any_of`/`all_of` wraps the list in a new closure that iterates. No new types, no Rust changes.

The harness runner manages the conversation loop externally. It spawns an agent, sends messages via `~>?`, receives results, and checks guardrails between turns. It builds the stop condition state record from tracked turn count, elapsed time (via `time.now`), budget percentage (via `budget.used_pct`), and last quality score. When any stop condition fires, it kills the agent and returns the last result.

The harness does NOT change how agents are written. Agents still implement the Agent trait with perceive/reason/act/reflect. The harness wraps the outer communication loop that drives the agent, adding resource checks between turns.

Brain programs do NOT use the harness — they have custom orchestration flows (saga-based pipeline, specialist agent dispatch, multi-tier memory) that are domain-specific and would not benefit from a generic wrapper. The harness is for simpler agents that follow the standard yield/handle/yield pattern.

# Gotchas

- **`std/workflow.lx` is embedded via `include_str!`.** The `refine` function is added to this file, which is fine — it has no imports beyond what workflow.lx already uses. The function is pure lx with no pkg/ dependencies.
- **`quality.refine_work` calls `refine` as a bare name (line 72).** This means `refine` must be in scope when `quality.lx` executes. Since `std/workflow` is auto-loaded as part of stdlib, its exports are available globally. The `+refine` export prefix makes it accessible.
- **The harness `run_with` function communicates with agents via `~>?` (ask).** This is the existing agent subprocess protocol — JSON-lines over stdin/stdout. The harness sends a message, waits for a response, checks guardrails, and repeats. It does not need access to the agent's internal state.
- **`budget.create` takes a record where numeric fields become dimensions.** Example: `{tokens: 100000; cost_usd: 0.50; tight_at: 50.0; critical_at: 80.0}`. Non-numeric fields (`tight_at`, `critical_at`) are threshold config, not dimensions. The harness passes `opts.budget` directly to `budget.create`.
- **`budget.used_pct` returns a record of dimension percentages**, e.g., `{tokens: 45.2; cost_usd: 30.0}`. To get a single number for the stop condition state, the harness takes the max value: `budget.used_pct b | values | fold 0.0 (acc v) { v > acc ? v : acc }`.

# Files Affected

| File | Change |
|------|--------|
| `crates/lx/std/workflow.lx` | Add `+refine` function (~25 lines) after `topo_sort` |
| `pkg/guard/conditions.lx` | New file — stop condition constructors and combinators (~40 lines) |
| `pkg/agents/harness.lx` | New file — `run_with` function wrapping agent with guardrails (~60 lines) |

---

## Task List

### Task 1: Implement the refine function in std/workflow.lx

Add an exported `+refine` function at the bottom of `crates/lx/std/workflow.lx`, after the `topo_sort` function (which ends at line 85). The function takes two arguments: `initial` (the starting work value) and `opts` (a record). Extract from opts: `grade_fn` as `opts.grade`, `revise_fn` as `opts.revise`, `threshold` as `opts.threshold ?? 80`, `max_r` as `opts.max_rounds ?? 3`. Create mutable bindings `work := initial` and `round := 0`. Enter a loop: increment round with `round <- round + 1`, call `result = grade_fn work`, check `result.score >= threshold` and if so break with `Ok {work: work; rounds: round; score: result.score}`, check `round >= max_r` and if so break with `Err {work: work; rounds: round; score: result.score}`, otherwise call `work <- revise_fn work result.feedback` and continue the loop. The function has no imports — it uses only core language features.

### Task 2: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 3: Commit refine implementation

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: implement refine function in std/workflow.lx"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 4: Create composable stop conditions module

Create a new file `pkg/guard/conditions.lx`. Add a header comment: `-- Composable stop conditions — combinable termination predicates for agent loops.`

Export the following constructor functions. Each returns a record with a `check` field that takes a `state` record argument and returns Bool. Use the `??` operator to default missing state fields to zero:

`+max_turns = (n) { {check: (state) { (state.turns ?? 0) >= n }} }`
`+timeout_ms = (ms) { {check: (state) { (state.elapsed_ms ?? 0) >= ms }} }`
`+budget_at = (pct) { {check: (state) { (state.budget_pct ?? 0.0) >= pct }} }`
`+score_above = (n) { {check: (state) { (state.last_score ?? 0.0) >= n }} }`
`+max_actions = (n) { {check: (state) { (state.action_count ?? 0) >= n }} }`

Export two combinator functions:

`+any_of = (conditions) { {check: (state) { conditions | any? (c) { c.check state } }} }`
`+all_of = (conditions) { {check: (state) { conditions | all? (c) { c.check state } }} }`

No imports needed — these are pure closures.

### Task 5: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 6: Commit conditions module

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add composable stop conditions module in pkg/guard/conditions.lx"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 7: Create agent harness runner

Create a new file `pkg/agents/harness.lx`. Add a header comment: `-- Agent harness — wraps agent communication with automatic budget, stop condition, and quality checks.`

Add imports: `use std/time`, `use pkg/agent/budget : budget_mod`.

Export a `+run_with` function taking two arguments: `handle` (an agent handle from `agent.spawn`) and `opts` (a record). The function:

1. Extract from opts: `budget_opts` as `opts.budget` (default None), `stop_cond` as `opts.stop_when` (default None), `quality_opts` as `opts.quality_gate` (default None).
2. Create mutable bindings: `turn_count := 0`, `start = time.now ()`, `active_budget := None`, `last_score := 0.0`, `last_result := None`.
3. If `budget_opts != None`, set `active_budget <- budget_mod.create budget_opts`.
4. Enter a loop:
   a. Increment `turn_count <- turn_count + 1`.
   b. If `active_budget != None`: call `status = budget_mod.status active_budget`. If `status == "exceeded"`, break with `Err {type: "BudgetExhausted"; turns: turn_count; last_result: last_result}`.
   c. Send message to agent: `result = handle ~>? {action: "handle"; turn: turn_count} ^`. Set `last_result <- result`.
   d. If `quality_opts != None`: call `grade = quality_opts.grader result`. Set `last_score <- grade.score`. If `grade.score < (quality_opts.threshold ?? 80)`, the result is below quality — log but continue (the agent's own refine loop handles revision).
   e. If `stop_cond != None`: build state record `check_state = {turns: turn_count; elapsed_ms: (time.now ()).ms - start.ms; budget_pct: active_budget != None ? (budget_mod.used_pct active_budget | values | fold 0.0 (acc v) { v > acc ? v : acc }) : 0.0; last_score: last_score; action_count: turn_count}`. Call `stop_cond.check check_state`. If true, break with `Ok {result: last_result; turns: turn_count; reason: "stop_condition"}`.
5. After loop breaks, return the break value.

### Task 8: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 9: Commit harness runner

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add agent harness runner with budget, stop conditions, and quality checks"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 10: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 11: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 12: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 13: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 14: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/AGENT_LIFECYCLE_GUARDRAILS.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **NEVER run raw cargo commands.** No `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`. ALWAYS use `just fmt`, `just test`, `just diagnose`, `just fix`. The `just` recipes include additional steps that raw `cargo` skips.
2. **Between implementation tasks, ONLY run `just fmt` + `git commit`.** Do NOT run `just test`, `just diagnose`, or any compilation/verification command between implementation tasks. These are expensive and only belong in the FINAL VERIFICATION section at the end.
3. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
4. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
5. **`just fmt` is not `cargo fmt`.** `just fmt` runs `dx fmt` + `cargo fmt` + `eclint`. Running `cargo fmt` alone is wrong.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.

## Task Loading Instructions

Read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
- `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
- `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded. Every sentence, every command, every instruction must be transferred exactly as written. Do NOT omit lines, rephrase instructions, drop the "verbatim" language from command instructions, or inject your own wording.
- `activeForm`: A present-continuous form of the subject (e.g., "Implementing the refine function")

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done. Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa. Do not run any command not specified in the current task. Do not "pre-check" compilation between implementation tasks. If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands. Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section.
