# Browser Automation for AI Agents: Playwright MCP vs Browser Use vs Agent Browser

The browser automation landscape for AI agents has fragmented into three distinct approaches: **accessibility-tree snapshots** (Playwright MCP, agent-browser), **full autonomous agent loops** (Browser Use), and **hybrid code+AI** (Stagehand/Browserbase). The key trade-off is token efficiency vs autonomy — structured snapshot tools use 4x fewer tokens but require more agent orchestration, while autonomous frameworks burn tokens freely but handle complex multi-step tasks without external coordination.

## Overview Matrix

| Dimension | Playwright MCP | Browser Use | Agent Browser (Vercel) | Stagehand (Browserbase) |
|-----------|---------------|-------------|----------------------|------------------------|
| **Developer** | Microsoft | Browser Use (Gregor Zunic) | Vercel Labs | Browserbase, Inc. |
| **GitHub** | [microsoft/playwright-mcp](https://github.com/microsoft/playwright-mcp) | [browser-use/browser-use](https://github.com/browser-use/browser-use) | [vercel-labs/agent-browser](https://github.com/vercel-labs/agent-browser) | [browserbase/stagehand](https://github.com/browserbase/stagehand) |
| **Stars** | ~29.8K | ~84.7K | ~25.2K | ~21.7K |
| **Created** | March 2025 | October 2024 | January 2026 | March 2024 |
| **Language** | TypeScript | Python | Rust CLI + Node.js | TypeScript (+ Python, Go, Ruby) |
| **License** | Apache 2.0 | MIT | Apache 2.0 | MIT |
| **Approach** | MCP server, structured tools | Autonomous agent loop | CLI commands for AI tool-use | Hybrid code+AI with caching |
| **Page Understanding** | Accessibility tree snapshots | DOM + accessibility tree + optional vision | Accessibility snapshots (ref-based) | Context builder (DOM-aware) |
| **Token Cost** | ~114K/task (MCP), ~27K (CLI) | High (full reasoning per step) | Low (ref-based, minimal) | Decreasing (auto-cache converges) |
| **MCP Support** | Is an MCP server | Both client and server | Community wrapper | Official MCP server |
| **Cloud Option** | No | Browser Use Cloud | No | Browserbase ($20-99/mo) |
| **Funding** | Microsoft | Undisclosed (VC-backed) | Vercel Labs | $67.5M ($300M valuation) |

## Architecture Comparison

### Playwright MCP — Structured Snapshot Server

Three-layer architecture: MCP Client (AI tool) → MCP Server (`@playwright/mcp`) → Browser (Chromium/Firefox/WebKit).

The server translates MCP `callTool` requests into Playwright API calls. Page state is returned as **accessibility tree snapshots** — the same semantic tree screen readers use — serialized into YAML-style text with unique `ref` identifiers per element. Incremental mode (default) sends only diffs after the first full snapshot.

70+ tools organized into capability groups: core automation (always on), plus opt-in caps for vision, storage, network, devtools, PDF, and testing. Transport: STDIO (local), HTTP/SSE (remote), or WebSocket (browser extension bridge).

Key innovation: **codegen mode** — automatically generates Playwright TypeScript test code as the agent interacts, producing reproducible scripts as a side-effect.

### Browser Use — Full Autonomous Agent

Architecture stack: `cdp-use` (custom CDP client, replaced Playwright dependency) → DOM/accessibility service → serialized element tree → agent loop → LLM.

The agent loop takes a task string, constructs browser state (text representation of interactive elements + optional screenshots with bounding boxes), sends to LLM, parses structured action output, executes via CDP, repeats. The browser session persists between agent steps.

Page understanding is multi-modal: CDP `GetFullAXTree` enriched into `EnhancedAXNode` objects, DOM traversal with computed styles for visibility detection, `DOMTreeSerializer` producing indexed XML (`[33]<div /> [35]<input type=text />`), plus optional screenshots as "ground truth."

Key innovation: **dual client/server MCP** — can both consume external MCP tools and expose itself as an MCP server.

### Agent Browser (Vercel) — CLI-First Ref-Based Interaction

Rust CLI for command parsing + Node.js daemon with Playwright for browser control. The daemon persists between commands for sub-100ms operation latency.

Uses accessibility snapshots with **ref-based element selection** — AI-friendly semantic identifiers rather than structural selectors. Workflow: `open <url>` → `snapshot -i` (get interactive refs) → interaction commands.

Three browser modes: headless Chromium, real Chrome with profile support, cloud-hosted remote browsers. Includes an **authentication vault** (locally encrypted, LLM never sees passwords) and a local web dashboard for real-time session monitoring.

Key innovation: **token efficiency** — reportedly the lowest token consumption of any browser automation tool, by keeping the interaction model minimal and ref-based.

### Stagehand (Browserbase) — Hybrid Code+AI

In v3, Stagehand talks directly to browsers via CDP with a modular driver system (Playwright, Puppeteer, or any CDP driver). Four core primitives: `act()` (natural-language actions), `extract()` (structured data via Zod schemas), `observe()` (discover available actions), `agent()` (autonomous multi-step workflows).

Key innovation: **auto-caching with self-healing**. When an AI action succeeds, Stagehand records the selector path and replays deterministically on subsequent runs. If replay fails (DOM changed), AI re-engages and updates the cache. Workflows converge toward Playwright-level speed/cost while retaining AI adaptability.

## Token Economics

This is the decisive trade-off for lx integration. Browser automation is one of the most token-hungry operations an agent can perform.

| Tool | Tokens per Typical Task | Why |
|------|------------------------|-----|
| **Agent Browser CLI** | ~15-20K (estimated) | Minimal ref-based snapshots, no schema overhead |
| **Playwright CLI** | ~27K | Snapshot/screenshot to disk, agent reads only what it needs |
| **Playwright MCP** | ~114K | 13.7K for tool schemas alone, full snapshot per step |
| **Browser Use** | Variable, high | Full DOM + optional vision per reasoning step |
| **Stagehand** | Decreasing over time | First run expensive, subsequent runs use cached selectors |

The MCP protocol itself adds overhead: tool schema definitions (~13.7K tokens) load at session start regardless of usage. Three MCP servers can consume 143K of a 200K-token context window before the agent reads its first user message.

## Integration with AI Coding Agents

All four tools support the major AI coding tools, but through different mechanisms:

| AI Tool | Playwright MCP | Browser Use | Agent Browser | Stagehand |
|---------|---------------|-------------|---------------|-----------|
| **Claude Code** | `claude mcp add` | MCP server mode | Direct CLI | MCP server |
| **Cursor** | MCP config | MCP server | CLI | MCP config |
| **VS Code Copilot** | MCP config | — | CLI | MCP config |
| **Codex** | MCP config | — | CLI | — |
| **Gemini CLI** | MCP config | — | CLI | — |

## LLM Provider Support

| Provider | Playwright MCP | Browser Use | Stagehand |
|----------|---------------|-------------|-----------|
| **OpenAI** | Any (model-agnostic) | Yes (`ChatOpenAI`) | Yes |
| **Anthropic** | Any | Yes (`ChatAnthropic`) | Yes |
| **Google** | Any | Yes (`ChatGoogle`) | Yes |
| **Azure** | Any | Yes (`ChatAzureOpenAI`) | — |
| **AWS Bedrock** | Any | Yes (`ChatAWSBedrock`) | — |
| **Ollama** | Any | Yes (`ChatOllama`) | Yes (open-source models) |
| **DeepSeek** | Any | Yes (dedicated module) | — |
| **Browser Use hosted** | — | Yes (`ChatBrowserUse` $0.20/M input) | — |

Playwright MCP and Agent Browser are model-agnostic by design — they're tools, not agents. The LLM choice is the caller's concern.

## Benchmarks

| Benchmark | Browser Use | Stagehand | Playwright MCP | Agent Browser |
|-----------|-------------|-----------|---------------|---------------|
| **WebVoyager** | 89.1% | Not published | N/A | N/A |
| **REAL Bench** | — | 19% | N/A | N/A |
| **BU Bench V1** | SOTA (own benchmark, 100 tasks) | — | — | — |

Browser Use leads on autonomous benchmarks. Stagehand's value proposition is production reliability + cost convergence, not raw benchmark scores.

## Known Limitations

### Playwright MCP
- Tool proliferation: 70+ tools degrades LLM tool selection accuracy (43% → <14%)
- Shadow DOM invisible to accessibility tree snapshots
- Not a security boundary — must not be used against production systems without isolation
- Token-heavy: 4x more than CLI equivalent

### Browser Use
- No caching — re-reasons every step, cost doesn't decrease over time
- No published scores on standard academic benchmarks (WebArena, etc.)
- Heavyweight: requires full CDP client, event bus, DOM services

### Agent Browser
- Very new (January 2026) — community MCP wrapper, not official
- No built-in cloud option
- Node.js daemon dependency alongside Rust CLI

### Stagehand
- Low autonomous benchmark scores (19% REAL Bench)
- Cloud infrastructure costs (Browserbase) on top of LLM costs
- Tightly coupled to Browserbase cloud for production use

## Relevance to lx

Browser automation is a critical agent capability. The comparison reveals a spectrum:

1. **Tools** (Playwright MCP, Agent Browser) — the agent orchestrates, tools execute specific actions. Maps to lx's `tool` invocations.
2. **Autonomous agents** (Browser Use) — a sub-agent that owns the browser and pursues goals. Maps to lx's `agent` spawning with `task` assignments.
3. **Hybrid** (Stagehand) — code-level control with AI fallback. Maps to lx's flow control with conditional `refine` loops.

For lx's MCP tool integration, Playwright MCP and Browser Use (as MCP server) are directly usable. For lx programs that need browser capabilities, the right abstraction may be a `browser` builtin that delegates to whichever backend is configured — similar to how lx handles LLM providers.

The token economics findings reinforce lx's design decision to minimize token overhead: a `browser.snapshot()` returning a ref-based element tree is far more token-efficient than streaming full DOM state through the agent's context window.
