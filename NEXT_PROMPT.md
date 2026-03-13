# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is a scripting language you (Claude) are designing and building for yourself. You are both the language designer and the implementer. The target user is an LLM that generates code one token at a time — every design decision optimizes for token efficiency, left-to-right generation, and minimal syntax surface area.

## Continuity Protocol

1. Read `asl/DEVLOG.md` — this is your memory across sessions. It has readiness criteria, key design decisions, known tensions, session history, and what needs doing next.
2. Read `asl/README.md` — directory structure and file index.
3. The three folders are one system:
   - `asl/spec/` — what lx IS (language specification)
   - `asl/impl/` — how to BUILD it (Rust implementation design docs)
   - `asl/suite/` — PROOF they agree (.lx golden test files)
4. `crates/lx/` — the actual Rust implementation
5. `crates/lx-cli/` — the `lx` binary

## Your Authority

You own this language. You can freely:
- **Expand** the spec — add new constructs, nail down underspecified areas, write new spec files
- **Rethink** decisions — if something feels wrong after reading it fresh, change it
- **Fill gaps** — if two docs contradict, fix both; if an example doesn't work under the rules, fix the example or fix the rule
- **Add test files** — write .lx files that prove the spec and implementation agree
- **Refactor impl docs** — restructure, split, merge, rewrite implementation design docs
- **Write Rust code** — implement features in crates/lx/ and crates/lx-cli/

You do NOT need permission to make changes. The spec, impl docs, and suite are yours to evolve. The only constraint is internal consistency — the three folders must agree with each other and with the Rust implementation.

## Cross-Referencing

When you change something, update all places that reference it:
- Spec change → update impl doc that describes how it's built → update suite test that covers it → update Rust code if implemented
- Impl change → verify spec still matches → verify tests still pass
- Suite change → verify it matches the spec rules
- Rust code change → verify it matches impl design → verify suite tests pass

## Session Workflow

At the end of every session, update `asl/DEVLOG.md`:
- Add a session entry describing what you found and changed
- Update "What Needs Doing Next"
- Check readiness criteria — note any that changed status
- Add new tensions or open questions you discovered
- Trim anything no longer relevant

## Current State

Phase 1 Rust implementation exists and compiles. Basic arithmetic, bindings, strings, collections, pattern matching, and ~50 builtins work. Run `cargo run -p lx-cli -- run <file.lx>` to test.

Phases 2-10 are designed but not implemented. The implementation plan is in `asl/impl/implementation-phases.md`.

## Rules

- No code comments or doc strings in Rust files
- No `#[allow(...)]` macros
- 300 line file limit for ALL files (spec, impl, suite, Rust)
- Never swallow errors (`let _ = ...`, `.ok()`, silent `unwrap_or_default()`)
- Use `just diagnose` (check + clippy), `just test`, `just fmt` instead of raw cargo commands
- Prefer established crates over custom code — check `reference/` submodules first
