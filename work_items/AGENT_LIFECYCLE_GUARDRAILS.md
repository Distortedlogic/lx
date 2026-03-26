# Goal

Wire the Agent trait's `run` loop to automatically compose existing guardrail packages (budget, circuit breaker, quality grading, stop conditions) so agents get harness-level reliability by declaring fields instead of manually calling guard functions. Implement the missing `refine` function in `std/workflow.lx`. Add composable stop conditions as a new `pkg/guard/conditions.lx` module. Update the Agent trait's `run` method to check declared guardrails before/after each turn. Update brain programs to use the new declarative fields instead of manual wiring, validating the design and shrinking the brain code.

# Why

- The benchmarks research found harness design drives agent scores more than model choice (CORE-Bench: 36-point gain from switching scaffolds with same model). The harness IS budget tracking + quality gates + circuit breaking + termination conditions. lx already has all these packages but they require manual wiring in every program.
- `pkg/agent/quality.lx` line 72 calls `refine` but the function does not exist anywhere in the codebase — not as a Rust builtin, not as an exported lx function. Every `refine_work`, `refine_response`, and `refine_code` call is broken.
- Brain's `main.lx` (203 lines) and `orchestrator.lx` manually call `monitor.guard()`, `budget.spend()`, `quality.grade()`, `circuit.check()` at every step. This boilerplate is repeated across brain agents and any future lx agent program.
- The CircuitBreaker in `pkg/guard/circuit.lx` checks max_turns/max_actions/max_time/repetition as a config blob, but conditions cannot be composed algebraically. AutoGen's composable termination with `|`/`&` operators was identified as one of its best design patterns.

# What Changes

**1. Implement `refine` in `std/workflow.lx`**

Add an exported `refine` function after the existing `topo_sort` function. It takes an initial value and an options record with `grade` (function returning `{score: Int; feedback: Str}`), `revise` (function taking current work and feedback string, returning revised work), `threshold` (Int, default 80), and `max_rounds` (Int, default 3). The function loops: call grade on current work, if score meets threshold return `Ok {work: work; rounds: round; score: result.score}`, if max rounds reached return `Err {work: work; rounds: round; score: result.score}`, otherwise call revise with current work and feedback, set work to the result, and continue.

**2. Add composable stop conditions in `pkg/guard/conditions.lx`**

Create a new file exporting condition constructor functions. Each constructor returns a record with a `check` field that takes a state record and returns Bool. The state record has fields: `turns` (Int), `elapsed_ms` (Int), `budget_pct` (Float), `last_score` (Float), `action_count` (Int). Constructor functions: `max_turns` (takes Int n, checks `state.turns >= n`), `timeout_ms` (takes Int ms, checks `state.elapsed_ms >= ms`), `budget_at` (takes Float pct, checks `state.budget_pct >= pct`), `score_above` (takes Float n, checks `state.last_score >= n`), `max_actions` (takes Int n, checks `state.action_count >= n`). Combinator functions: `any_of` (takes list of conditions, returns true if any condition's check returns true), `all_of` (takes list of conditions, returns true if all conditions' check returns true).

**3. Add optional guardrail fields to the Agent trait in `std/agent.lx`**

Add three optional fields after the existing `max_turns` method: `budget` (default None — when set to a record like `{tokens: 100000; cost_usd: 0.50}`, the run loop creates a budget via `budget.create` and calls `budget.spend` after each `think`/`think_with` call), `stop_when` (default None — when set to a condition record from `pkg/guard/conditions.lx`, the run loop calls `condition.check` after each turn and breaks if true), `quality_gate` (default None — when set to a record like `{grader: fn; threshold: 80}`, the run loop calls the grader on the result of each `handle` call and if below threshold, calls `refine` with the work).

Modify the existing `run` method to check these fields. Before the yield/handle/yield loop body: if `self.budget` is not None, check budget status and break with `Err BudgetExhausted` if exceeded. After handle returns: if `self.stop_when` is not None, build the state record from current turn count, elapsed time, budget percentage, and last score, call `self.stop_when.check state`, and break if true. If `self.quality_gate` is not None, call the grader on the result; if below threshold, call refine.

