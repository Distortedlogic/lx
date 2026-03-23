# Emerging Agentic Tools: Zed, Cline, Roo Code, Trae, Void, PearAI, Qodo, and Others

This document covers the second tier of agentic coding tools — projects that are either rising fast, serve niche needs, or represent interesting architectural bets that haven't yet achieved the scale of the top-tier tools.

## Zed

### Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [zed-industries/zed](https://github.com/zed-industries/zed) |
| **Stars** | 76,200+ |
| **Language** | Rust |
| **Funding** | $32M (Sequoia) |
| **Creators** | Creators of Atom and Tree-sitter |
| **Framework** | GPUI (custom GPU-accelerated UI framework) |

### Architecture

Zed is written entirely in Rust with GPUI, a custom GPU-accelerated rendering framework. This makes it the fastest code editor available — not an AI tool with an editor bolted on, but a performance-first editor with AI capabilities added.

### Agent Panel

- **Slash Commands** — `/file`, `/diagnostics`, `/prompt` for explicit context management
- **MCP Integration** — MCP server prompts appear in the slash command menu
- **Profiles** — Tool grouping: Write (edit + terminal), Ask (read-only), Minimal
- **External Agents** — Claude Code, Gemini CLI, and Codex can run within Zed
- **Thread Management** — Auto-generated titles, editable messages, full conversation history

### Competitive Position

Zed's bet is that raw editor performance matters even in the AI era. For developers who find VS Code forks too slow, Zed offers native-speed editing with growing AI capabilities. The risk is that AI features remain behind the VS Code fork ecosystem.

---

## Cline

### Overview

| Metric | Value |
|--------|-------|
| **Website** | [cline.bot](https://cline.bot) |
| **GitHub** | [cline/cline](https://github.com/cline/cline) |
| **VS Code Installs** | 5M+ |
| **License** | Open source |

### Key Features

- **Autonomous Agent** — Creates/edits files, executes commands, uses browser with permission at each step
- **Plan/Act Modes** — Plan first, then execute
- **MCP Integration** — First-class MCP support for external tools
- **Zero Model Markup** — Developers pay only API costs with full provider flexibility
- **CLI 2.0** — Terminal agent control plane (launched 2026)
- **Enterprise** — Team features available

### Competitive Position

Cline's moat is cost transparency. While Cursor charges $20/month with opaque credit systems, Cline passes through raw API costs. For teams that want AI coding without vendor lock-in on both model and pricing, Cline is the default VS Code choice.

---

## Roo Code

Roo Code (formerly Roo-Cline) is a fork of Cline with additional features:
- **Custom Modes** — Define specialized agent behaviors
- **Enhanced Model Flexibility** — Same broad provider support as Cline
- **MCP Support** — Full Model Context Protocol integration
- Diverged from Cline to pursue different feature priorities

---

## Trae (ByteDance)

### Overview

| Metric | Value |
|--------|-------|
| **Website** | [trae.ai](https://www.trae.ai) |
| **Developer** | ByteDance |
| **Platform** | VS Code fork |
| **Pricing** | Free (unlimited access to DeepSeek R1, Claude Sonnet) |

### Key Features

- **Builder Mode** — Describe a project in natural language, Trae generates the entire codebase
- "Think-before-doing" approach in Builder mode
- **Free Model Access** — Unlimited DeepSeek R1 and Claude 3.7/4.x Sonnet

### Privacy Controversy

Trae was caught collecting user telemetry data even after users opted out:
- ByteDance's data collection continues despite opt-out settings
- Extensive telemetry system documented by security researchers
- Multiple reports from The Register, CyberNews, and security firms
- **Red flag for enterprise or privacy-sensitive use cases**

---

## Void

### Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [voideditor/void](https://github.com/voideditor/void) |
| **Stars** | 28,400+ |
| **Platform** | VS Code fork |
| **Status** | **Development paused** (late 2025) |

### Architecture

Open-source Cursor alternative with:
- AI chat, inline editing, autocomplete
- Any model support, including local models
- Full data control — doesn't send code to external servers
- One-click VS Code settings/themes/keybinds migration

### Current Status

Development officially paused as of late 2025. The team is "exploring novel coding ideas." The codebase remains functional as a reference implementation. Community can build and maintain their own versions.

---

## PearAI

### Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [trypear/pearai-app](https://github.com/trypear/pearai-app) |
| **Platform** | VS Code fork + Continue fork |
| **Pricing** | $15/month (freemium) |
| **Backed by** | Y Combinator |

### Controversy

PearAI forked Continue (open-source AI coding assistant) and was criticized for essentially cloning an existing open-source project, receiving YC funding, and not clearly attributing the original work. TechCrunch and Hacker News covered the controversy extensively.

### Features

- AI chat, PearAI Creator, AI debugging
- Context awareness from codebase
- VS Code fork with all extensions compatible

---

## Qodo (formerly CodiumAI)

### Overview

| Metric | Value |
|--------|-------|
| **Website** | [qodo.ai](https://www.qodo.ai) |
| **GitHub** | [qodo-ai/pr-agent](https://github.com/qodo-ai/pr-agent) |
| **Founded** | 2022 (by Itamar Friedman, Dedy Kredo) |
| **Team** | ~100 people (Israel, U.S.) |

### Focus

Qodo focuses on code quality and integrity rather than code generation:
- **PR Agent** — Open-source automated PR review
- **AI Code Review** — Bug detection, security analysis
- **Test Generation** — Automated test creation
- **Quality Gates** — AI-powered quality enforcement in CI/CD

---

## Continue.dev

### Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [continuedev/continue](https://github.com/continuedev/continue) |
| **Stars** | 26,000+ |
| **Platform** | VS Code + JetBrains extensions |
| **License** | Open source |

### Features

- Open-source AI coding assistant
- Source-controlled AI checks enforceable in CI
- CLI for automated checks
- Flexibility in model choice and data control
- The foundation that PearAI forked

---

## Supermaven

### Overview

| Metric | Value |
|--------|-------|
| **Website** | [supermaven.com](https://supermaven.com) |
| **Status** | **Sunsetting** (announced on Hacker News) |
| **Context** | 1M token context window |
| **Latency** | Sub-10ms completions |

### Key Innovation

Supermaven's 1M token context window and sub-10ms latency were its differentiators. It focused purely on code completion speed rather than agentic capabilities. The sunsetting suggests that pure completion tools without agentic features face an increasingly difficult market.

---

## Tabnine

### Overview

| Metric | Value |
|--------|-------|
| **Website** | [tabnine.com](https://www.tabnine.com) |
| **Focus** | Enterprise code completion with total control |
| **IDE Support** | VS Code, JetBrains, and more |

### Features

- Enterprise Context Engine for codebase understanding
- AI agents with enterprise control
- On-premise deployment options
- Code generation from natural language comments
- Focus on enterprise security and compliance
