# Pi: The Minimal Coding Agent Harness

## Identity

Pi is an open-source, minimal terminal coding agent created by Mario Zechner (GitHub: `badlogic`), the creator of the libGDX game development framework. The project lives at `github.com/badlogic/pi-mono`. The official website is `shittycodingagent.ai` (redirects to `pi.dev`). The name was intentionally chosen to be "entirely un-Google-able, so there will never be any users."

27,600 GitHub stars, 2,900 forks, 156 contributors, 179 releases (latest v0.62.0, March 2026). Written in TypeScript (95.8%). MIT license.

Development started August 2025. The foundational blog post, "What I learned building an opinionated and minimal coding agent," hit #1 on Hacker News in November 2025.

## Core Philosophy

Zechner's explicit stance: "if I don't need it, it won't be built." Four principles define Pi's design:

**Minimalism.** The system prompt is under 1,000 tokens (vs. ~13,000 for Claude Code, mid-range for OpenCode). Only four tools ship by default: `read`, `write`, `edit`, `bash`. Zechner argues frontier models are "RL-trained up the wazoo" and understand coding agent contexts without extensive prompting. Users report "5-10x longer effective token windows versus Claude Code due to minimal system prompts and lack of MCP overhead."

**Context engineering.** "Exactly controlling what goes into the model's context yields better outputs." Zechner criticizes existing harnesses for "injecting stuff behind your back that isn't even surfaced in the UI." Pi surfaces everything.

**Observability.** "I want to inspect every aspect of my interactions with the model." Sessions are stored as JSONL files with tree structure. Zechner contrasts this with Claude Code's sub-agents being "a black box within a black box."

**YOLO mode by default.** No built-in permission system. Zechner argues that once a tool can write and execute code, "it's pretty much game over" for security anyway. Security is the user's responsibility, buildable via extensions.

## Architecture: The Monorepo

Pi is organized as 7 layered packages, each solving a distinct concern:

| Package | Purpose |
|---------|---------|
| **pi-ai** | Unified multi-provider LLM API |
| **pi-agent-core** | Agent loop: tool execution, validation, event streaming, state management |
| **pi-coding-agent** | CLI: session management, built-in tools, extensions, themes, project context |
| **pi-tui** | Custom terminal UI framework with differential rendering |
| **pi-web-ui** | Lit web components for browser-based chat with artifact rendering |
| **pi-mom** | Slack bot with per-channel agent isolation and Docker sandboxing |
| **pi-pods** | CLI for managing vLLM deployments on GPU pods (DataCrunch, RunPod, Vast.ai) |

### Layer 1: pi-ai (Unified LLM API)

Abstracts four distinct API formats (OpenAI Completions, OpenAI Responses, Anthropic Messages, Google Generative AI) into a single interface. Supports 20+ providers: Anthropic, OpenAI, Google, xAI, Groq, Cerebras, OpenRouter, Bedrock, Mistral, GitHub Copilot, Ollama, and any OpenAI-compatible endpoint.

Key capabilities:
- Streaming with AbortController support throughout the pipeline
- Tool calling with TypeBox schema validation and automatic AJV argument validation
- Thinking/reasoning level control across providers (minimal/low/medium/high/xhigh)
- Cross-provider context handoffs: switch models mid-conversation while preserving context. Anthropic thinking traces convert to `<thinking>` tags for other providers. Internal signed blobs handled transparently.
- Token and cost tracking (best-effort)
- Progressive JSON parsing for streaming tool call arguments
- Model registry parsing OpenRouter and models.dev data into TypeScript types

### Layer 2: pi-agent-core (Agent Loop)

Tools are defined with TypeBox schemas. The execute function returns separate `content` (sent to LLM) and `details` (UI-only) blocks -- a clean separation between what the model sees and what the user sees.

Event system: `agent_start`, `agent_end`, `turn_start`, `turn_end`, `message_start`, `message_update`, `message_end`, `tool_execution_start`, `tool_execution_update`, `tool_execution_end`.

Runtime control methods enable mid-session reconfiguration:
- `agent.steer()` -- interrupt current work and redirect
- `agent.followUp()` -- queue a message for after the agent naturally stops
- `agent.setModel()` / `agent.setThinkingLevel()` / `agent.setSystemPrompt()` / `agent.setTools()` -- swap model, reasoning level, prompt, or tool set without restarting
- `agent.replaceMessages()` -- for compaction

