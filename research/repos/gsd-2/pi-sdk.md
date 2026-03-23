# GSD-2: Pi SDK Foundation

GSD-2 is built on top of the Pi SDK — a terminal-native coding agent harness supporting 20+ LLM providers. Understanding Pi is essential to understanding GSD-2's extension model, session management, and runtime behavior.

## What Pi Is

Pi is a coding agent runtime that provides:
- Multi-model agent sessions with branching history
- Extension system for custom tools, commands, and UI
- Terminal UI (TUI) with keyboard shortcuts
- Compaction engine for context limits
- Tool executor for file operations and shell commands
- Session persistence as JSONL
- RPC mode for embedding in other applications

GSD-2 is a **branded app built on Pi** — it sets `PI_PACKAGE_DIR` to a shim directory and `piConfig.name: "gsd"`, giving it its own identity while inheriting all Pi infrastructure.

## Four Modes of Operation

| Mode | Purpose | Use Case |
|------|---------|----------|
| **Interactive (TUI)** | Full terminal UI | Default `gsd` command |
| **RPC** | JSON-RPC over stdin/stdout | Headless child process, VS Code extension |
| **JSON** | Single-shot JSON output | Scripting, piping |
| **Print** | Single-shot text output | Quick queries |

## The Agent Loop

Pi's core loop:

```
User Input → before_agent_start hook → System Prompt Assembly →
  agent_start → turn_start → context assembly →
    LLM generates response → tool_call → tool_result →
  turn_end → agent_end →
Loop or Exit
```

GSD hooks into this at `before_agent_start` (inject GSD system context) and `agent_end` (auto-mode advancement).

## Session Model

Sessions are persisted as JSONL trees supporting branching:
- Branch: fork conversation at any point
- Resume: continue from a prior session
- Compact: summarize old messages to free context

Per-directory scoping: `~/.gsd/sessions/<escaped-cwd>/`.

## Extension Runtime

### Registration

Extensions register via:
- `registerTool(definition)` — give the LLM new tools
- `registerCommand(name, handler)` — slash commands
- `registerUI(component)` — TUI components
- Event hooks (before/after agent turns, tool calls, etc.)

### ExtensionContext

Provides access to:
- `ctx.agent` — agent session, model, messages
- `ctx.ui` — TUI rendering, dialogs, overlays
- `ctx.fs` — filesystem operations
- `ctx.git` — git operations
- `ctx.process` — process management
- `ctx.env` — environment variables
- `ctx.settings` — configuration

### ExtensionAPI

Allows extensions to:
- Register tools, commands, UI components
- Modify system prompt per-turn
- Control compaction (message summarization)
- Manage models/providers
- Override built-in tools
- Create new sessions (`ctx.newSession()`)

### Packaging

Extensions distributed via npm with `install` command. Pi's resource loader searches:
1. `~/.gsd/agent/extensions/` (GSD bundled, synced on launch)
2. `~/.pi/agent/extensions/` (Pi bundled, if not shadowed)

## Tool System

Built-in tools: read, bash, edit, write, grep, find, ls.

Custom tools via extension:
```typescript
registerTool({
  name: "my_tool",
  description: "Does something",
  parameters: { /* JSON Schema */ },
  handler: async (params) => { /* implementation */ }
})
```

## Compaction Engine

When context approaches limits:
- `session_before_compact` hook fires
- GSD can save `continue.md` or block compaction during auto-mode
- Old messages summarized to free space
- Context pressure monitor at 70% usage

## Context Pipeline

```
User input → before_agent_start →
  System prompt = static sections + dynamic injections (extension hooks) →
    agent_start → tool_call/tool_result hooks →
  turn_end → agent_end
```

Extensions can inject context at multiple points:
- System prompt modification (per-turn)
- before_agent_start (inject project state)
- Context files (CLAUDE.md, AGENTS.md)

## Multi-Model Support

Pi supports 20+ LLM providers via `@gsd/pi-ai`:
- Anthropic (Claude)
- OpenAI (GPT, o-series)
- Google (Gemini)
- Mistral
- AWS Bedrock
- Azure OpenAI
- OpenRouter (proxy to many providers)
- GitHub Copilot
- And more via custom provider registration

Per-phase model selection means different models for different phases of work.

## TUI System

Pi's TUI provides:
- Component interface (render, keyInput, focusable)
- Built-in components: Text, Padding, HStack, VStack, Button, Input, Select, Checkbox
- Dialog methods: info, warn, error, question, choice
- Persistent UI: overlays, sidebar, status bar
- Custom editors
- Tool and message renderers
- Theming: color palettes, style attributes
- IME support for input methods
- Keyboard shortcuts
- Message queue for async communication

The GSD dashboard overlay (`Ctrl+Alt+G`), visualizer overlay, and guided flows all use this system.

## Why GSD Built on Pi (Not From Scratch)

1. **Multi-provider support** — 20+ LLM providers out of the box
2. **Session management** — JSONL persistence, branching, compaction
3. **Extension system** — hooks, tools, commands, UI without forking
4. **TUI framework** — rich terminal UI components
5. **Tool executor** — file ops, shell, grep built in
6. **RPC mode** — headless child process communication
7. **MCP support** — Model Context Protocol integration
8. **Community ecosystem** — Pi packages and extensions

GSD adds the **state machine, hierarchical planning, dispatch pipeline, verification gates, cost tracking, and autonomous execution** on top of Pi's agent infrastructure.
