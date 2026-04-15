# Dioxus Audit Unit 04: Remaining Mixed Class Attributes

## Goal

Remove the last verified RSX `class` strings that still mix interpolated dynamic values with static classes after the earlier page cleanup units.

## Why

The page batches cleared the obvious static-first cases, but the repo still contains dynamic-first mixed class strings such as `"{wrapper} animate-transcript-enter"` and component class props like `"{select_cls} w-full"`. These are the same audit violation under `./rules/dioxus-audit.md` and need to be cleared before the Dioxus class-attribute audit is complete.

## Changes

- Split remaining DOM-node mixed class strings into static and dynamic `class` attributes.
- For component props that accept a single `class: String`, compose the class string outside the RSX call instead of mixing interpolation and static tokens directly in the RSX attribute.
- Preserve behavior exactly.

## Files Affected

- `crates/lx-desktop/src/pages/agents/transcript_groups.rs`
- `crates/lx-desktop/src/pages/agents/transcript_blocks.rs`
- `crates/lx-desktop/src/pages/routines/schedule_editor.rs`

## Task List

1. Update the two transcript group/container wrappers so `animate-transcript-enter` remains static and the dynamic wrapper class moves to its own `class` attribute.
2. Update `ScheduleEditor` and its helper functions so `Select` and `input` class props no longer use interpolated-plus-static RSX strings; use explicit precomposed strings instead.
3. Re-run the repo-wide mixed-class regex and confirm it finds no remaining mixed class strings.
4. Run formatting and Rust diagnostics.

## Verification

- `rg -n -P 'class:\s*"(?!\s*\{[^}]+\}\s*")[^"]*\{[^}]+\}[^"]*"' crates --type rust`
- `just fmt`
- `just rust-diagnose`
