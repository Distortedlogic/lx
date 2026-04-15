# Dioxus Audit Unit 03: Remaining Page Class Attribute Cleanup

## Goal

Remove the remaining verified page-level RSX `class` attributes that mix static classes and interpolated dynamic values inside the same string literal.

## Why

`./rules/dioxus-audit.md` forbids mixed static and dynamic class strings because they hide dynamic behavior inside long string literals and make class composition harder to read and audit. The first page batch is already complete; this unit covers the remaining verified page hits from the strict regex pass.

## Changes

- Split each mixed `class: "static {dynamic}"` string in the listed page files into one static `class` attribute and one or more dynamic `class` attributes.
- Preserve existing behavior, including empty-string dynamic classes and existing inline styles.
- Do not refactor unrelated layout or logic.

## Files Affected

- `crates/lx-desktop/src/pages/plugins/plugin_settings.rs`
- `crates/lx-desktop/src/pages/goals/properties.rs`
- `crates/lx-desktop/src/pages/goals/detail.rs`
- `crates/lx-desktop/src/pages/agents/budget_tab.rs`
- `crates/lx-desktop/src/pages/agents/skills_tab.rs`
- `crates/lx-desktop/src/pages/agents/list.rs`
- `crates/lx-desktop/src/pages/agents/transcript_blocks.rs`
- `crates/lx-desktop/src/pages/costs/budget_card.rs`
- `crates/lx-desktop/src/pages/issues/properties.rs`
- `crates/lx-desktop/src/pages/issues/kanban_card.rs`
- `crates/lx-desktop/src/pages/issues/kanban.rs`
- `crates/lx-desktop/src/pages/issues/list.rs`
- `crates/lx-desktop/src/pages/projects/detail.rs`
- `crates/lx-desktop/src/pages/projects/list.rs`
- `crates/lx-desktop/src/pages/projects/new_dialog.rs`
- `crates/lx-desktop/src/pages/approvals/detail.rs`
- `crates/lx-desktop/src/pages/approvals/card.rs`

## Task List

1. Update every verified mixed class string in the listed files so all static classes remain in static `class` attributes and every interpolated value moves into its own dynamic `class` attribute.
2. Preserve behavior for dynamic status colors, rings, opacity classes, and fill-color classes without introducing helper wrappers or unrelated refactors.
3. Re-run the strict mixed-class regex on the listed files and confirm it returns no matches.
4. Run the repository formatting and Rust diagnostics commands used by the existing audit units.

## Verification

- `rg -n -P 'class:\s*"(?:[^"{][^"]*\{[^}]+\}[^"]*|\{[^}]+\}[^"\s][^"]*)"' crates/lx-desktop/src/pages/plugins/plugin_settings.rs crates/lx-desktop/src/pages/goals/properties.rs crates/lx-desktop/src/pages/goals/detail.rs crates/lx-desktop/src/pages/agents/budget_tab.rs crates/lx-desktop/src/pages/agents/skills_tab.rs crates/lx-desktop/src/pages/agents/list.rs crates/lx-desktop/src/pages/agents/transcript_blocks.rs crates/lx-desktop/src/pages/costs/budget_card.rs crates/lx-desktop/src/pages/issues/properties.rs crates/lx-desktop/src/pages/issues/kanban_card.rs crates/lx-desktop/src/pages/issues/kanban.rs crates/lx-desktop/src/pages/issues/list.rs crates/lx-desktop/src/pages/projects/detail.rs crates/lx-desktop/src/pages/projects/list.rs crates/lx-desktop/src/pages/projects/new_dialog.rs crates/lx-desktop/src/pages/approvals/detail.rs crates/lx-desktop/src/pages/approvals/card.rs`
- `just fmt`
- `just rust-diagnose`
