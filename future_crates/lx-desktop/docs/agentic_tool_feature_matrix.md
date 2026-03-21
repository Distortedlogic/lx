# Agentic Tool Feature Matrix

Consolidated, deduplicated feature list extracted from analysis of 20+ agentic coding tools (March 2026). Generic capabilities only — no tool attributions, no issues, no benchmarks.

---

## 1. Agent Loop & Execution

- ReAct / TAOR loop (Think-Act-Observe-Repeat)
- State machine agent loop with typed event stream (content, tool call, tool result, reasoning, error, finished, loop detected, context overflow)
- Spec-driven execution (requirements → design → implementation tasks before any code is written)
- EARS notation for requirements (`WHEN [condition] THE SYSTEM SHALL [expected behavior]`)
- Requirements-first vs design-first workflow variants
- Four capability primitives (Read, Write/Edit, Execute, Connect)
- Plan mode (read-only analysis before acting)
- Architect mode (two-model approach: planner reasons about the problem, editor translates to file edits)
- Switchable agent modes with different tool access, system prompts, and file restrictions per mode
- Development autonomy modes (fully autonomous / agent-assisted with checkpoints / human-approval-per-step / custom mixed)
- Iterative refinement (generate → execute → observe failures → fix loop)
- Loop detection (detect agent stuck in cycles, break out)
- Automatic context compaction (summarize conversation history when approaching context window limit)
- Model-tool co-training (model specifically trained to work with the agent's tool primitives)

- AI-assisted debugging as dedicated capability (analyze runtime errors, suggest root causes, trace execution — distinct from general code editing)

## 2. Tool Use

- File read (single file, multiple files, images, PDFs, notebooks)
- File write / create
- File edit (surgical string replacement with uniqueness constraint)
- Patch-based file edit (structured diff with operation declarations, context markers, progressive fuzzy matching)
- Search/replace block editing with layered matching fallbacks (exact → whitespace-insensitive → indentation-preserving → Levenshtein fuzzy)
- Two-stage apply model (primary LLM generates change sketch focused on logic, specialized apply model integrates into existing file)
- Shell / terminal command execution
- Web search with grounding
- Web fetch (retrieve and process URL content)
- Browser automation (headless browser interaction, screenshots, click elements, form filling)
- Ask user (interactive clarification dialog)
- File glob / pattern search
- Content grep / regex search (ripgrep-backed)
- Directory listing
- LSP integration (type info, symbol definitions, real-time diagnostics from language servers)
- Code graph / cross-repo symbol search (symbols, references, call hierarchies, dependency trees)
- Image / screenshot analysis (multimodal vision input)
- Tool search / deferred tool loading (load tool schemas on demand to save context)
- Omission placeholder detection (detect and reject "// rest of code here" laziness, force complete output)
- `@` shorthand for including file content in prompts
- `!` shorthand for executing shell commands inline
- Multi-directory context inclusion (add context from multiple paths beyond working directory)
- Codebase search agent (AI-assisted semantic search as a dedicated sub-agent tool)
- Content safety module (filter unsafe/harmful generated content)
- Annotated code diffs as structured deliverables

## 3. Code Editing Formats

- Search/replace blocks (delimited original/replacement sections)
- Structured patch format (begin/end envelope, add/update/delete operations, scope headers instead of line numbers)
- Unified diff format
- Whole file replacement
- Two-stage apply (change sketch + integration model)
- Editor-diff / editor-whole (formats optimized for architect mode's editor model)
- Direct string replacement (old_string → new_string with uniqueness constraint)
- Diff-fenced (filename inside code fence, for specific model families)

## 4. Sub-Agents & Parallelism

- Sub-agent spawning (isolated context window, one-way result return to parent)
- Typed sub-agents with specialized roles (explore/search, plan/architect, general-purpose, codebase-investigator, memory-manager, custom-defined)
- Background agents (async, cloud-based, clone repo into isolated VM, open PR when done)
- Agent teams (multi-agent coordination with shared task lists, bidirectional communication between teammates, team lead synthesizes results)
- Multi-repository autonomous write operations (agent makes changes across multiple repos as part of a single task, not just cross-repo search)
- Worktree isolation (agent works in a git worktree copy, main branch untouched)
- Parallel tool execution (read-only tools run concurrently)
- Scheduled / recurring agent sessions (cron-like, configurable frequency, prompt, playbook)
- Event-triggered agents (fire on Slack message, GitHub event, PagerDuty alert, webhook, Linear update, schedule)
- Configurable agent selection and notification preferences per scheduled session

## 5. Context Engineering

### 5.1 Project Configuration Files

- Hierarchical instruction files (global → project → directory, closer overrides farther)
- Override files (take priority over regular instruction files at same level)
- Fallback filename support (check multiple filenames in priority order)
- Maximum combined instruction file size (configurable, e.g. 32 KiB)
- YAML front matter inclusion modes: `always` (every interaction), `auto` (AI decides relevance from description), `manual` (explicit user reference)
- Modular imports (`@file.md` syntax to compose large instruction files from smaller components)
- Per-mode instruction files (different rules for code mode vs architect mode vs debug mode)
- Skills / skill packages (directory-based with YAML frontmatter definition file, optional scripts/references/assets)
- Progressive disclosure for skills (only metadata indexed until semantically triggered, full content loaded on demand, released after task)
- Skill integration patterns: instruction-only, asset references, few-shot examples, deterministic script execution, complex orchestration
- Powers (bundled packages of MCP servers + instruction files + hooks, loaded on demand)
- Cross-tool skill format conversion (single skill definition convertible to multiple tool-specific formats)
- Community skill libraries (installable collections of hundreds/thousands of skills)

### 5.2 Context Retrieval & Understanding

- Semantic code search engine (embedding-based, understands meaning not just text, scales to 500K+ files)
- Quantized vector search (shrink search space by orders of magnitude while maintaining >99.9% fidelity)
- Repository map (tree-sitter AST parsing + PageRank dependency ranking, compressed to token budget)
- RAG-based codebase indexing (index on project open, retrieve relevant snippets per query)
- Cross-repository search (keyword and semantic/natural language across all repos)
- Commit history as context (full git history available to agent)
- JIT context discovery (auto-load instruction files when tools access new directories)
- Context engine as standalone MCP server (plug semantic understanding into any agent)
- Smart model routing (classify query complexity, route to powerful vs fast model)
- Local model for routing decisions (run small local model to classify without API call)
- Action tracking / flow awareness (monitor file edits, terminal commands, clipboard, navigation to infer intent)
- Proactive suggestions (adapt in real-time based on observed developer behavior)
- Codemaps (AI-annotated visual maps of code structure for codebase onboarding)
- Live context refresh (as code evolves, agent refreshes its understanding automatically)
- Precise reference finding (identify all usages of a symbol across codebase)
- Call hierarchy and dependency tree navigation
- Open files as immediate context
- Embeddings-based codebase semantic search (separate from full semantic engine)
- Multi-file comprehension from single prompt (read existing patterns, check schemas, create handlers matching conventions)

### 5.3 Memory & Persistence

- Persistent full working context across sessions (maintain agent's complete situational awareness across sessions — not just discrete saved facts/memories, but the full contextual state)
- File-based persistent memory with typed categories (user preferences, feedback/corrections, project state, external references)
- Memory index file (always loaded, concise pointers to individual memory files)
- Agent-written auto-memory (agent saves corrections and preferences automatically without user action)
- Knowledge base learning (persistent pattern capture — code patterns, naming conventions, team preferences — anticipate style over time)
- Session management (save, resume, share conversations, SQLite-backed)
- Session sharing via links
- Editable messages in conversation threads (edit a previously submitted message rather than only appending)
- Auto-generated conversation/thread titles (based on content, without user input)
- Cross-session memory (answer questions about work from previous sessions)
- Workspace-scoped memories (persistent facts scoped to workspace)
- Playbooks (reusable system prompts for repeated tasks)
- Memory user controls (view, edit, delete individual memories)

### 5.4 Context Window Management

- 1M-2M token context windows
- Auto-compaction with compression prompt (summarize rather than truncate)
- Configurable compaction trigger threshold (e.g. 90-95% of window)
- Progressive disclosure (load full skill/instruction content only when relevant)
- Token caching / prompt caching
- Thinking budget cap (prevent runaway reasoning, configurable max tokens)

## 6. Security & Sandboxing

### 6.1 Sandbox Methods

- macOS Seatbelt (`sandbox-exec` with configurable profiles)
- Linux Landlock + seccomp (kernel-level filesystem + syscall filtering)
- Linux bubblewrap + seccomp (container-like isolation)
- Docker / Podman containers
- gVisor / runsc (user-space kernel intercepts all syscalls — strongest isolation)
- LXC / LXD (full-system container sandboxing)
- Windows Native (Restricted Token API or icacls integrity levels)
- Full VM sandbox (cloud VM with terminal + editor + browser — complete environment isolation)
- Cloud Ubuntu VMs (cloned repo in isolated VM per agent)
- Application-level hooks (PreToolUse can deny/allow/ask per tool call)
- Two-phase cloud runtime (setup phase has network access for dependencies, agent phase runs offline)

### 6.2 Approval Policies

- Human-in-the-loop (approve every file edit, command, browser action)
- Tiered approval (auto for reads, ask for writes/commands/network)
- Per-mode tool restrictions (each mode has allowed/disallowed tools and file patterns)
- Protected paths (always read-only regardless of mode: `.git`, config dirs)
- Sandbox escalation (per-command approval to bypass sandbox, with audit flag)
- Granular auto-approval (approve reads but not writes to specific paths, approve specific commands)
- Policy lockdown (admin-controlled TOML-based policy engine with integrity checking)
- Content safety filtering
- Trusted folders configuration

## 7. Hooks & Automation

### 7.1 Hook Events

- PreToolUse (block, modify input, or approve before tool execution)
- PostToolUse (react after execution, inject additional context, format output)
- UserPromptSubmit (intercept user prompt before processing)
- Stop (agent execution ends)
- SubagentStop (sub-agent completes)
- PreCompact (before context compaction, inject custom compaction instructions)
- File Created (glob pattern match on new files)
- File Saved (glob pattern match on saved files)
- File Deleted (glob pattern match on deleted files)
- Pre Task Execution (before spec task begins — setup scripts, prerequisite validation)
- Post Task Execution (after spec task completes — tests, linting, notifications)
- Agent Lifecycle (agent start/stop)
- Manual / user-triggered hooks

### 7.2 Hook Handler Types

- Command hooks (shell commands receiving JSON via stdin)
- Prompt hooks (natural language instructions)
- Agent hooks (full agent invocation)
- Synchronous (blocking) and asynchronous (fire-and-forget with timeout) execution
- Tool-name pattern matching (hook fires only for specific tools)

### 7.3 Automation Systems

- Event-driven automations (Slack, GitHub, PagerDuty, webhooks, schedules trigger always-on agents)
- GitHub Actions integration (official actions for PR review, issue triage, `@mention` triggers)
- Issue-to-PR automation (assign issue → agent plans, implements, opens PR, responds to review)
- Scheduled / recurring sessions (cron-like)
- Code review automation (scan PRs for bugs, security, quality; suggest fixes; auto-fix)
- Watch mode (monitor repo files for AI instruction comments, process as coding requests)
- Automatic lint and test after every AI change (auto-fix lint errors)
- PR comment response loop (agent revises based on code review feedback, iterates until approved)

## 8. Model & Provider Support

- Multi-provider support (connect to multiple LLM providers)
- Any provider via API key (BYOK, OpenAI-compatible endpoints)
- Local model support (Ollama, LM Studio, self-hosted)
- Purpose-built speed-optimized coding model (custom-trained model optimized for raw throughput over reasoning depth as default fast-path)
- Smart model routing (auto-select powerful vs fast model per query complexity)
- Local model for routing classification (run small local model to classify without API call)
- Per-task model switching (different models for different agent modes)
- Oracle model (dedicated powerful model for hardest queries)

## 9. Extended Thinking & Reasoning

- Extended thinking (internal reasoning before response)
- Adaptive thinking (model dynamically decides how much reasoning to apply)
- Think tool (explicit pause-and-reason tool invoked mid-execution to evaluate state)
- Deep think mode (extended reasoning with thinking tokens)
- Thinking budget cap (configurable maximum thinking tokens)
- Interleaved thinking (reasoning between tool calls, not just before first response)
- Thinking token usage transparency (surface reasoning/thinking token consumption in usage tracking)

## 10. Extensibility & Ecosystem

### 10.1 MCP (Model Context Protocol)

- MCP client (connect to external tool servers)
- MCP server (expose tools to other agents)
- Pre-bundled MCP servers (ship useful servers out of the box)
- Skills bundle MCP servers (mcp.json in skill directory, servers start on launch, tools hidden until skill loaded)
- MCP transport methods (stdio, SSE, HTTP, WebSocket)
- MCP tool namespacing (`mcp__<server>__<tool>`)

### 10.2 Skills / Plugins / Extensions

- Skills (SKILL.md with progressive disclosure)
- Plugins (distributable packages bundling slash commands + sub-agents + hooks + MCP configs + skills, with manifest)
- Plugin namespacing (`plugin-name:skill-name`)
- Extensions (installable from GitHub / extension gallery)
- Toolboxes (simple executable scripts, auto-discovered on startup via action descriptor)
- Powers (bundled MCP + instructions + hooks for specific technologies)
- Custom slash commands (user-defined reusable commands invocable with `/`)
- Cross-tool skill format standard
- Plugin / extension marketplace
- MCP Apps with interactive UIs (charts, diagrams, whiteboards rendered inside the tool)
- Remote plugins (plugins executing as remote services rather than local code, distinct from MCP tool servers)

### 10.3 SDK / Programmatic Access

- Agent SDK (build custom agents with same tools, loop, and context management — Python, TypeScript, Rust)
- Three API layers: one-shot query, streaming query, bidirectional multi-turn client
- Custom tools via in-process MCP servers (macro-based tool definition)
- Session management in SDK (separate contexts, fork sessions, quick switch)
- Dynamic control (interrupt mid-execution)
- Budget control in SDK (max token/cost budget, fallback model)
- Structured output (JSON schema for constrained outputs)
- File checkpointing in SDK (track changes, rewind to previous state)
- Plugin system in SDK (load plugins from filesystem)
- Beta feature flags (e.g. 1M context window)
- Headless / non-interactive mode (CLI flag for scripting, JSON and stream-JSON output)
- Agent-to-Agent protocol (A2A — inter-agent communication standard)
- Agent Client Protocol (ACP — run as agent inside other IDEs)
- Wire protocol (newline-delimited JSON for cross-language extensibility)

## 11. Git & Version Control

- Automatic commits per AI change with Conventional Commit messages
- Commit attribution metadata (mark commits as AI-generated)
- Automatic branch creation
- Automatic PR creation with detailed descriptions
- PR review / feedback loop (agent responds to PR comments, revises, iterates)
- Git worktree isolation (agent works in worktree copy, main branch untouched)
- Undo AI changes (single command revert)
- Shadow git snapshotting (capture working tree state before tool execution, rollback on failure)
- File checkpointing (track file changes, rewind to any previous state)
- Dirty file handling (auto-commit uncommitted changes before AI edits, configurable)

## 12. UI & Interaction Surfaces

### 12.1 Interface Types

- Terminal TUI (GPU-accelerated, Bubble Tea / Ratatui / Ink)
- VS Code fork (full IDE with AI deeply integrated)
- VS Code extension (runs in standard VS Code)
- JetBrains plugin / integration (via ACP or native plugin)
- Desktop app (native macOS/Linux/Windows, Tauri or Electron)
- Web interface (browser-based agent interaction)
- Headless CLI (non-interactive, scriptable)
- Neovim plugin
- Editor-agnostic (not tied to any single IDE, runs in multiple)
- External CLI agents embedded as first-class panels in a host editor (host editor runs agents from different vendors as integrated panels)

### 12.2 IDE Features

- GPU-accelerated editor/IDE rendering framework (custom GPU-accelerated UI rendering for the code editor itself, enabling native-speed performance)
- Dual interface (editor view for synchronous coding + manager view for orchestrating async agents)
- Three-surface model (editor + terminal + browser as integrated agent workspace)
- Composer mode (multi-file editing from single prompt, shown in diff view, applied atomically)
- Tab completion (AI-generated, sub-10ms latency, custom sparse model)
- Next edit prediction (predict what developer will edit next, not just current line)
- Multi-cursor AI-aware editing
- Agent following (editor jumps to files agent reads/edits in real-time)
- Change review (accept/reject individual hunks or entire change set in multi-buffer view)
- One-click settings/themes/keybinds migration from VS Code
- Vim-like keybindings throughout

### 12.3 Terminal Features (Agent-Era)

- GPU-accelerated rendering (handle agent output floods without lag)
- Native desktop notifications (zero-config, OSC 9/99/777 escape sequences)
- Embeddable terminal library (reusable core for building purpose-built agent terminals)
- Standalone VT parsing library (zero-dependency, SIMD-optimized, embeddable)
- Multi-pane / split for parallel agents
- Shell integration (OSC 133 command boundaries for "command finished" detection)
- Kitty Graphics Protocol (images in terminal)
- Tmux Control Mode support
- Ligature and variable font support
- AppleScript / scriptable terminal automation
- Platform-native rendering (Metal on macOS, OpenGL on Linux — not Electron)
- AI-native terminal emulator (AI capabilities like command suggestions and natural language shell built directly into the terminal application, vs running a separate agent inside a conventional terminal)
- **[NOVEL] Heterogeneous panes** (tabs/splits can be terminal OR browser OR UI canvas — konsole-like paning where each pane is a different surface type)

### 12.4 Purpose-Built Agent Terminal Features

- Vertical tabs showing git branch / PR status / working directory per pane
- Notification rings when agents need attention
- Centralized notification panel across all agent sessions
- Built-in scriptable browser alongside terminal panes
- Socket API for external automation of terminal sessions

### 12.5 Collaboration

- Session sharing via links
- Artifact commenting (feedback on agent output without interrupting execution)
- Chat / messaging integration (assign tasks, receive notifications via external messaging)
- Issue tracker integration (link tasks, track progress)

## 13. Observability & Debugging

- Real-time token usage tracking (token consumption per message in UI)
- OpenTelemetry support (audit trails)
- DevTools inspector (built-in debugging for agent internals)
- Artifact audit trails (implementation plans, screenshots, browser recordings)
- Playbook analytics (session count, merged PRs per playbook, activity charts)
- Agent following mode (editor tracks agent activity in real-time)
- Efficiency metrics (edits per file, build attempts, with automatic feedback)
- Confirmation bus (dedicated system for managing approval request flow)

## 14. Voice & Multimodal

- Voice-to-code (in-chat voice command, seamless switching between voice and text)
- Image / screenshot input (paste images, Figma designs, screenshots for vision-model analysis)
- Screenshot → UI code generation
- Browser session recording output (video artifacts of agent testing)
- Multimodal understanding (code + screenshots + API responses + natural language simultaneously)

## 15. Code Quality & Testing

- Automatic lint after every AI change (run linter on modified files, auto-fix errors)
- Automatic test execution after every AI change
- Configurable lint and test commands
- AI code review (scan PRs for bugs, security flaws, code quality)
- AI code review auto-fix (fix identified issues automatically)
- AI test generation (automated test creation from code)
- Quality gates (AI-powered quality enforcement in CI/CD pipelines)
- Source-controlled AI checks enforceable in CI (markdown-defined checks that run as agents on every PR)
- `/context` command (automatically identify which files need editing for a given request)

## 16. Project Scaffolding & Generation

- Builder mode (describe project in natural language, generate entire codebase including structure, dependencies, implementation)
- "Think-before-doing" scaffolding (plan architecture before generating code)
- Code generation from natural language comments
- Spec-to-implementation pipeline (requirements → design → sequenced tasks → code)

## 17. Emerging Patterns

- Hybrid workflow (one tool generates features, another reviews code before merge)
- Agentmaxxing (running multiple AI agents in parallel on separate tasks/worktrees)
- The agent-era terminal stack (GPU-accelerated terminal + tmux + git worktrees + agent + notification hooks)
- Context engine as infrastructure (semantic code understanding as standalone service usable by any agent)
- Cross-tool instruction file compatibility (same instruction files work across multiple competing agents)
- Agent-native terminal (purpose-built terminal for agent workflows rather than repurposing general-purpose terminals)
