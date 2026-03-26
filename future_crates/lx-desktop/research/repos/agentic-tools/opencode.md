# OpenCode & Crush: The Open-Source Terminal Agent Ecosystem

OpenCode and its upstream successor Crush prove that **an open-source, terminal-native AI coding agent can achieve parity with proprietary tools by combining Go's performance, Bubble Tea's TUI framework, and multi-provider model access into a polished developer experience**. What started as a side project from the SST (Serverless Stack) team became the most popular open-source coding CLI, spawning a complex lineage: the original author moved the project to Charm Bracelet as Crush, while SST continued maintaining their fork as OpenCode under the Anomaly organization.

## Overview

There are actually **three distinct projects** sharing the name/lineage:

| Project | GitHub | Stars | Language | Status |
|---------|--------|-------|----------|--------|
| **Original** | `opencode-ai/opencode` | 11,499 | Go | Archived (Sep 2025) |
| **Crush** | `charmbracelet/crush` | 21,699 | Go | Active (original author) |
| **OpenCode (Anomaly)** | `anomalyco/opencode` | 126,000 | TypeScript/Rust | Active (dominant fork) |

| Metric | Value |
|--------|-------|
| **Website** | [opencode.ai](https://opencode.ai) |
| **Monthly Users** | 650,000+ (within 5 months of launch) |
| **Interfaces** | TUI, Desktop app (Tauri), VS Code extension, Web |
| **License** | MIT |
| **Founded by** | Anomaly (SST team: Jay V (CEO), Frank Wang (CTO), Dax Raad, Adam Elmore) |
| **Revenue** | OpenCode Zen (hosted models) |

## Lineage and History

The OpenCode ecosystem has a complex but important history:

1. **SST Origins** — The Anomaly team (formerly Serverless Stack/SST) built terminal-first UIs for SST and launched Terminal, a coffee subscription for the terminal that generated $100K+ in its first year
2. **OpenCode Launch** — June 19, 2025, by Jay, Frank, Dax Raad, and Adam Elmore
3. **Crush Fork** — The original author moved to Charm Bracelet, renaming the project to Crush (July 29, 2025)
4. **SST/Anomaly Fork** — SST continued maintaining their fork as opencode-ai/opencode under the Anomaly organization
5. **Antigravity Auth Bridge** — The community-built `opencode-antigravity-auth` project (9,697 stars) enables OpenCode to piggyback on Antigravity's free tier for model access

## Architecture

### Bubble Tea TUI (Model-View-Update)

OpenCode's terminal interface is built on Charm Bracelet's Bubble Tea framework, implementing the Elm Architecture pattern:

- **appModel** — Central orchestrator implementing `tea.Model` with `Init()`, `Update()`, `View()` methods
- **Component-based** — Distinct components for chat, status display, logging, and interactive dialogs
- **Editor** — Vim-like text input with integrated editing capabilities
- **Pages** — Multiple view pages (chat, logs, sessions) navigated via keyboard shortcuts

### Dual-Process Architecture

Running `opencode` launches two processes:

1. **JS HTTP Server** — Backend handling AI provider integration, tool execution, conversation state
2. **Go TUI** — Frontend rendering the terminal interface, sending prompts, displaying results

### Storage

- **SQLite** — Persistent storage for conversations and session history
- **Session Management** — Save, resume, and share multiple conversation sessions via links

## Key Features

### Multi-Provider Model Access

Supports 75+ LLM providers out of the box. Users can switch models mid-conversation. Notable integrations include direct OpenAI, Anthropic, Google, AWS Bedrock, Groq, Azure OpenAI, and OpenRouter.

### Multi-Session Support

Multiple parallel agents can run on the same project simultaneously, each maintaining its own conversation history. This is a clear advantage over Aider for complex projects requiring parallel workstreams.

### Language Server Protocol (LSP)

IDE-level intelligence in the terminal — the AI gets access to type information, symbol definitions, and real-time diagnostics from language servers.

### Tool Integration

AI can execute shell commands, search files, and modify code directly. Built-in tool framework with extensibility via MCP.

### Plan Mode

Structured planning before execution, similar to Claude Code's plan mode.

### Privacy-First

Stores no code or context data beyond local SQLite sessions.

## Crush (Charm Bracelet)

Crush is the upstream continuation by the original author at Charm Bracelet. It combines Go's speed with Charm's design philosophy — the same team that built Bubble Tea, Lip Gloss, and the broader Charm ecosystem of terminal tools.

The relationship: `anomalyco/opencode` (SST fork) and `charmbracelet/crush` (original author's continuation) share a common ancestor but have diverged in features and direction.

## Competitive Position

| Aspect | OpenCode | Claude Code | Aider |
|--------|----------|-------------|-------|
| **Source** | Open (MIT) | Proprietary | Open (Apache 2.0) |
| **Language** | Go | TypeScript | Python |
| **TUI** | Bubble Tea (polished) | Ink (React-like) | Basic readline |
| **Multi-session** | Yes (parallel) | Sub-agents | No |
| **LSP** | Yes | Deferred tools | No |
| **Model Lock-in** | None (75+ providers) | Claude only | Any (via litellm) |
| **Repo Map** | No | Internal | Yes (tree-sitter + PageRank) |

OpenCode's primary advantage is freedom from model lock-in combined with a polished TUI experience. Its disadvantage is lacking Claude Code's deep model-tool co-training and Aider's repo map technique.
