# GSD-2: Design Principles and Key Patterns

## Core Design Principles

### 1. State Lives on Disk

`.gsd/` is the sole source of truth. No in-memory state survives across sessions. Auto mode reads files, makes decisions, writes files, and advances. This enables crash recovery, multi-terminal steering, session resumption, and parallel workers.

### 2. Fresh Session Per Unit

Every task, research phase, and planning step gets a clean context window. The LLM starts oriented (files already pre-inlined in the prompt) instead of spending tool calls reading files. Prevents quality degradation from context accumulation.

### 3. Deterministic Orchestration, Non-Deterministic Execution

The dispatch rules, phase transitions, and verification gates are **deterministic code** (a TypeScript state machine). The LLM handles only **content generation** — writing plans, code, summaries. This means orchestration overhead is zero (no tokens spent on "what do I do next?").

### 4. Automation with Human-in-the-Loop

Default to autonomous execution, but provide natural seam points for human steering:
- `/gsd discuss` in a separate terminal writes decisions picked up at next phase boundary
- `/gsd capture` injects thoughts into the pipeline
- `/gsd steer` hard-steers plan documents during execution
- `Escape` pauses auto mode
- Step mode advances one unit at a time for review

### 5. Verification is Mechanical, Not Behavioral

Quality gates are code (shell commands: lint, test, typecheck), not LLM behavior. Verification commands run automatically after each task with auto-fix retries. The agent doesn't "decide" whether to test — it's forced to.

### 6. Meaningful Commits, Not Generic Messages

Commit messages are generated from task summaries. `git log` tells you what actually shipped, not "update files" or "fix stuff."

### 7. Cost-Conscious by Default

Three token profiles (budget/balanced/quality), complexity-based model routing, budget ceilings with enforcement, adaptive learning from routing history. The system actively minimizes cost without sacrificing quality.

### 8. Extensibility as First-Class

The Pi SDK extension system allows deep customization without forking: custom tools, commands, UI, hooks, model overrides, system prompt injection. 17+ bundled extensions. User extensions in `~/.gsd/agent/extensions/`.

## Architectural Patterns

### Dispatch Rules as Data Structure

Instead of a 130-line if-else chain, dispatch rules are an array of `{name, match}` tuples evaluated in order (first match wins). Testable, extensible, inspectable. Each rule is a pure function from state to action.

### Inlining Over Tool Calls

Prompts inline all file content upfront via `inlineFile()` / `inlineFileSmart()`. Agents don't waste context reading files they already have. Eliminates blocking tool calls for initialization. Every prompt has a `## Inlined Context` section separating preloaded content from fresh instructions.

### Semantic Chunking for Large Files

Files over 3KB can be chunked by TF-IDF relevance to task description via `inlineFileSmart()`. Saves ~20-40% tokens while keeping relevant content.

### Boundary Maps for Interface Contracts

Slices define what they produce and what they consume in the roadmap's Boundary Map table. This prevents invisible dependencies and forces interface thinking before implementation.

### Summary Distillation and Injection

Prior summaries provide continuity across sessions. For 3+ dependency summaries, distillation/truncation keeps injected context within budget (~2500 tokens). Summary frontmatter (`provides`, `key_files`, `key_decisions`, `patterns_established`) enables structured consumption.

### Two-File Loader Pattern

`loader.ts` sets environment variables with zero SDK imports, then dynamically imports `cli.ts`. This ensures `PI_PACKAGE_DIR` and other env vars are set before any SDK code evaluates. Fast-paths `--version` and `--help` to avoid ~1s import time.

### Always-Overwrite Resource Sync

Bundled extensions and agents are synced to `~/.gsd/agent/` on every launch (not just first run). `npm update -g gsd-pi` takes effect immediately. Version-stamped to skip when current.

### Atomic File Writes

All state changes use `atomicWriteSync()` to prevent corruption on crash. Critical for the state-on-disk model.

### Lock File Crash Sentinel

`auto.lock` tracks the current unit. If the process dies, next launch detects the stale lock, reads the surviving session file, synthesizes recovery context from every tool call that made it to disk, and resumes.

### Append-Only Decision Register

`DECISIONS.md` is append-only — rows are never deleted. Decisions can be superseded but not erased. Provides full audit trail and prevents agents in fresh sessions from re-debating already-settled questions.

### Native Hot Path

Performance-critical read operations use the Rust N-API engine (libgit2 for git, ripgrep for grep, ast-grep for structural search). The dispatch hot path reads `.gsd/` state via native batch parsing.

## Key Design Decisions

### Why Fresh Sessions Instead of Continuing?

1. **Context pollution** — accumulated chat history degrades reasoning quality
2. **Optimal packing** — each session contains exactly the context needed for that unit
3. **Parallelizability** — independent sessions can run concurrently
4. **Crash safety** — losing a session loses only the current unit, not all prior work
5. **Cost efficiency** — no tokens wasted on stale context from prior phases

### Why State-on-Disk Instead of In-Memory?

1. **Crash recovery** — process can die and restart without losing progress
2. **Multi-terminal** — user can steer from a separate terminal
3. **Inspectability** — `cat .gsd/STATE.md` shows current progress
4. **Parallel workers** — multiple processes coordinate via filesystem
5. **Version control** — planning artifacts tracked in git

### Why Milestone → Slice → Task Instead of Flat Task List?

1. **Demo checkpoints** — slices produce demoable outcomes (user validation points)
2. **Context budgeting** — tasks sized to fit one context window
3. **Dependency management** — slice-level dependencies with boundary maps
4. **Clean git history** — one squash commit per milestone
5. **Adaptive replanning** — reassess roadmap after each slice based on new information

### Why Declarative Dispatch Rules?

1. **Testability** — each rule is a pure function from state to action
2. **Inspectability** — `console.log(rules)` shows the full decision tree
3. **Extensibility** — add rules by appending to array
4. **Maintainability** — 15 focused rules vs one 130-line if-else
5. **Debuggability** — rule name in logs tells you exactly why a dispatch happened

### Why Git Worktrees for Isolation?

1. **Clean separation** — milestone work doesn't touch main working directory
2. **Parallel safety** — multiple milestones in separate directories
3. **Atomic merge** — squash-merge gives one clean commit on main
4. **Easy revert** — `git revert <squash-commit>` undoes entire milestone
5. **No branch switching** — avoids invisible state changes (ADR-001)

## Configuration Philosophy

Preferences merge hierarchy (later overrides earlier):

```
Token profile defaults → Explicit preferences → Global (~/.gsd/) → Project (.gsd/) → Mode (solo/team) → Environment variables
```

Per-phase model selection means research can use a cheap model while planning uses an expensive one. Fallback chains ensure resilience. Dynamic routing optimizes within tier constraints.

## What Makes GSD-2 Different

| Problem | Typical Agent | GSD-2 |
|---------|--------------|-------|
| Context degrades over long sessions | Hope for the best | Fresh session per unit |
| Orchestration burns tokens | LLM decides what to do next | Deterministic state machine |
| Crashes lose all progress | Start over | Lock file + session forensics + resume |
| No visibility into progress | Read chat history | STATE.md + dashboard + metrics |
| No cost control | Surprise bill | Budget ceiling + profiles + enforcement |
| Quality enforcement | "Please write tests" | Mechanical verification gates |
| Git history is messy | Generic commit messages | Task-derived messages + squash merge |
| Can't handle large projects | Loses track after ~5 files | Hierarchical decomposition + boundary maps |
| No adaptive planning | Fixed plan from start | Reassess roadmap after each slice |