Add `use pkg/agent/budget : budget_mod` and `use pkg/guard/conditions` and `use std/time` as imports at the top of `std/agent.lx`, after the existing `use std/prompt {Prompt}` on line 1. The file already imports `std/prompt` — add the new imports on separate lines below it.

**4. Update brain programs to use declarative Agent fields**

In `programs/brain/main.lx`: remove manual `monitor.guard`, `budget.spend`, and `quality.grade` calls where they duplicate the new Agent trait guardrails. The brain agent can set `budget`, `stop_when`, and `quality_gate` fields and let the trait's `run` loop handle them.

In `programs/brain/orchestrator.lx`: same pattern — declare guardrails on the orchestrator's agent setup rather than manually calling guard functions at each step.

# How It Works

The `refine` function is a standalone loop — no trait required. It receives work, grades it, optionally revises, and returns the best version. `quality.refine_work` already has the call site at line 72 expecting this signature. The function's Ok/Err return matches the existing `result ? { Ok r -> r.work; Err r -> r.work }` pattern at line 94.

The stop conditions module is purely functional — each constructor returns a record with a `check` closure. Combining with `any_of`/`all_of` wraps the list in a new closure that iterates. No new types, no Rust changes. An agent declares `stop_when: any_of [max_turns 25; timeout_ms 60000; budget_at 100.0]` and the run loop calls `self.stop_when.check state` each turn.

The Agent trait changes are backward-compatible. All new fields default to None. Existing agents that do not set `budget`, `stop_when`, or `quality_gate` get the current behavior unchanged. The `run` method checks `self.budget != None` before engaging budget logic, etc.

# Files Affected

| File | Change |
|------|--------|
| `crates/lx/std/workflow.lx` | Add `+refine` function (~25 lines) |
| `pkg/guard/conditions.lx` | New file — stop condition constructors and combinators (~60 lines) |
| `crates/lx/std/agent.lx` | Add `budget`, `stop_when`, `quality_gate` fields; modify `run` loop; add imports |
| `programs/brain/main.lx` | Remove manual guard/budget/quality calls, declare fields instead |
| `programs/brain/orchestrator.lx` | Remove manual guard/budget calls, declare fields instead |

---

## Task List

### Task 1: Implement the refine function in std/workflow.lx

Add an exported `+refine` function at the bottom of `crates/lx/std/workflow.lx`, after the `topo_sort` function. The function takes two arguments: `initial` (the starting work value) and `opts` (a record). Extract from opts: `grade_fn` as `opts.grade`, `revise_fn` as `opts.revise`, `threshold` as `opts.threshold ?? 80`, `max_r` as `opts.max_rounds ?? 3`. Create mutable bindings `work := initial` and `round := 0`. Enter a loop: increment round, call `result = grade_fn work`, check `result.score >= threshold` and if so break with `Ok {work: work; rounds: round; score: result.score}`, check `round >= max_r` and if so break with `Err {work: work; rounds: round; score: result.score}`, otherwise call `work <- revise_fn work result.feedback` and continue the loop.

### Task 2: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 3: Commit refine implementation

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: implement refine function in std/workflow.lx"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 4: Create composable stop conditions module

Create a new file `pkg/guard/conditions.lx`. Add a header comment: `-- Composable stop conditions — combinable termination predicates for agent loops.` Add `use std/time`. Export the following constructor functions, each returning a record with a `check` field that takes a `state` record argument and returns Bool:

- `+max_turns = (n) { {check: (state) { (state.turns ?? 0) >= n }} }`
- `+timeout_ms = (ms) { {check: (state) { (state.elapsed_ms ?? 0) >= ms }} }`
- `+budget_at = (pct) { {check: (state) { (state.budget_pct ?? 0.0) >= pct }} }`
- `+score_above = (n) { {check: (state) { (state.last_score ?? 0.0) >= n }} }`
- `+max_actions = (n) { {check: (state) { (state.action_count ?? 0) >= n }} }`

