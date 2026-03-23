# GSD-2: System Architecture

## Source Tree

```
gsd-2/
├── src/
│   ├── loader.ts              Bootstrap: sets env vars, zero SDK imports
│   ├── cli.ts                 Main CLI router: print/interactive/headless dispatch
│   ├── onboarding.ts          First-run setup wizard (LLM provider + API keys)
│   ├── wizard.ts              Env hydration from stored auth.json
│   ├── app-paths.ts           Directory constants (~/.gsd/, sessions, auth)
│   ├── models-resolver.ts     LLM model configuration resolution
│   ├── resource-loader.ts     Bundled resource sync (extensions → ~/.gsd/agent/)
│   ├── extension-discovery.ts Dynamic extension entry point discovery
│   ├── headless.ts            Headless mode orchestrator
│   ├── headless-events.ts     Event classification (terminal, blocked, idle)
│   ├── headless-ui.ts         Auto-response & progress formatting
│   ├── headless-context.ts    Context loading & project bootstrap
│   ├── headless-answers.ts    Pre-supplied answer injection
│   ├── headless-query.ts      Read-only state snapshot (no LLM)
│   └── resources/
│       ├── extensions/
│       │   ├── gsd/           Core GSD extension (60+ modules)
│       │   ├── browser-tools/ Playwright automation
│       │   ├── search-web/    Brave/Tavily/Jina search
│       │   ├── google-search/ Gemini-powered search
│       │   ├── context7/      Library documentation
│       │   ├── bg-shell/      Background process management
│       │   ├── subagent/      Isolated context execution
│       │   ├── mac-tools/     macOS Accessibility APIs
│       │   ├── mcporter/      Lazy MCP server integration
│       │   ├── voice/         Speech-to-text
│       │   ├── slash-commands/ Custom command creation
│       │   ├── lsp/           Language Server Protocol
│       │   ├── ask-user-questions.ts
│       │   ├── get-secrets-from-user.ts
│       │   ├── async-jobs/    Background task execution
│       │   ├── remote-questions/ Discord/Slack/Telegram
│       │   ├── ttsr/          Tool-triggered system rules
│       │   └── universal-config/
│       ├── agents/
│       │   ├── scout.md       Fast codebase recon
│       │   ├── researcher.md  Web research
│       │   ├── worker.md      General execution
│       │   ├── typescript-pro.md
│       │   └── javascript-pro.md
│       ├── AGENTS.md
│       └── GSD-WORKFLOW.md
├── packages/
│   ├── native/                N-API TypeScript wrappers
│   ├── pi-agent-core/         Vendored agent session management
│   ├── pi-ai/                 Unified LLM provider API
│   ├── pi-tui/                Terminal UI library
│   └── pi-coding-agent/       Core CLI implementation
├── native/
│   ├── crates/engine/         N-API Rust cdylib
│   ├── crates/grep/           Ripgrep Rust library
│   ├── crates/ast/            AST search via ast-grep
│   └── npm/                   Platform-specific binaries
├── studio/                    Electron + React IDE (experimental)
├── vscode-extension/          VS Code integration (15 commands + chat)
├── scripts/                   Build, dev, CI orchestration
├── tests/                     Smoke, fixture, live tests
├── pkg/                       Pi SDK shim (PI_PACKAGE_DIR target)
└── docs/                      182 files of documentation
```

## Bootstrap Flow

### Two-File Loader Pattern

```
loader.ts  (synchronous, zero SDK imports)
  │
  ├── Fast-path --version / --help (avoids ~1s import time)
  ├── Resolve bundled vs distributed resources (dist/ > src/)
  ├── Set environment variables:
  │   ├── PI_PACKAGE_DIR → pkg/ (not project root, avoids theme collision)
  │   ├── PI_SKIP_VERSION_CHECK → true
  │   ├── GSD_CODING_AGENT_DIR → ~/.gsd/agent/
  │   ├── NODE_PATH → GSD's node_modules
  │   ├── GSD_VERSION → package version
  │   ├── GSD_BIN_PATH → absolute loader path
  │   ├── GSD_WORKFLOW_PATH → GSD-WORKFLOW.md path
  │   └── GSD_BUNDLED_EXTENSION_PATHS → discovered extension entries
  ├── Ensure workspace packages symlinked (@gsd/* scope)
  ├── Respect HTTP_PROXY / HTTPS_PROXY
  └── Dynamic import cli.ts
          │
          cli.ts  (imports Pi SDK, heavy dependencies)
            ├── parseCliArgs() → flags, model, mode, extensions
            ├── Subcommand routing (config, update, sessions, headless)
            ├── Print mode (text, json, rpc, mcp)
            └── Interactive mode (TUI session)
```

