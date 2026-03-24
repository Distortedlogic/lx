# Goal

Remove dead infrastructure from the AST and linter modules: delete the unused ExprMatcher module and delete the dead-code build_parent_map function and its module.

# Why

- `ExprMatcher` in `linter/matcher.rs` is used by zero lint rules, zero tests, and covers only 8 of 28 Expr variants — it will silently rot as the AST evolves
- `build_parent_map` in `ast/parent_map.rs` has zero call sites anywhere in the codebase — it is dead code that adds maintenance burden

# What changes

1. Delete `linter/matcher.rs` and its module declaration
2. Delete `ast/parent_map.rs`, its module declaration, and its re-export

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/linter/matcher.rs` | Delete entirely |
| `crates/lx/src/linter/mod.rs` | Remove `pub mod matcher;` line |
| `crates/lx/src/ast/parent_map.rs` | Delete entirely |
| `crates/lx/src/ast/mod.rs` | Remove `mod parent_map;` line and `pub use parent_map::build_parent_map;` re-export |

# Task List

### Task 1: Delete ExprMatcher

Delete the file `crates/lx/src/linter/matcher.rs` entirely. In `crates/lx/src/linter/mod.rs`, remove the `pub mod matcher;` line. Verify no other file imports from `linter::matcher` by searching for `matcher::` and `ExprMatcher` across the codebase.

### Task 2: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 3: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: delete unused ExprMatcher"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Delete build_parent_map

Delete the file `crates/lx/src/ast/parent_map.rs` entirely. In `crates/lx/src/ast/mod.rs`, remove the `mod parent_map;` line and the `pub use parent_map::build_parent_map;` re-export line. Verify no other file references `build_parent_map` by searching across the codebase.

### Task 5: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 6: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: delete dead-code build_parent_map"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 7: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 8: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 9: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 10: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 11: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/VISITOR_PHASE0_STANDALONE_CLEANUPS.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

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

To execute this work item, read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
- `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
- `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded
- `activeForm`: A present-continuous form of the subject

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execute tasks strictly in order. Run commands EXACTLY as written. Do not substitute `cargo` for `just`. Do not run any command not specified in the current task. Do not "pre-check" compilation between implementation tasks. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands.
