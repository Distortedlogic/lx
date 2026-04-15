# Dioxus Audit Unit 02: Page Class Attribute Cleanup

## Goal

Remove the verified Dioxus audit violations in page files where static utility classes and dynamic fragments are mixed inside one `class` string.

## Why

After the primitive cleanup and verification-blocker fix, the strict mixed-class regex from `rules/dioxus-audit.md` still returns page-level violations under `crates/lx-desktop/src/pages/`. This unit is intentionally limited to the first verified batch of eight files so the cleanup remains mechanical and reviewable.

The Dioxus audit rule requires these to be split into a static `class` attribute plus separate dynamic `class` attributes. The fix is local and should not change rendering or behavior.

## Changes

- Split every remaining mixed `class` string in the affected first-batch page files into explicit static and dynamic `class` attributes.
- Preserve the same DOM structure, event handlers, data flow, and styling semantics.
- Eliminate spacing tricks such as `...{bg}` and `min-h-9{sel_class}` by moving the dynamic token into its own `class` attribute.

## Files Affected

- `work_items/dioxus_audit_unit_02_page_class_attr_cleanup.md`
- `crates/lx-desktop/src/pages/company_skills/mod.rs`
- `crates/lx-desktop/src/pages/company_import.rs`
- `crates/lx-desktop/src/pages/org/tree_view.rs`
- `crates/lx-desktop/src/pages/dashboard/activity_charts.rs`
- `crates/lx-desktop/src/pages/plugins/plugin_card.rs`
- `crates/lx-desktop/src/pages/companies/company_card.rs`
- `crates/lx-desktop/src/pages/company_skills/skill_tree.rs`
- `crates/lx-desktop/src/pages/agents/runs_tab.rs`

## Task List

1. Rewrite each matched mixed `class` string in the listed first-batch files into one static `class` plus one or more dynamic `class` attributes.
2. Preserve empty-string safety by removing embedded spacing dependencies from the dynamic fragments.
3. Re-run the strict mixed-class regex on the files listed in this work item and confirm it returns no matches for this batch.
4. Run repo formatting and Rust diagnostics to verify the cleanup did not introduce syntax or lint regressions.

## Verification

- `rg -n -P 'class:\\s*\"(?:[^\"{][^\"]*\\{[^}]+\\}[^\"]*|\\{[^}]+\\}[^\"\\s][^\"]*)\"' crates/lx-desktop/src/pages/company_skills/mod.rs crates/lx-desktop/src/pages/company_import.rs crates/lx-desktop/src/pages/org/tree_view.rs crates/lx-desktop/src/pages/dashboard/activity_charts.rs crates/lx-desktop/src/pages/plugins/plugin_card.rs crates/lx-desktop/src/pages/companies/company_card.rs crates/lx-desktop/src/pages/company_skills/skill_tree.rs crates/lx-desktop/src/pages/agents/runs_tab.rs`
- `just fmt`
- `just rust-diagnose`
