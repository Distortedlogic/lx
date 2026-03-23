# GSD-2: Extensions and Agents

## Extension Architecture

Extensions are TypeScript modules that hook into the Pi runtime. They can register tools, intercept events, register slash commands, render custom UI, persist state, control rendering, modify system prompts, and manage models.

### Extension Lifecycle

```
activate → context available → events fired → deactivate
```

### Event System

```
session_start → input → before_agent_start → agent_start → turn_start →
  context → tool_call → tool_result → turn_end →
agent_end → session_shutdown
```

### Extension Capabilities

- Register custom tools (give the LLM new abilities)
- Register slash commands (user-facing actions)
- Register custom UI (dialogs, persistent elements, overlays)
- Persist state (localStorage across restarts)
- Modify system prompt (per-turn dynamic changes)
- Control compaction (message summarization strategies)
- Manage models/providers (register custom providers, switch models)
- Override built-in tools (remote execution)

## Bundled Extensions (17+)

### GSD Core (`gsd/`)

The core workflow engine with 60+ modules:

| Category | Modules |
|----------|---------|
| State Machine | `auto.ts`, `auto-dispatch.ts`, `auto-prompts.ts`, `auto-start.ts` |
| State & Files | `state.ts`, `files.ts`, `paths.ts`, `types.ts` |
| Commands | `commands.ts`, `commands-prefs-wizard.ts`, `commands-config.ts`, `commands-inspect.ts`, `commands-maintenance.ts`, `commands-handlers.ts` |
| Verification | `verification-gate.ts`, `verification-evidence.ts` |
| Git | `worktree.ts`, `auto-worktree.ts`, `git-service.ts`, `git-self-heal.ts` |
| Recovery | `auto-recovery.ts`, `crash-recovery.ts`, `session-forensics.ts` |
| Planning | `post-unit-hooks.ts`, `pre-dispatch-hooks.ts`, `auto-idempotency.ts` |
| Budget | `auto-budget.ts`, `metrics.ts`, `context-budget.ts` |
| Tool Tracking | `auto-tool-tracking.ts`, `auto-timeout-recovery.ts`, `auto-stuck-detection.ts` |
| Parallel | `parallel-orchestrator.ts`, `parallel-eligibility.ts`, `parallel-merge.ts` |
| Skills | `skill-discovery.ts`, `skill-telemetry.ts` |
| UI | `dashboard-overlay.ts`, `visualizer-overlay.ts`, `guided-flow.ts` |
| Config | `preferences.ts`, `auto-model-selection.ts` |
| Health | `doctor.ts`, `doctor-proactive.ts`, `auto-observability.ts` |
| Persistence | `activity-log.ts`, `session-status-io.ts` |

### Browser Tools (`browser-tools/`)

Playwright-based browser automation with 20+ tool categories:

| Tool Category | Capabilities |
|---------------|-------------|
| Navigation | Go to URL, back, forward, reload |
| Screenshot | Capture page with positioning/constraints |
| Interaction | Click, type, scroll, hover, drag |
| Inspection | Query elements, extract data, accessibility markup |
| Session | Launch, close, switch pages, manage tabs |
| Assertions | Verify page state, compare values |
| Refs | Reference elements across actions, stale detection |
| Wait | For elements, conditions, network quiet |
| Forms | Fill forms, extract values |
| Intent | High-level action planning |
| PDF | Interact with PDFs |
| State Persistence | Save/restore page state |
| Network Mock | Intercept/mock API responses |
| Device | Emulate devices, orientation, viewport |
| Extract | Structured data extraction from page |
| Visual Diff | Compare page screenshots |
| Codegen | Generate action code |
| Action Cache | Save/retrieve action sequences |
| Injection Detection | Detect injected scripts |

Key internals:
- `ActionTimeline` — timeline of tool calls with status tracking
- `DiffResult` — before/after change tracking
- `FailureHypothesis` — categorized failure signals
- Adaptive settling (wait for DOM mutations)
- Element reference tracking across actions
- Compact page state capture

### Background Shell (`bg-shell/`)

Long-running process management with intelligent lifecycle:

| Action | Purpose |
|--------|---------|
| `start` | Launch with auto-classification & readiness detection |
| `digest` | Structured summary (~30 tokens vs ~2000 raw) |
| `output` | Raw output lines, incremental delivery |
| `highlights` | Significant lines only (errors, URLs, results) |
| `wait_for_ready` | Block until process signals readiness |
| `send` | Write stdin |
| `send_and_wait` | Expect-style: send + wait for output pattern |
| `run` | Execute on persistent shell session, block until done |
| `env` | Query shell cwd and environment variables |
| `signal` | Send OS signal (SIGINT, SIGTERM, SIGHUP) |
| `list` | All processes with status |
| `kill` | Terminate |
| `restart` | Kill + relaunch |
| `group_status` | Health of process group |

