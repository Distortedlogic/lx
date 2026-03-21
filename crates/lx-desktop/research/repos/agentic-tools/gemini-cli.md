# Gemini CLI: Google's Open-Source Terminal Agent with 1M-Token Context

Gemini CLI demonstrates that **an open-source terminal agent backed by a free tier with generous rate limits can achieve near-100K GitHub stars by removing every friction point between developers and frontier models**. Built in TypeScript, Apache 2.0 licensed, and offering 60 requests/minute free with a personal Google account, it is the most accessible entry point to agentic coding — and the highest-starred coding CLI on GitHub.

## Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [google-gemini/gemini-cli](https://github.com/google-gemini/gemini-cli) |
| **Stars** | 98,469 |
| **Forks** | 12,453 |
| **Language** | TypeScript |
| **License** | Apache 2.0 |
| **Created** | April 17, 2025 |
| **Latest Release** | v0.34.0 (March 17, 2026) |
| **Default Model** | Gemini 3 Pro (1M token context) |
| **Free Tier** | 60 req/min, 1,000 req/day (personal Google account) |
| **Install** | `npx @google/gemini-cli` or `brew install gemini-cli` |
| **Topics** | ai, ai-agents, cli, gemini, gemini-api, mcp-client, mcp-server |

## Architecture

### Source Structure

```
packages/core/src/
├── agent/          # Agent loop
├── agents/         # Multi-agent support
├── billing/        # Rate limiting, credit tracking
├── code_assist/    # Code understanding
├── commands/       # Slash commands
├── config/         # Configuration management
├── core/           # Core runtime
├── hooks/          # Lifecycle hooks
├── mcp/            # MCP client/server integration
├── policy/         # Security policies
├── routing/        # Model routing
├── safety/         # Content safety
├── sandbox/        # Sandboxing implementations
├── scheduler/      # Task scheduling
├── skills/         # Skills system
├── tools/          # Built-in tools
└── voice/          # Voice input
```

### Built-in Tools

- **File System Operations** — Read, write, edit files
- **Shell Commands** — Execute terminal commands with `!` shorthand
- **Web Fetch & Search** — Google Search grounding, web fetching
- **File Access** — `@` shorthand for including file content in prompts

### GEMINI.md Context System

Hierarchical context file system, analogous to CLAUDE.md:

1. **Global** — `~/.gemini/GEMINI.md` (default instructions for all projects)
2. **Workspace** — `GEMINI.md` in workspace directories and parent directories
3. **JIT (Just-in-Time)** — Auto-discovered when tools access files in directories with `GEMINI.md`

Supports modular imports with `@file.md` syntax to break large context files into components.

Memory management via `/memory` command: `show`, `reload`, `add <text>`.

## Sandboxing

Platform-specific isolation with four methods:

| Method | Platform | Description |
|--------|----------|-------------|
| **macOS Seatbelt** | macOS | `sandbox-exec` with `permissive-open` profile (restricts writes outside project) |
| **Container (Docker/Podman)** | Cross-platform | Complete process isolation |
| **Windows Native** | Windows | `icacls` with Low Mandatory Level integrity |
| **gVisor (runsc)** | Linux | Strongest isolation — user-space kernel intercepts all syscalls |

Configuration: `GEMINI_SANDBOX=runsc` or in settings as `sandbox: "runsc"`.

## Extensions & Custom Commands

- **Custom Commands** — Create reusable slash commands
- **MCP Server Integration** — Both client and server support
- **Custom Extensions** — Build and share tool extensions

## Key Features

- **Checkpointing** — Save and resume conversations
- **Token Caching** — Optimize token usage
- **Headless Mode** — Non-interactive scripting with `-p` flag
- **JSON Output** — `--output-format json` for structured output, `--output-format stream-json` for streaming
- **Multi-directory** — `--include-directories` to add context from multiple paths
- **Model Selection** — `-m gemini-2.5-flash` to switch models

## Release Cadence

- **Preview** — Weekly on Tuesdays (UTC 23:59)
- **Stable** — Regular stable releases
- **Nightly** — Daily nightly builds

## Competitive Position

Gemini CLI's 98K stars (vs Claude Code's closed-source, Codex's 66K, OpenCode's 95K) reflects the power of free-tier access. The combination of Apache 2.0 license, Google Search grounding, and 1M-token context makes it the default recommendation for developers who want a free, open-source terminal agent.

| Aspect | Gemini CLI | Claude Code | Codex CLI |
|--------|------------|-------------|-----------|
| **License** | Apache 2.0 | Proprietary | Apache 2.0 |
| **Stars** | 98,469 | N/A | 66,539 |
| **Language** | TypeScript | TypeScript | Rust |
| **Free Tier** | 60 req/min | No | ChatGPT Plus required |
| **Context** | 1M tokens | 200K (Sonnet) / 1M (Opus) | Varies |
| **Search** | Google Search grounding | WebSearch tool | No built-in |
| **Sandbox** | 4 methods (Seatbelt, Docker, Windows, gVisor) | Basic | 3 methods (Seatbelt, Landlock, Windows) |