### Layer 3: pi-coding-agent (CLI)

Four operating modes:
1. **Interactive** -- standard terminal chat
2. **Print/JSON** -- one-shot output
3. **RPC** -- headless JSON protocol over stdin/stdout for IDE integration (strict LF-delimited JSONL framing)
4. **SDK** -- programmatic embedding via `createAgentSession()`

### TUI Implementation

Custom retained-mode terminal UI framework (not React/Ink). Differential rendering algorithm:
1. First render outputs all lines
2. Width changes trigger full clear and re-render
3. Normal updates find first differing line and re-render from there onward
4. Uses synchronized output escape sequences to prevent flicker

Zechner notes this is "dead simple" with memory overhead of "a few hundred kilobytes for very large sessions." Armin Ronacher: Pi is "written like excellent software. It doesn't flicker, it doesn't consume a lot of memory, it doesn't randomly break." This contrasts with Claude Code's Ink (React for terminals), where HN commenters reported "11 of the 16 ms budget per frame being wasted" on React overhead, causing persistent flickering.

## The Four-Tool Philosophy

Pi ships with only four tools:

| Tool | Description |
|------|-------------|
| `read` | File/image reading with line-based pagination (default 2,000 lines) |
| `write` | File creation/overwriting with automatic directory creation |
| `edit` | Surgical text replacement requiring exact match |
| `bash` | Command execution with optional timeout |

Everything else is built through extensions or agents spawning themselves via bash. This is the most aggressive tool minimalism in the CLI coding agent space.

## Anti-Patterns: What Pi Deliberately Omits

**No MCP support.** Zechner explicitly rejects Model Context Protocol. His argument: MCP servers consume 13,000-21,000 tokens describing tools that may never be used in a session. His alternative: "simple CLI tools with README files" where "the agent reads the README when it needs the tool, pays the token cost only when necessary." This is lazy-loading tool definitions via the filesystem rather than eager-loading via protocol negotiation.

**No built-in sub-agents.** Agents spawn themselves via bash when needed, preserving full observability. Zechner argues mid-session context gathering via sub-agents indicates poor workflow planning. This is a direct philosophical rejection of the sub-agent pattern used by Claude Code and OpenCode.

**No built-in plan mode.** Instead: TODO.md and PLAN.md files on disk. Zechner criticizes Claude Code's Plan Mode for requiring "approval of a shit ton of command invocations." File-based planning preserves full agent autonomy.

**No built-in permission system.** Users build their own safety via extensions if they want it.

## Extensibility System

Pi is described as "aggressively extensible" despite its minimal core:

- **TypeScript Extensions:** Modules with 20+ interception points for custom tools, commands, keyboard shortcuts, event handlers, UI components. Events include `context`, `session_before_compact`, `tool_call`, `before_agent_start`, `session_start`, `session_switch`.
- **Skills:** Self-contained capability packages following the Agent Skills specification, loading on-demand with setup instructions.
- **Prompt Templates:** Custom slash commands as markdown templates with argument placeholders.
- **Themes:** Customizable with live reload.
- **Pi Packages:** Bundle extensions/skills/themes, share via npm or git. Install with `pi install npm:@foo/bar` or `pi install git:github.com/user/repo`.

Armin Ronacher described the key principle: "if you want the agent to do something that it doesn't do yet, you don't go and download an extension...you ask the agent to extend itself."

## Configuration

- **AGENTS.md** -- project instructions loaded at startup from `~/.pi/agent/`, parent directories, and current directory (also supports CLAUDE.md for compatibility)
- **SYSTEM.md** -- replace or append to default system prompt per-project or globally
- **settings.json** -- global and per-project configuration
- **Custom models** -- full JSON configuration for non-standard models/providers
- **Keybindings** -- configurable via `~/.pi/agent/keybindings.json`

## Session Model

Sessions stored as JSONL files with **tree structure** -- not linear like Claude Code. This enables branching conversations. The `content` vs `details` split in tool results means the JSONL files contain exactly what the model saw, making them directly useful for post-processing, debugging, and training data extraction.