Features:
- Process type classification: server, build, test, watcher, generic, shell
- Readiness detection: port probing, pattern matching, auto-classification
- Output diffing & dedup: detect novel errors vs repeated noise
- Process groups: manage related processes as unit
- Cross-session persistence: survive context resets
- Context injection: proactive alerts for crashes and state changes

### Subagent (`subagent/`)

Spawns separate `pi` process for each invocation with isolated context window.

Three modes:
```typescript
// Single
{ agent: "name", task: "..." }

// Parallel (concurrent with limit)
{ tasks: [{ agent: "name", task: "..." }, ...] }

// Chain (sequential with output forwarding)
{ chain: [{ agent: "name", task: "... {previous} ..." }, ...] }
```

Agent discovery searches:
- User agents: `~/.pi/agents/`
- Project agents: `.pi/agents/`

Usage stats tracked: turns, input/output tokens, cache read/write, cost, context %.

### Voice (`voice/`)

Real-time speech-to-text:
- **macOS** — Native Speech framework (Swift binary compiled on first use)
- **Linux** — Python + GROQ API (sounddevice + whisper-cpp)
- Voice footer indicator (flashing dot)
- Toggle via `Ctrl+Alt+V`

### LSP (`lsp/`)

Language Server Protocol integration: diagnostics, definitions, references, hover, rename, code actions.

### Other Extensions

| Extension | Purpose |
|-----------|---------|
| Search the Web | Brave Search, Tavily, Jina page extraction |
| Google Search | Gemini-powered web search with AI-synthesized answers |
| Context7 | Up-to-date library/framework documentation |
| Mac Tools | macOS native app automation via Accessibility APIs |
| MCPorter | Lazy on-demand MCP server integration |
| Slash Commands | Custom command creation |
| Ask User Questions | Structured user input with single/multi-select, validation |
| Secure Env Collect | Masked secret collection + environment variable injection |
| Async Jobs | Background job management (bash, await, cancel) |
| Remote Questions | Discord/Slack/Telegram integration for headless mode |
| TTSR | Tool-Triggered System Rules |
| Universal Config | Discover existing AI tool configs (.env, config.json, etc.) |

## Bundled Agents

### Scout

Fast codebase reconnaissance returning compressed context for handoff.
- Tools: read, grep, find, ls, bash
- Outputs: file lists with line ranges, critical code sections, architecture summary, start points
- Thoroughness levels: Quick (targeted), Medium (follow imports), Thorough (all dependencies)
- Used as first-pass explorer before other agents dive deep

### Researcher

Web researcher using web_search and bash tools.
- Synthesizes current information from multiple queries
- Output format: Summary, Key Findings (with source URLs), numbered Sources
- Factual, cites sources, notes conflicting results

### Worker

General-purpose isolated context agent with full capabilities.
- Key constraint: does NOT spawn subagents or orchestrate
- If work is GSD orchestration/planning, reports caller should use specialist instead
- Outputs: Completed section, Files Changed, Notes (optional handoff info)

### TypeScript Pro

Senior TypeScript developer for advanced type system patterns.
- Specializes in: type-first development, strict mode, full-stack type safety, build tooling
- Initialization: reads tsconfig, assesses patterns, identifies framework, checks lint
- Core patterns: conditional types, mapped types, template literal types, branded types, result types

### JavaScript Pro

Available for TypeScript/JavaScript work (complementary to TypeScript Pro).

## Skills (17 Bundled)

| Skill | Domain |
|-------|--------|
| frontend-design | Frontend UI/UX |
| swiftui | SwiftUI development |
| debug-like-expert | Structured debugging |
| rust-core | Rust development |
| axum-web-framework | Axum web framework |
| axum-tests | Axum testing |
| tauri | Tauri desktop apps |
| tauri-ipc-developer | Tauri IPC patterns |
| tauri-devtools | Tauri dev tools |
| github-workflows | GitHub Actions CI/CD |
| security-audit | Security auditing |
| security-review | Security review |
| security-docker | Docker security |
| review | Code review |
| test | Testing |
| lint | Linting |

### Skill Discovery Modes

- `auto` — automatically load matching skills
- `suggest` (default) — suggest relevant skills for approval
- `off` — manual only

### Skill Lifecycle

- `/gsd skill-health` — dashboard with usage stats, success rates, token trends
- Staleness detection (60 days default) auto-deprioritizes unused skills
- Post-unit analysis for drift detection
- Custom skills in `~/.gsd/agent/skills/<name>/SKILL.md`
- Project-local skills in `.pi/agent/skills/`
