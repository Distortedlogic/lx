# Verification Unit 02: Org Chart Parse Blocker

## Goal

Restore parseability of `crates/lx-desktop/src/pages/org/chart.rs` so repo formatting and Rust diagnostics can run again.

## Why

The current workspace cannot complete `just fmt` or `just rust-diagnose` because `crates/lx-desktop/src/pages/org/chart.rs` fails to parse with an unexpected closing delimiter near the node-card rendering block. This blocks all later execute-loop verification, so it must be fixed before continuing broader audit remediation.

## Changes

- Repair the RSX syntax in `OrgChart` without changing the intended org chart behavior.
- Keep the fix local to `crates/lx-desktop/src/pages/org/chart.rs` unless a directly required import change is needed.
- Prefer the smallest parse-safe rewrite of the node-card rendering branch.

## Files Affected

- `work_items/verification_unit_02_org_chart_parse_blocker.md`
- `crates/lx-desktop/src/pages/org/chart.rs`

## Task List

1. Inspect the failing node-card rendering block in `OrgChart` and identify the local syntax causing the delimiter mismatch.
2. Rewrite that block into parse-safe RSX syntax without changing the rendered data or click behavior.
3. Re-read the full file to ensure the new structure does not introduce additional brace, hook, or control-flow issues.
4. Run repo verification commands again to confirm the parse blocker is gone.

## Verification

- `just fmt`
- `just rust-diagnose`