Export two combinator functions:

- `+any_of = (conditions) { {check: (state) { conditions | any? (c) { c.check state } }} }`
- `+all_of = (conditions) { {check: (state) { conditions | all? (c) { c.check state } }} }`

### Task 5: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 6: Commit conditions module

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add composable stop conditions module in pkg/guard/conditions.lx"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 7: Add guardrail fields and imports to the Agent trait

Edit `crates/lx/std/agent.lx`. The file currently starts with `use std/prompt {Prompt}` on line 1, followed by a blank line, then `+Trait Agent = {` on line 3. Add three new import lines between line 1 and the blank line: `use pkg/agent/budget : budget_mod`, `use pkg/guard/conditions`, `use std/time`. The file should then start with four `use` lines, blank line, then the trait.

Inside the `+Trait Agent = {` block, after the existing `max_turns = () { 25 }` line (currently line 52), add three new fields on separate lines:

- `budget = None`
- `stop_when = None`
- `quality_gate = None`

### Task 8: Modify the Agent trait run loop to check guardrails

Edit `crates/lx/std/agent.lx`. The current `run` method is at lines 36-43 and looks like this:

```
  run = () {
    self.init ()
    loop {
      msg = yield {status: "ready"}
      result = self.handle msg
      yield result
    }
  }
```

Replace it with a version that tracks turn state and checks guardrails. The new `run` method should: call `self.init ()`, create mutable bindings `turn_count := 0` and `start = time.now ()` and `active_budget := None`, check `self.budget != None` and if so set `active_budget <- budget_mod.create self.budget`, then enter the loop. In the loop: increment `turn_count`, if `active_budget != None` call `budget_mod.status active_budget` and if the result is `"exceeded"` then break with `Err {type: "BudgetExhausted"}`. Then yield `{status: "ready"}` to get msg. Call `result = self.handle msg`. If `self.stop_when != None`, build a state record with `turns: turn_count`, `elapsed_ms: (time.now ()).ms - start.ms`, `budget_pct: active_budget != None ? (budget_mod.used_pct active_budget | values | first ?? 0.0) : 0.0`, `last_score: 0.0`, `action_count: turn_count` — then call `self.stop_when.check state` and if true break with result. Yield result and continue the loop.

Do NOT change or remove any existing methods other than `run`. The following methods must remain exactly as they are: `role`, `goal`, `system_prompt`, `bootstrap_context`, `examples`, `max_context_tokens`, `init`, `perceive`, `reason`, `act`, `reflect`, `handle`, `build_prompt`, `on_turn_start`, `on_turn_end`, `on_error`, `tools`, `max_turns`, `think`, `think_with`, `think_structured`, `use_tool`, `delegate`, `escalate`, `describe`, `health`, `ask`, `tell`.

### Task 9: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 10: Commit Agent trait guardrails

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add budget, stop_when, quality_gate guardrail fields to Agent trait"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 11: Update brain/main.lx to use declarative guardrails

Edit `programs/brain/main.lx`. In the `think` function, remove the manual `mon.guard state "reasoning" ^` call at line 92 and the manual `ensure_quality` function call pattern (lines 152-162) where it manually grades and refines. Instead, where the brain creates its cognitive agent configuration, add `budget: {tokens: 100000; cost_usd: 1.00}` and `stop_when: any_of [max_turns 50; timeout_ms 120000]` declarations. Add `use pkg/guard/conditions {any_of max_turns timeout_ms}` to the imports. Keep all other logic unchanged — the brain's custom perception/reasoning/tool execution flow remains manual since it is domain-specific, not generic guardrail logic.

### Task 12: Format and commit

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 13: Commit brain update

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: use declarative guardrails in brain/main.lx"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 14: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 15: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 16: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 17: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 18: Remove work item file

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
