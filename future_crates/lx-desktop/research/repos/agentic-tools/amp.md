# Amp: Sourcegraph's Code-Intelligence-First Agentic Tool

Amp proves that **deep code intelligence infrastructure — the kind Sourcegraph spent a decade building for code search and navigation — provides a fundamental advantage when transformed into an agentic coding platform**. Launched in May 2025 as a research preview, Amp is now spinning out as an independent company (Amp Inc.) from Sourcegraph, with co-founders Quinn Slack and Beyang Liu leading the new entity while Dan Adler becomes CEO of Sourcegraph.

## Overview

| Metric | Value |
|--------|-------|
| **Website** | [ampcode.com](https://ampcode.com) |
| **Developer** | Amp Inc. (spun out from Sourcegraph) |
| **Launch** | May 2025 (research preview) |
| **Founders** | Quinn Slack (CEO), Beyang Liu |
| **Investors** | Craft, Redpoint, Sequoia, Goldcrest, a16z (shared with Sourcegraph) |
| **Interfaces** | CLI, VS Code, Cursor, Windsurf |
| **npm** | `@sourcegraph/amp` |
| **Install** | `npm install -g @sourcegraph/amp` |
| **Documentation** | [ampcode.com/manual](https://ampcode.com/manual) |
| **Pricing** | $10 free daily grant, then pay-per-use (no markup on LLM costs) |

## Architecture

### Agent Modes

| Mode | Model(s) | Use Case |
|------|----------|----------|
| **Smart** | Claude Opus 4.6, GPT-5.4 (unconstrained) | General-purpose, high-quality |
| **Rush** | Claude Haiku 4.5 | Fast, narrowly defined tasks |
| **Deep** | GPT-5.3 Codex (extended thinking) | Complex reasoning problems |

GPT-5.4 serves as Amp's "oracle" for the most challenging queries.

### Code Intelligence Integration

Unlike tools that treat the codebase as flat text, Amp leverages Sourcegraph's:
- **Code graph** — Symbol relationships, call hierarchies, dependency trees
- **Cross-repository search** — Keyword and semantic/natural language search across all repos
- **Reference finding** — Precise identification of all usages of a symbol
- **Live context refresh** — As code evolves, the agent refreshes context to reflect latest state

### Built-in Tools

Core tools visible via `amp tools list`:
- **Bash** — Execute shell commands with safety controls
- **codebase_search_agent** — Intelligent codebase search with AI assistant
- **create_file** — Create or overwrite files in workspace
- **Task tool** — Spawn subagents for complex tasks (each with own context window)

### Extensibility

**Toolboxes** — Extend Amp with simple scripts instead of MCP servers. On startup, Amp scans the toolbox directory and auto-discovers custom tools via `TOOLBOX_ACTION`.

**MCP Integration** — Full Model Context Protocol support. Comes pre-bundled with useful MCP servers. Skills can bundle MCP servers via `mcp.json` — servers start when Amp launches, tools hidden until skill loads.

**AGENT.md / AGENTS.md** — Project-specific files for codebase structure, development practices, coding standards, build/test commands.

### Thread Collaboration

Threads sync to ampcode.com and can be shared publicly, with teams, or kept private. Teams can reuse what works, track adoption, and improve together through shared thread history.

## Spinout (Amp Inc.)

Sourcegraph announced that Amp is becoming an independent company. Dan Adler steps in as CEO of Sourcegraph. Quinn Slack takes over as CEO of Amp Inc. The separation reflects different operational speeds: code search has a steady, infrastructure-heavy cadence; Amp moves on much faster cycles shaped by agent behavior and interaction models.

Board investors (Craft, Redpoint, Sequoia, Goldcrest, a16z) continue to serve on both companies' boards.

## Pricing

- **$10 free daily grant** for all modes and models (including Opus 4.6)
- After daily grant, usage consumes paid credits
- $10 USD in free credits upon signup
- **No markup** — LLM costs passed through directly for individuals and non-enterprise workspaces
- Enterprise plans include Zero Data Retention (ZDR) for all LLM providers

## Competitive Position

Amp's differentiation is code intelligence depth plus IDE-agnostic architecture. Unlike Cursor and Windsurf (VS Code forks), Amp isn't tied to any single editor.

| Aspect | Amp | Claude Code | Cursor |
|--------|-----|-------------|--------|
| **Code Intelligence** | Sourcegraph graph (pre-indexed) | On-demand file reads | IDE LSP |
| **Cross-repo** | Yes (Sourcegraph native) | No (single workspace) | No |
| **Models** | Opus 4.6, GPT-5.4, Haiku 4.5, GPT-5.3 Codex | Claude only | Multi-provider |
| **Interface** | CLI + VS Code + Cursor + Windsurf | CLI + IDE extensions | IDE-only |
| **Thread Sharing** | Built-in (team collaboration) | No | No |
| **Pricing** | $10/day free + pay-per-use | API costs | $20/mo subscription |
| **Enterprise** | ZDR, MCP, Sourcegraph DNA | Growing | Popular |