### Resource Synchronization

On every launch, bundled resources are synced from `src/resources/` → `~/.gsd/agent/`:
- `extensions/` → `~/.gsd/agent/extensions/`
- `agents/` → `~/.gsd/agent/agents/`
- `skills/` → `~/.gsd/agent/skills/`

Optimization: skips copy when `managed-resources.json` version matches current GSD version (avoids ~128ms sync overhead).

### Extension Discovery

`discoverExtensionEntryPaths(extensionsDir)` scans:
1. Top-level `.ts`/`.js` files → standalone extension entry points
2. Subdirectories → check `package.json` for `pi.extensions` array, fallback to `index.ts`/`index.js`

### Session Storage

Per-directory scoping in `~/.gsd/sessions/<escaped-cwd>/`. Automatic migration from flat layout to per-directory structure.

## State-on-Disk Model

`.gsd/` is the **sole source of truth**. Auto mode reads it, writes it, and advances based on what it finds. No in-memory state survives across sessions.

This enables:
- **Crash recovery** — lock file tracks current unit; next `/gsd auto` reads surviving session
- **Multi-terminal steering** — `/gsd discuss` in terminal 2 writes decisions picked up at next phase boundary
- **Session resumption** — `continue.md` captures exact resume point
- **Parallel workers** — each worker owns a milestone via `GSD_MILESTONE_LOCK`

### Directory Structure

```
.gsd/
├── milestones/              (tracked in git)
│   └── M001/
│       ├── M001-CONTEXT.md     User decisions from discuss phase
│       ├── M001-ROADMAP.md     Milestone plan with slice checkboxes
│       ├── M001-RESEARCH.md    Codebase/tech research
│       ├── M001-SUMMARY.md     Rollup on completion
│       ├── M001-PARKED         Marker: parked milestone
│       └── slices/
│           └── S01/
│               ├── S01-PLAN.md       Task decomposition
│               ├── S01-CONTEXT.md    Slice decisions
│               ├── S01-RESEARCH.md   Slice research
│               ├── S01-SUMMARY.md    Completion summary
│               ├── S01-UAT.md        User acceptance tests
│               ├── continue.md       Ephemeral resume point
│               └── tasks/
│                   ├── T01-PLAN.md
│                   └── T01-SUMMARY.md
├── PROJECT.md              (tracked) Living doc of what project is
├── DECISIONS.md            (tracked) Append-only decision register
├── REQUIREMENTS.md         (tracked) Project requirements
├── QUEUE.md                (tracked) Future milestone queue
├── KNOWLEDGE.md            (tracked) Cross-session memory/rules
├── CAPTURES.md             Pending thought captures
├── preferences.md          Project-level preferences
├── STATE.md                (gitignored) Derived cache
├── auto.lock               (gitignored) Crash sentinel
├── completed-units.json    (gitignored) Dispatch idempotency
├── metrics.json            (gitignored) Token/cost ledger
├── routing-history.json    (gitignored) Adaptive model learning
├── gsd.db                  (gitignored) SQLite cache
├── worktrees/              (gitignored) Separate checkouts
│   └── M001/               Full git worktree
├── activity/               (gitignored) JSONL session dumps
├── runtime/                (gitignored) Dispatch/timeout records
├── parallel/               (gitignored) Coordinator IPC
│   ├── M001.status.json
│   └── M001.signal.json
└── reports/                HTML exports
```

## Dispatch Pipeline

The core loop that drives auto-mode:

