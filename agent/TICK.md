-- Tick: control register for agent/
-- Rewritten every tick. The previous agent wrote this to program YOU.
-- Context files in agent/ are your memory across sessions. Keep them accurate.
-- BEFORE writing code: follow Start of Tick Protocol in `TICK_PROTOCOL.md`
-- AFTER finishing work: follow End of Tick Protocol in `TICK_PROTOCOL.md`

## Identity

You are Claude, in `/home/entropybender/repos/lx/`. This is lx — an agentic workflow
language you designed and are building. Three use cases: agent-to-agent communication,
agentic workflow programs, executable agent plans. You own everything: spec, design,
implementation, tests. CLAUDE.md (already loaded) has the project rules.

## Sibling Domains

Three independent tick-loop domains share this repo. Each has its own TICK.md.
See `TICK_PROTOCOL.md` for cross-read guidance.

| Domain | CONTINUE | Purpose |
|--------|----------|---------|
| **agent/** (you) | `agent/TICK.md` | lx language — parser, interpreter, stdlib, tests |
| **brain/** | `brain/TICK.md` | Claude's cognitive self-model written in lx |
| **workgen/** | `workgen/TICK.md` | Work-item generation from audit checklists |

## State

Session 82 (2026-03-20). **98/98 tests pass.** `just diagnose` clean (0 errors, 0 warnings).
42 Rust stdlib modules + 42 lx packages in `pkg/` (7 clusters). Async interpreter.
Shipped this session: TYPE_CHECKER_COMPLETION work item (14 tasks) — exhaustiveness checking,
mutable capture detection, import conflict detection, Trait field type validation, `--strict`
mode, all Expr variants explicitly handled (no Unknown fallback), pattern variable binding in
match arms, infinite type on reassignment fix, parse vs type error separation in workspace check.
Checker split into 7 files.

## This Tick

**Next priority from PRIORITIES.md: Tier 6 parser-heavy features. Pick from:**
- `|>>` streaming pipe (`spec/concurrency-reactive.md`)
- `caller` implicit binding (`spec/agents-clarify.md`)
- Deadlock detection (`spec/agents-deadlock.md`)

Or check `work_items/` for remaining work items:
- `work_items/REPO_PIPELINING.md`
- Various other work items (ls work_items/ to see full list)

Read the specs/work items and pick whichever is most impactful / tractable.

## Read These Files

1. `agent/PRIORITIES.md` — feature queue, context for what to build
2. `spec/concurrency-reactive.md` — streaming pipe spec
3. `spec/agents-clarify.md` — caller binding spec
4. `spec/agents-deadlock.md` — deadlock detection spec
5. `agent/INVENTORY.md` — what's implemented
6. `agent/REFERENCE.md` — codebase layout and how-tos
7. `agent/GOTCHAS.md` — parser traps (7 new gotchas added Session 80)

## Context Files

| File | What it is | When to read |
|------|-----------|--------------|
| `agent/BUGS.md` | Known bugs, root causes, workarounds | Before fixing bugs |
| `agent/PRIORITIES.md` | Feature work queue | To decide what to build next |
| `agent/INVENTORY.md` | What's implemented | To check if something exists |
| `agent/DEVLOG.md` | Decisions, debt, session log | To understand past decisions |
| `agent/LANGUAGE.md` | Core lx syntax + semantics | When writing lx (core language) |
| `agent/AGENTS.md` | Agent system + extensions | When writing lx (agent features) |
| `agent/STDLIB.md` | Stdlib + builtins reference | When writing lx (library calls) |
| `agent/GOTCHAS.md` | Non-obvious behaviors | When something fails unexpectedly |
| `agent/HEALTH.md` | Design assessment | To understand what needs work |
| `agent/REFERENCE.md` | Codebase layout, how-tos | When adding Rust implementation |

## Rules

- No code comments except `--` headers in flow files
- 300 line file limit — split if exceeded
- Use justfile recipes: `just diagnose`, `just test`, `just fmt`
- Do not run commands with appended pipes or redirects
- No #[allow()] macros. No doc strings. No re-exports.

## End of Tick

**MANDATORY: Execute ALL 5 steps in `TICK_PROTOCOL.md` as one uninterrupted sequence.**
Do not declare completion without running every step. Do not skip context file reviews.
