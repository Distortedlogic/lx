# OpenAI Codex CLI: Rust-Built Terminal Agent with Platform-Native Sandboxing

Codex CLI demonstrates that **a Rust-native terminal agent with OS-level sandboxing and a specialized patch format can achieve security-first autonomous coding while maintaining the developer-in-the-loop experience**. Built by OpenAI, written in Rust (95.6% of codebase), and open-sourced under Apache 2.0, Codex CLI pairs GPT-5.x models with a two-layer security model (sandbox enforcement + approval policy) that lets it operate autonomously within defined boundaries.

## Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [openai/codex](https://github.com/openai/codex) |
| **Stars** | 66,539 |
| **Forks** | 8,876 |
| **Language** | Rust (95.6%), Python (2.3%), TypeScript (1.3%) |
| **License** | Apache 2.0 |
| **Created** | April 13, 2025 |
| **Build System** | Bazel |
| **Default Model** | GPT-5.3-Codex |
| **Auth** | ChatGPT account (Plus/Pro/Team/Edu/Enterprise) or API key |
| **Platforms** | macOS, Linux, Windows (experimental/WSL) |
| **Install** | `npm install -g @openai/codex` or `brew install --cask codex` |

## Architecture

### Codebase Structure

```
codex/
├── codex-cli/      # CLI frontend
├── codex-rs/       # Rust core library
├── sdk/            # Developer SDK
└── mcp/            # Shell tool MCP integration
```

### Two-Layer Security Model

Codex separates enforcement from policy:

1. **Sandbox Mode** — What Codex can technically do (filesystem access, network capability)
2. **Approval Policy** — When Codex must ask permission before acting

### Sandbox Modes

| Mode | Behavior | Protected |
|------|----------|-----------|
| **read-only** | Inspect files only, no edits or commands without approval | `.git`, `.agents`, `.codex` always read-only |
| **workspace-write** (default) | Read/edit within workspace, run local commands | Network off by default |
| **danger-full-access** | No filesystem or network boundaries | Use sparingly |

### Platform-Native Sandboxing

| Platform | Technology |
|----------|-----------|
| **macOS** | Seatbelt (`sandbox-exec`) |
| **Linux** | bubblewrap + seccomp (Landlock) |
| **Windows** | Restricted Token / Native implementation |

Sandbox constraints apply to spawned commands — inherited restrictions affect `git`, package managers, and all subprocess tools.

### Approval Policies

| Level | Flag | Behavior |
|-------|------|----------|
| **Auto** (default) | `--full-auto` | Reads/edits within workspace; asks for external or network |
| **Safe Read-Only** | `--sandbox read-only --ask-for-approval on-request` | Only reads; asks for everything else |
| **Full Access** | `--yolo` / `--dangerously-bypass-approvals-and-sandbox` | No sandbox, no approvals |

## V4A Patch Format

Codex uses a specialized diff format (`apply_patch`) that models are trained to produce:

- **Operation declarations** — Add/Update/Delete file sections
- **Context markers** — `@@ function_name` instead of line numbers
- **Prefix indicators** — Space (context), minus (remove), plus (add)
- **Progressive fuzzy matching** — Exact → trimmed whitespace → all whitespace removed

The system prompt teaches a shell-first toolkit (`cat`, `grep`, `find`, `git`) and reserves file mutation for the strict `apply_patch` envelope, pushing toward minimal surgical diffs.

## AGENTS.md Configuration

Hierarchical instruction system:

1. **Global** — `~/.codex/AGENTS.md` or `AGENTS.override.md`
2. **Project** — Git root down to current directory
3. **Merge Order** — Concatenate root-down; closer files override
4. **Override** — `AGENTS.override.md` takes priority over `AGENTS.md` at same level
5. **Max size** — 32 KiB combined (configurable via `project_doc_max_bytes`)
6. **Fallback names** — Configurable via `project_doc_fallback_filenames`

Also supports `TEAM_GUIDE.md` and `.agents.md` as alternative filenames.

## Surfaces

| Surface | Description |
|---------|-------------|
| **CLI** | Terminal-native with interactive TUI |
| **Desktop App** | macOS app (launched February 2026), invoked via `codex app` |
| **IDE Extensions** | VS Code, Cursor, Windsurf |
| **Cloud/Web** | `chatgpt.com/codex` — isolated OpenAI containers |

### Cloud Architecture

Two-phase runtime:
1. **Setup Phase** — Network access for installing dependencies
2. **Agent Phase** — Offline by default, sandboxed execution

## Benchmarks

| Benchmark | GPT-5.3-Codex | Claude Code |
|-----------|---------------|-------------|
| **Terminal-Bench 2.0** | 77.3% | 65.4% |
| **Code review / edge cases** | Stronger | Faster generation |

Developer consensus: Codex catches logical errors, race conditions, and edge cases better. Claude Code generates features faster. The hybrid workflow — Claude generates, Codex reviews — is increasingly common.

## Competitive Position

Codex CLI's Rust-native architecture gives it speed and memory safety advantages. Its sandboxing depth (platform-native enforcement on every OS) is the most sophisticated among CLI agents. The main trade-off is model lock-in to OpenAI and the requirement for a ChatGPT subscription.
