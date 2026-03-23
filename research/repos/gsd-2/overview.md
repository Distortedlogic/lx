# GSD-2: Autonomous Coding Agent Framework

## Repository Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [gsd-build/gsd-2](https://github.com/gsd-build/gsd-2) |
| **Language** | TypeScript + Rust (N-API) |
| **License** | — |
| **npm Package** | `gsd-pi` |
| **Node.js** | >= 20.6.0 |
| **Version** | 2.28.0 |
| **Type** | ESM (`"type": "module"`) |
| **Build** | npm workspaces monorepo |

## What It Is

GSD-2 (Get Shit Done 2) is a standalone **coding agent CLI** built on the Pi SDK that automates software project development end-to-end. It wraps an LLM agent in a **state machine driven by files on disk**, giving it programmatic control over context windows, file management, git operations, cost tracking, and state persistence.

The original GSD (v1) was a prompt framework installed into Claude Code slash commands — it relied entirely on the LLM reading prompts and doing the right thing. GSD-2 is fundamentally different: it's a **TypeScript application that controls the agent session**.

One command, walk away, come back to a fully built project with clean git history:

```bash
npm install -g gsd-pi
gsd
/gsd auto
```

## v1 vs v2

| Aspect | v1 (Prompt Framework) | v2 (Agent Application) |
|--------|----------------------|------------------------|
| Runtime | Claude Code slash commands | Standalone CLI via Pi SDK |
| Context | Hope the LLM doesn't fill up | Fresh session per task, programmatic control |
| Auto mode | LLM self-loop with overhead | State machine reading `.gsd/` files |
| Crash recovery | None | Lock files + session forensics |
| Git | LLM writes git commands | Worktree isolation, sequential commits, squash merge |
| Cost tracking | None | Per-unit token/cost ledger with dashboard |
| Stuck detection | None | Retry once, then stop with diagnostics |
| Timeout supervision | None | Soft/idle/hard timeouts with recovery steering |
| Context injection | "Read this file" | Pre-inlined into dispatch prompt |
| Roadmap reassessment | Manual | Automatic after each slice |
| Skill discovery | None | Auto-detect and install relevant skills |
| Verification | Manual | Automated commands with auto-fix retries |
| Reporting | None | Self-contained HTML reports with metrics and DAGs |
| Parallel execution | None | Multi-worker parallel milestone orchestration |

## Core Value Proposition

GSD-2 solves a fundamental problem: **coding agents alone are not sufficient for large projects**. They lack:

1. **State management** — context windows are finite; quality degrades with accumulated garbage
2. **Actual automation** — LLM loop overhead burns context on orchestration
3. **Reproducibility** — no crash recovery or session resumption
4. **Observability** — no cost tracking, no progress dashboard, no stuck detection
5. **Quality enforcement** — no verification, no adaptive replanning, no meaningful commits

By wrapping the agent in a **state machine driven by files on disk**, GSD-2 gives it superpowers:

- **Fresh context per unit** → no quality degradation
- **Programmatic dispatch** → zero overhead orchestration
- **Crash recovery** → resume from disk state
- **Cost tracking** → full visibility and budgeting
- **Verification enforcement** → quality gates are mechanical, not behavioral
- **Meaningful commits** → clean git history
- **Adaptive replanning** → roadmap evolves as work reveals information

## Two Modes of Work

**Step Mode** (default) — `gsd` → `/gsd`:
State machine like auto mode, but pauses between units with a wizard showing what completed and what's next. Advance one step at a time, review output, continue when ready. On-ramp mode.

**Auto Mode** (autonomous) — `gsd` → `/gsd auto`:
Same state machine, but runs continuously without pausing. Fresh context per task, crash recovery, verification enforcement, meaningful commits, HTML reports — "run it and walk away."

## Tech Stack

**Languages:** TypeScript, Rust (N-API)

**Core Dependencies:**
- `@anthropic-ai/sdk` — Anthropic API
- `openai` — OpenAI API
- `@google/genai` — Google Gemini API
- `@mistralai/mistralai` — Mistral API
- `@aws-sdk/client-bedrock-runtime` — AWS Bedrock
- `@modelcontextprotocol/sdk` — MCP protocol
- `playwright` — Browser automation
- `marked` — Markdown parsing
- `yaml` — YAML parsing
- `sharp` — Image processing
- `sql.js` — In-memory SQL
- `chalk` — Terminal colors
- `@clack/prompts` — Interactive prompts

## Commands

| Command | What it does |
|---------|-------------|
| `/gsd` | Step mode — execute one unit at a time |
| `/gsd auto` | Autonomous mode — full loop |
| `/gsd next` | Explicit step mode |
| `/gsd quick` | Quick task with GSD guarantees, skip planning |
| `/gsd stop` | Stop auto mode gracefully |
| `/gsd steer` | Hard-steer plan documents during execution |
| `/gsd discuss` | Discuss architecture and decisions |
| `/gsd status` | Progress dashboard |
| `/gsd queue` | Queue future milestones |
| `/gsd prefs` | Model selection, timeouts, budget ceiling |
| `/gsd forensics` | Post-mortem investigation of failures |
| `/gsd doctor` | Runtime health checks with auto-fix |
| `/gsd export --html` | Generate HTML report for milestone |
| `/gsd capture` | Fire-and-forget thought capture |
| `/gsd triage` | Classify and resolve pending captures |
| `/gsd visualize` | 4-tab workflow overlay |
| `/gsd parallel` | Multi-worker parallel milestone control |
| `gsd headless [cmd]` | Run without TUI (CI, cron, scripts) |
| `gsd headless query` | Instant JSON snapshot (~50ms, no LLM) |
| `gsd sessions` | Interactive session picker |

**Keyboard Shortcuts:**
- `Ctrl+Alt+G` — Toggle dashboard overlay
- `Escape` — Pause auto mode
- `Ctrl+Alt+V` — Toggle voice transcription
- `Ctrl+Alt+B` — Toggle background shell overlay
