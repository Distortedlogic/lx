# OpenCode: Open-Source Terminal Coding Agent

## Identity and Lineage

OpenCode is an open-source, terminal-based AI coding agent. Two distinct projects share the name, which matters for understanding the design space.

**Original (Go version, now Charm's Crush):** Built March 2025 by Kujtim Hoxha as "TermAI," later renamed OpenCode. Pure Go, monolithic binary, Bubble Tea TUI. 11,566 stars at archival. Charm hired Hoxha, moved the repo to `charmbracelet/crush` (21,923 stars, custom license). The Go architecture was clean and self-contained: agent loop, provider abstraction, SQLite persistence, LSP client, permission system, and TUI all in one binary.

**Current SST version (TypeScript + Go):** Forked April 2025 by Dax Raad and Adam Elmore of SST (Serverless Stack). 129,608 stars. Rewrote the backend in TypeScript on Bun, kept the Go TUI initially, later migrated the UI to SolidJS-based terminal rendering (`@opentui/solid`). This is the version at `opencode.ai` and the one the community refers to as "OpenCode" today.

The split was contentious. Charm ceded the name. The SST fork took the domain, the AUR package, and the momentum.

## Architecture

### Original Go Version

Monolithic Go binary with a modular internal structure:

```
internal/
  llm/agent/      -- ReAct loop, sub-agent spawning, MCP tool integration
  llm/models/     -- Model definitions with costs and context window sizes
  llm/provider/   -- Provider abstraction (Anthropic, OpenAI, Gemini, Bedrock, etc.)
  llm/tools/      -- bash, edit, glob, grep, ls, view, write, patch, fetch, sourcegraph, diagnostics
  tui/            -- Bubble Tea terminal UI
  lsp/            -- Language Server Protocol client
  session/        -- SQLite-backed session management
  permission/     -- Approve/deny tool calls
  pubsub/         -- Event system for agent<->TUI communication
```

The agent loop is a standard ReAct cycle: stream LLM response, check for tool calls, execute tools, append results, repeat until the model returns a final answer without tool use.

### SST Version (Current)

Client-server architecture:

- **Server:** JavaScript on Bun runtime, HTTP via Hono framework. Handles all AI interaction, tool execution, session persistence.
- **TUI Client:** Originally Go/Bubble Tea, later migrated to SolidJS-based terminal rendering (`@opentui/solid`). Communicates with the server via HTTP and Server-Sent Events (SSE).
- **Packaging:** `bun build --compile` bundles JS server + Bun runtime + TUI into a single executable.

The TUI is a thin client; all business logic lives in the server. The agent loop in the SST version uses Vercel's AI SDK for orchestration. `SessionPrompt.loop()` runs tool calls via `Tool.execute()`, persists state to `sessions.db` (SQLite via Drizzle ORM), and streams updates via SSE.

## Model and Provider Support

The Go version supported ~15 providers: Anthropic (Claude 3/3.5/3.7/4 families), OpenAI (GPT-4o/4.1/4.5, O1/O3/O4), Google Gemini, AWS Bedrock, Groq, Azure OpenAI, VertexAI, GitHub Copilot, OpenRouter, and local models via configurable endpoints.

The SST version expanded to 75+ providers via Vercel AI SDK: everything above plus Vercel AI Gateway, MiniMax, Hugging Face Inference, Cerebras, io.net, and Ollama for fully local/offline usage.

## Tool System

| Tool | Description |
|------|-------------|
| `bash` | Shell command execution (permission-gated) |
| `edit` | Search-and-replace file editing via unique string matching |
| `write` | Write/overwrite entire files |
| `patch` | Apply multi-file unified diffs atomically |
| `view` | Read file contents with offset/limit |
| `glob` | File pattern matching |
| `grep` | Content search via regex |
| `ls` | Directory listing |
| `fetch` | URL fetching |
| `sourcegraph` | Cross-repo code search |
| `diagnostics` | LSP diagnostics for files |
| `agent` | Spawn read-only sub-agent |

### Sub-Agent System

The `agent` tool spawns read-only "Task Agents" with access limited to `glob`, `grep`, `ls`, `sourcegraph`, and `view`. Cannot modify files. Multiple sub-agents run concurrently. Each gets its own session and context window. This isolates exploratory search from the main agent's context budget.

### MCP Integration

Full Model Context Protocol support via `mcp-go` (original) and native MCP client (SST). Supports `stdio`, `http`, and `sse` transports. MCP tools appear alongside built-in tools with the same permission model.

### LSP Integration

Launches real Language Server Protocol servers (gopls, typescript-language-server, etc.) for real-time diagnostics after file edits. The SST version auto-detects and starts LSP servers for 40+ languages. The AI agent receives type errors and diagnostic information in the same turn it makes an edit.

## Context Management

**Auto-compact threshold:** At 95% of the model's context window, triggers automatic summarization. Creates a summary message and continues the session with compressed context. Graduated reduction strategy: first prune tool outputs, then strip media attachments, then full conversation summarization.

**Session persistence:** SQLite-backed. Multiple conversation sessions per project. Session switching via Ctrl+A. Cost tracking at token-level granularity.

## Operating Modes

**Build Mode (default):** All tools enabled, full read/write access.

**Plan Mode:** Edit tools disabled except for `.opencode/plans/*.md`. Read-only analysis and planning without risk of unintended mutations. Novel pattern not present in Claude Code or Aider.

**Non-interactive/headless:** `opencode -p "prompt" -f json -q` runs a single prompt, prints output, exits. Auto-approves all permissions. Useful for CI/CD and scripting.

## Permission Model

Every destructive tool call (bash, edit, write, patch) requires user approval. Three options: allow once, allow for session (persistent grant), deny.

## Context Files and Configuration

Reads project context from `.github/copilot-instructions.md`, `.cursorrules`, `CLAUDE.md`, `opencode.md`, and other convention files.

**Skills system (SST version):** Reusable instruction files discovered from repo or home directory. Community-driven skill sharing.

**Custom commands:** User-defined and project-scoped commands stored as Markdown with named argument placeholders (`$ISSUE_NUMBER`, `$AUTHOR_NAME`).

**GitHub integration (SST version):** Mention `/opencode` or `/oc` in GitHub comments to execute tasks within GitHub Actions runners.

## Hashline Edit Innovation

The SST version introduced "Hashline" -- a hash-anchored edit mechanism where every line the agent reads is tagged with a content hash. The agent edits by referencing hash tags rather than searching for unique strings. This eliminates stale-line errors common in traditional search-and-replace editing, where file modifications between read and edit cause match failures.

## Security Incident

**CVE-2026-22812 (CVSS 8.8):** OpenCode's unauthenticated HTTP server on localhost had permissive CORS headers, allowing malicious websites to trigger command execution via cross-origin requests. Fixed in version 1.0.216 with authentication controls. The client-server architecture created an attack surface that monolithic CLI tools like Claude Code and Pi avoid by design.

## Anthropic Block Incident (January 2026)

Anthropic blocked OpenCode and other third-party tools from using Claude via consumer OAuth tokens. OpenCode had been sending headers spoofing the Claude Code client identity. DHH called it "very customer hostile." Dax Raad's response: "It's their business; they have the right to enforce their terms however they like." OpenAI responded by officially partnering with OpenCode, extending subscription support. OpenCode gained 18,000 GitHub stars in two weeks during the controversy.

## Relevance to lx

**Client-server split tradeoffs:** OpenCode's Bun server + thin TUI client enables multiple client types (TUI, desktop, web, Discord bot) against a shared backend. But it introduced a localhost attack surface (CVE-2026-22812) that monolithic designs avoid. lx's harness layer should consider whether multi-client support justifies the surface area.

**Plan/Build mode separation:** A structurally enforced read-only planning mode that disables mutation tools is a pattern lx could adopt natively. In lx terms, this maps to a `mode` block or annotation on `agent` definitions that restricts which tools are bound.

**Hashline editing:** Hash-anchored file editing eliminates a common failure mode in agentic coding. If lx provides file-editing primitives to agents, hash-anchoring is worth incorporating.

**LSP-as-agent-context:** Feeding real-time type errors and diagnostics from language servers directly into the agent's tool results is a powerful closed-loop pattern. lx programs that orchestrate coding agents could expose an `lsp_diagnostics` tool that wraps this interaction.

**Sub-agent isolation model:** Read-only sub-agents with restricted tool sets running on independent context windows is exactly the kind of structured concurrency pattern lx should make first-class. The `agent` tool in OpenCode maps cleanly to lx's `spawn` with a capability-restricted tool binding.

**75+ provider abstraction:** The breadth of provider support (via Vercel AI SDK) shows the value of a thin, unified LLM abstraction layer. lx's backend trait system serves this role but should remain provider-agnostic and thin.