```
1. Read disk state (.gsd/ files via deriveState())
2. Determine next unit type and ID (dispatch rules)
3. Classify complexity → select model tier
4. Apply budget pressure adjustments
5. Check routing history for adaptive adjustments
6. Dynamic model routing (if enabled) → select cheapest model for tier
7. Resolve effective model (with fallbacks)
8. Check pending captures → triage if needed
9. Build dispatch prompt (applying inline level compression)
10. Create fresh agent session
11. Inject prompt and let LLM execute
12. On completion: snapshot metrics, verify artifacts, persist state
13. Loop to step 1
```

### State Derivation

`deriveState(basePath)` reconstructs `GSDState` from `.gsd/` files:
- Uses native batch parsing when available (Rust module reads all `.md` files in one call)
- Fallback to sequential JS file reads
- 100ms TTL cache (avoids re-reading within a dispatch cycle)

### Auto-Mode Bootstrap Sequence

When auto-mode starts, 22 steps run before first dispatch:

1. Git init if not repo
2. `.gitignore` baseline patterns
3. `.gsd/` directory creation
4. Crash lock detection + crash recovery
5. Debug mode init
6. State derivation + stale worktree recovery
7. Milestone branch recovery
8. Guided flow (if no active milestone or needs discussion)
9. Session state initialization
10. SIGTERM handler registration
11. Integration branch capture
12. Auto-worktree setup (if configured)
13. DB lifecycle (migration from markdown if needed)
14. Metrics initialization
15. Routing history initialization
16. Model snapshot
17. Skill snapshot (if discovery enabled)
18. Status notification
19. Lock file write
20. Secrets collection from manifest
21. Self-heal (clear stale records + .git/index.lock)
22. Pre-flight validation (milestone queue check)

## Packages

### @gsd/native (v0.1.0)

High-performance Rust N-API bindings. Platform binaries distributed as `@gsd-build/engine-{platform}`.

| Module | Backing | Purpose |
|--------|---------|---------|
| grep | ripgrep | Content search with glob filtering, .gitignore support |
| glob | — | Gitignore-aware file discovery |
| ps | — | Cross-platform process tree management |
| highlight | syntect | Syntax highlighting |
| ast | ast-grep | Structural code search |
| diff | — | Fuzzy text matching + unified diff |
| text | — | ANSI-aware text measurement/wrapping |
| html | — | HTML-to-Markdown conversion |
| image | — | Decode, encode, resize images |
| fd | — | Fuzzy file path discovery |
| clipboard | — | Native clipboard access |
| xxhash | — | Fast hashing |
| git | libgit2 | Read operations (dispatch hot path) |
| gsd-parser | — | GSD file parsing + frontmatter extraction |

### @gsd/pi-agent-core (v0.57.1)

Vendored general-purpose agent core from pi-mono. Pure TypeScript, zero deps. Foundational agent session management primitives.

### @gsd/pi-ai (v0.57.1)

Unified LLM provider API: Anthropic, OpenAI, Google, Mistral, AWS Bedrock. OAuth support, validation (ajv + TypeBox schemas), proxy support.

### @gsd/pi-tui (v0.57.1)

Terminal UI library: chalk styling, marked markdown, mime-types, terminal control. Optional koffi for native bindings.

### @gsd/pi-coding-agent (v0.57.1)

Core coding agent CLI: command routing, core agent logic, execution modes (interactive/RPC/print), utilities. Depends on sql.js, yaml, glob, diff, proper-lockfile.

## Studio (Experimental)

Electron + Vite + React 19 + Tailwind CSS desktop IDE:
- Workflow visualization
- Milestone/slice/task management
- Real-time progress monitoring
- Session control
- State: zustand
- UI: react-resizable-panels, Phosphor icons

## VS Code Extension

Full-featured integration with 15 commands + `@gsd` chat participant:
- Sidebar dashboard (connection status, model, tokens, cost, actions)
- Start/stop/new session/switch model/cycle thinking/compact context/abort/export
- Keyboard shortcuts: `Ctrl+Shift+G` chords
- Protocol: JSON-RPC over stdin/stdout
- Publisher: FluxLabs