## Benchmark Performance

Pi with Claude Opus 4.5 was tested on Terminal-Bench 2.0 (five trials per task, leaderboard-eligible). Achieved competitive placement against Codex, Cursor, and Windsurf. Notably, Terminus 2 (the bench team's minimal agent providing raw tmux access without sophisticated tools) also ranks highly, supporting Zechner's thesis that minimal approaches are sufficient.

## Community Ecosystem

**OpenClaw connection:** OpenClaw (145,000+ GitHub stars in a single week) uses Pi's components as its agent runtime. Pi provides the engine; OpenClaw provides the user-facing product. Tencent launched a suite of AI products built on OpenClaw compatible with WeChat.

**Notable forks:**
- **oh-my-pi** (can1357) -- "batteries-included" fork adding hash-anchored edits, LSP integration for 40+ languages, Python cell execution, browser automation, subagents, and a memory system that extracts durable knowledge from past sessions.
- **pi-mono-py** (williepaul) -- Python port of the toolkit.

## The Three Kingdoms: Pi vs Claude Code vs OpenCode

A comparison from yun123.io characterizes the three as archetypes:

| Aspect | Claude Code | Pi | OpenCode |
|--------|------------|-----|----------|
| **Analogy** | "Rails" | "Arch Linux" | "VS Code" |
| **System prompt** | ~10,000 tokens | <1,000 tokens | Mid-range |
| **Core tools** | 8+ specialized | 4 minimal | Variable |
| **UI framework** | Ink (React) | Custom pi-tui | SolidJS + OpenTUI |
| **Data storage** | Filesystem | Filesystem (JSONL tree) | SQLite (Drizzle ORM) |
| **Security** | Secure-by-default | Trust-by-default | Smart defaults |
| **MCP** | Yes | No (intentionally) | Yes |
| **Sub-agents** | Yes (black box) | No (bash-spawned) | Yes (read-only) |

Zechner on Claude Code: "has turned into a spaceship with 80% of functionality I have no use for." Also cites persistent flickering and unpredictable system prompt/tool changes that "break my workflows."

## Relevance to lx

**Minimal system prompt thesis.** Pi's <1,000 token system prompt achieving competitive benchmark results with the same models validates a core lx design principle: the language should generate minimal, precise prompts rather than bloated instruction sets. lx programs should compile to tight context, not sprawling system messages.

**Four-tool sufficiency.** The claim that `read`, `write`, `edit`, `bash` are sufficient for competitive coding agent performance aligns with lx's "Terraform for agents" philosophy. lx can provide a small, composable tool primitive set rather than a kitchen-sink tool registry. Complex capabilities compose from simple ones.

**Anti-MCP argument and lazy tool loading.** Pi's rejection of MCP in favor of filesystem-based tool documentation that agents read on-demand is a lazy-loading pattern. In lx terms, this suggests tool bindings should be declarative but not eagerly injected into context. An `tools` block could list available tools without materializing their schemas until the agent requests them. Token-efficient tool discovery.

**Content vs details separation.** Pi's tool execute returning separate `content` (model-visible) and `details` (UI-only) is a clean pattern. lx tool definitions could have explicit `returns` (what enters context) and `displays` (what the user sees) sections.

**Tree-structured sessions.** JSONL with branching conversation trees rather than linear logs enables conversation forking, rollback, and exploration. lx's session model should support non-linear conversation histories natively.

**Agent self-extension.** "Ask the agent to extend itself" rather than downloading extensions. lx programs could include meta-programming constructs where agents modify their own tool bindings or spawn new tool definitions at runtime.

**Runtime reconfiguration.** `agent.steer()`, `agent.setModel()`, `agent.setTools()` for mid-session changes maps to lx's need for dynamic agent reconfiguration within a running workflow. The `agent` block could support `reconfigure` actions that swap model, tools, or prompt mid-execution.

**Monorepo layering as design template.** Pi's 7-package layering (LLM API → agent core → coding agent → TUI → web UI → Slack bot → GPU pods) is a clean separation of concerns. lx's own architecture (parser → evaluator → runtime → harness → CLI) follows a similar pattern. The pi-ai unified LLM abstraction is comparable to lx's backend trait system.