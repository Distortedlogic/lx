# Goal

Verify the entire restructure compiles clean, no dead code, no broken imports, no warnings. Fix anything found.

# Why

WU-A through WU-D made structural changes across two repos (common_pty and lx-desktop): routes deleted, pages moved, modules renamed, imports changed, voice pipeline rewritten. This unit catches any breakage that individual units couldn't detect in isolation.

# Prerequisites

All of WU-A, WU-B, WU-C, and WU-D must be completed.

# Task List

### Task 1: Verify compilation and fix all errors

**Subject:** Run diagnostics and fix any compile errors

**Description:** Run `just rust-diagnose` from `/home/entropybender/repos/lx`. Read the output. If there are any errors in the `lx-desktop` crate:

For each error:
1. Read the flagged file and line
2. Identify the root cause (missing import, wrong path, moved module, etc.)
3. Fix it minimally

Common issues to expect after a restructure:
- Imports referencing `crate::pages::repos::*` or `crate::pages::terminals::*` that no longer exist
- Menu bar actions referencing removed routes (e.g., `Route::Terminals {}`)
- Server API imports referencing moved modules
- Missing `pub` on items that moved between modules

Do NOT add features, refactor, or clean up code beyond fixing compile errors.

**ActiveForm:** Fixing compile errors from restructure

---

### Task 2: Fix all warnings

**Subject:** Eliminate every clippy/compiler warning in lx-desktop

**Description:** Run `just rust-diagnose` again after Task 1. If there are any warnings in `lx-desktop`:

For each warning:
1. Read the flagged file and line
2. Fix the warning:
   - `unused_import`: remove the import
   - `unused_variable`: remove the variable or use it
   - `unused_mut`: remove the `mut`
   - `dead_code`: if the item is truly unused across the entire crate, remove it. If it's used but the compiler can't see the usage (e.g., behind a cfg gate), investigate before removing.
   - `clippy::*`: fix per clippy's suggestion

Do NOT add `#[allow(...)]` macros. Do NOT use underscore-prefix to suppress warnings (except for the pre-existing `_ECHARTS_JS` and `_WIDGET_BRIDGE_JS` statics in `app.rs` which are asset registrations).

**ActiveForm:** Fixing all warnings from restructure

---

### Task 3: Verify voice_backend.rs is still used

**Subject:** Check that voice_backend.rs has live callers

**Description:** Search the codebase for references to `voice_backend`, `ClaudeCliBackend`, `SESSION_ID`, `SESSION_CREATED`, and `SYSTEM_PROMPT`.

Expected callers:
- `AgentView` in `terminal/view.rs` uses `ClaudeCliBackend.query()` (from WU-2)
- `voice_banner.rs` uses `SESSION_ID`, `SESSION_CREATED`, and `SYSTEM_PROMPT` (from WU-D)
- `agents/mod.rs` uses `SESSION_ID` for the session display

If all three callers exist, `voice_backend.rs` is live. No action needed.

If `ClaudeCliBackend` has no callers (meaning `AgentView` was changed or removed), then the `AgentBackend` import and trait impl are dead code. Remove `ClaudeCliBackend` struct and its `impl AgentBackend` block, but keep `SESSION_ID`, `SESSION_CREATED`, and `SYSTEM_PROMPT` since the voice pipeline uses them.

If `voice_backend.rs` is entirely unused, delete it and remove `pub mod voice_backend;` from `lib.rs`.

**ActiveForm:** Verifying voice_backend.rs has live callers

---

### Task 4: Final format and verify

**Subject:** Format and confirm zero errors zero warnings

**Description:** Run `just fmt` from `/home/entropybender/repos/lx`. Then run `just rust-diagnose`. The expected output for lx-desktop is `0 errors, 0 warnings`. If not, return to Task 1.

**ActiveForm:** Final format and diagnostic verification

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_RESTRUCTURE_CLEANUP.md" })
```
