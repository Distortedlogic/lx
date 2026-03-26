# Anthropic Agent SDK: Deep Dive

## Identity

The Claude Agent SDK is the agent loop that powers Claude Code, packaged as a library. Released September 29, 2025. Python repo: `anthropics/claude-agent-sdk-python` (5.8k stars, MIT, v0.1.50). TypeScript: `anthropics/claude-agent-sdk-typescript` (v0.2.74). The SDK literally IS Claude Code -- ships the CLI binary bundled inside the package and communicates via JSON-line IPC.

At Anthropic internally: "Claude Code has begun to power almost all of our major agent loops" beyond coding into research, video creation, and note-taking.

## Five-Layer Architecture

### Layer 1: MCP (Connectivity)

Open standard for connecting agents to external tools/data. MCP servers provide integrations (GitHub, Slack, databases). Three transports: stdio (local), HTTP/SSE (remote), SDK MCP servers (in-process custom tools).

### Layer 2: Skills

Specialized capabilities as filesystem artifacts. A Skill is a directory with `SKILL.md` (YAML frontmatter + markdown instructions). Metadata discovered at startup (lightweight). Full content loaded on-demand when Claude determines relevance. Located in `.claude/skills/` (project) or `~/.claude/skills/` (user).

### Layer 3: Agent (The Core Loop)

The single-agent execution loop: prompt → evaluate → tool calls → observe results → repeat until no tool calls. Built-in tools: Read, Write, Edit, Bash, Glob, Grep, WebSearch, WebFetch.

### Layer 4: Subagents

Separate agent instances with fresh context windows. Only final result propagates back. Enables parallelization and context isolation. Cannot spawn their own subagents (no nesting).

### Layer 5: Agent Teams (Experimental)

Multiple Claude Code instances with a shared task list and inter-agent mailbox. Teammates communicate directly (not just back to lead). File-locked task claiming for race-condition safety.

## The Agent Loop

1. **Receive prompt.** System prompt + tool definitions + conversation history. `SystemMessage` with subtype `"init"`.
2. **Evaluate.** Claude responds with text and/or tool calls. `AssistantMessage` with text and tool_use blocks.
3. **Execute tools.** Read-only tools (`Read`, `Glob`, `Grep`) run **concurrently**. State-modifying tools (`Edit`, `Write`, `Bash`) run **sequentially**. Results feed back.
4. **Repeat** steps 2-3 until Claude produces a response with **no tool calls**.
5. **Return.** `ResultMessage` with final text, usage, cost, session ID, stop_reason.

**Stop conditions:** `success`, `error_max_turns`, `error_max_budget_usd`, `error_during_execution`, `error_max_structured_output_retries`.

When a tool is denied, Claude receives a rejection message and attempts a different approach.

## Tool Search Tool

The architecturally significant feature for scaling. Problem: multi-server MCP setups consume ~55k tokens in tool definitions before any work.

**Mechanism:**
1. Include tool search tool in `tools` list
2. Mark tools with `defer_loading: true` -- NOT loaded into context
3. Claude initially sees only tool search + non-deferred tools
4. When Claude needs a capability, calls tool search (regex or BM25)
5. API returns 3-5 `tool_reference` blocks pointing to matching tools
6. References automatically expanded into full definitions server-side
7. Claude calls discovered tools normally

**Token savings:** 85% reduction in tool definition overhead. Maximum 10,000 tools in catalog.

**Activation:** Default `true` (always on). Configurable: `auto` (activates at 10% context), `auto:N` (custom threshold), `false`.

Indexes tool names, descriptions, argument names, and argument descriptions.

## Context Management

Everything accumulates in context: system prompt, tool definitions, history, tool I/O. Content that stays the same across turns is prompt-cached.

**Auto-compaction:** When approaching context limit, SDK summarizes older history. Emits `SystemMessage` with subtype `"compact_boundary"`.

**Critical design:** Compaction replaces older messages with a summary. Specific instructions from early in conversation may not survive. Persistent rules belong in **CLAUDE.md** (re-injected every turn), not initial prompt.

**Customization:** CLAUDE.md summarization instructions, `PreCompact` hook for custom logic, manual `/compact` command.

## Subagents

```python
agents={
    "code-reviewer": AgentDefinition(
        description="Expert code reviewer",
        prompt="Analyze code quality...",
        tools=["Read", "Glob", "Grep"],
        model="sonnet",
    )
}
```

**Inherit:** Own system prompt, project CLAUDE.md, tool definitions (inherited or subset).
**Do NOT inherit:** Parent conversation history, parent tool results, parent system prompt, skills.
**Key constraint:** Cannot spawn their own subagents (no nesting).
**Resumption:** Capture `agentId` and session ID. Subagent transcripts persist independently and survive main conversation compaction.

## Agent Teams (Experimental)

Fundamentally different from subagents:

| | Subagents | Agent Teams |
|---|---|---|
| Communication | Report back to parent only | Teammates message each other directly |
| Coordination | Parent manages everything | Shared task list with self-coordination |
| Context | Own window, results summarized back | Fully independent context windows |

Components: team lead (main session), teammates (separate instances), task list (pending/in-progress/completed with dependencies, file-locked), mailbox (inter-agent messaging).

