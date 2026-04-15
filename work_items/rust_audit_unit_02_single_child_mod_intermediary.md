# Rust Audit Unit 02: Single-Child Module Intermediary

## Goal

Remove the verified single-child `mod.rs` intermediary in `lx-eval` without changing the public module path.

## Why

`./rules/rust-audit.md` flags `mod.rs` files that only namespace one child module and re-export it. `crates/lx-eval/src/builtins/agent/mod.rs` is an exact match: it only declares `mod spawn;` and re-exports `bi_agent_spawn`. The child file should become `agent.rs`, and the intermediary should disappear.

## Changes

- Move `crates/lx-eval/src/builtins/agent/spawn.rs` to `crates/lx-eval/src/builtins/agent.rs`.
- Delete `crates/lx-eval/src/builtins/agent/mod.rs`.
- Keep the existing `crate::builtins::agent::bi_agent_spawn` path unchanged.

## Files Affected

- `crates/lx-eval/src/builtins/agent/mod.rs`
- `crates/lx-eval/src/builtins/agent/spawn.rs`
- `crates/lx-eval/src/builtins/agent.rs`

## Task List

1. Move the only child module implementation into `crates/lx-eval/src/builtins/agent.rs`.
2. Remove the now-unnecessary `mod.rs` intermediary.
3. Verify that references to `crate::builtins::agent::bi_agent_spawn` still resolve unchanged.
4. Run formatting and Rust diagnostics.

## Verification

- `find crates/lx-eval/src/builtins/agent -maxdepth 2 -type f | sort`
- `rg -n 'crate::builtins::agent::bi_agent_spawn|pub(crate) mod agent;' crates --type rust`
- `just fmt`
- `just rust-diagnose`