Plan approval: teammates can plan in read-only mode; lead reviews before implementation.

## Permission Model

Evaluated in strict 5-stage pipeline:

1. **Hooks** (allow, deny, or continue)
2. **Deny rules** (`disallowed_tools`) -- checked BEFORE `bypassPermissions`
3. **Permission mode** (`bypassPermissions` approves all reaching this step)
4. **Allow rules** (`allowed_tools`)
5. **`canUseTool` callback** -- runtime approval

Modes: `default` (no auto-approvals), `dontAsk` (deny unmatched), `acceptEdits` (auto-approve file ops), `bypassPermissions` (all tools), `plan` (no execution).

**Critical:** `allowed_tools` does NOT constrain `bypassPermissions`. Use `disallowed_tools` to block under bypass. `bypassPermissions` inherited by all subagents, cannot be overridden.

## MCP Integration

```python
mcp_servers={
    "github": {
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-github"],
        "env": {"GITHUB_TOKEN": os.environ["GITHUB_TOKEN"]}
    }
}
```

Transports: stdio, HTTP, SSE, SDK MCP servers (in-process). Tool naming: `mcp__<server>__<tool>`. Requires explicit `allowedTools` permission. Also loadable from `.mcp.json` at project root.

## System Prompt Design

Four approaches:
1. **Default (minimal):** Only tool instructions.
2. **Claude Code preset:** Full Claude Code system prompt. `system_prompt={"type": "preset", "preset": "claude_code"}`.
3. **Preset + append:** Preserves built-in + custom instructions.
4. **Custom string:** Replaces everything (lose built-in tools, safety, context).

CLAUDE.md loaded via `setting_sources=["project"]`. Acts as persistent memory. The `claude_code` preset does NOT automatically load CLAUDE.md.

## API Design

```python
async def query(
    prompt: str | AsyncIterable[dict],
    options: ClaudeAgentOptions | None = None,
) -> AsyncIterator[Message]
```

Key `ClaudeAgentOptions`: `max_turns`, `max_budget_usd`, `effort` (low/medium/high/max), `thinking` (adaptive/enabled/disabled), `output_format` (JSON schema for structured output), `sandbox` (network config, excluded commands), `betas` (context-1m), `fallback_model`.

Message types: `UserMessage`, `AssistantMessage`, `SystemMessage`, `ResultMessage`, `StreamEvent`. Plus task lifecycle: `TaskStartedMessage`, `TaskProgressMessage`.

Custom tools via `@tool` decorator → `create_sdk_mcp_server()`.

Multi-provider: Anthropic API (default), Amazon Bedrock, Google Vertex AI, Microsoft Azure AI Foundry.

## Comparison to OpenAI Agents SDK

| Aspect | Claude Agent SDK | OpenAI Agents SDK |
|--------|-----------------|-------------------|
| Architecture | Hooks + subagents | Handoffs + guardrails |
| Built-in tools | 8 (Read, Write, Edit, Bash, Glob, Grep, WebSearch, WebFetch) | 3 hosted (web search, file search, code interpreter) |
| Multi-agent | Subagents (delegation) + Agent Teams (collaboration) | Handoffs (conversation transfer) |
| Tool protocol | MCP (open standard) | Function calling (OpenAI-specific) |
| Control | Fine-grained 5-stage permission pipeline | Guardrails for I/O validation |
| Stars | ~6k | ~19k |

## Relevance to lx

**The agent loop is trivial.** Prompt → evaluate → tool calls → repeat. No complex state machine. lx's runtime should implement the same simple loop. The sophistication is in the TOOLS, CONTEXT MANAGEMENT, and ORCHESTRATION, not in the loop itself.

**Tool search is essential for scale.** Dynamic on-demand tool discovery with deferred loading is how you scale beyond 30 tools. lx should support lazy tool binding: declare tools in the program but don't inject their schemas until the agent actually needs them. The `defer_loading: true` pattern should be a default behavior.

**Context management is the real problem.** Auto-compaction, CLAUDE.md re-injection, subagent context isolation -- these are the hard engineering problems. lx's runtime needs equivalent machinery: automatic summarization when context is pressured, persistent instructions that survive compaction, and fresh context windows for spawned sub-agents.

**Subagents as context isolation boundaries.** Fresh context, only final result returns, cannot nest. This maps directly to lx's `spawn` semantics. The no-nesting constraint is interesting -- it prevents unbounded context fan-out but limits composability.

**Agent Teams introduce shared state.** Task list + mailbox for direct inter-agent messaging goes beyond simple delegation. lx's multi-agent primitives should support both patterns: parent-child delegation (subagents) AND peer-to-peer coordination (teams).

**The 5-stage permission pipeline.** Hooks → Deny → Mode → Allow → Callback. lx's capability/permission model for agent tool access should follow a similar layered evaluation. The key insight: deny rules must be checked BEFORE bypass modes to maintain safety invariants.

**The SDK IS the CLI.** The decision to ship Claude Code's CLI binary inside the SDK package (spawning it as a subprocess) is pragmatic engineering. It means the SDK has exactly the same capabilities as the CLI. For lx, this suggests the runtime binary should serve dual purpose: interactive CLI and embeddable library.