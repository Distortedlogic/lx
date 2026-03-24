# Claude Code: Terminal-Native Agent That Became #1 in 8 Months

Claude Code proves that **a terminal-native agent with deep model-tool co-training, a minimal 50-line orchestration loop, and a Think-Act-Observe-Repeat (TAOR) architecture can outperform IDE-integrated competitors** by letting all intelligence reside in the model and prompt structure rather than hard-coded decision trees. Launched May 2025, it overtook GitHub Copilot and Cursor within 8 months to capture 46% of "most loved" mentions.

## Overview

| Metric | Value |
|--------|-------|
| **Developer** | Anthropic |
| **Launch** | May 2025 |
| **Market Position** | #1 "most loved" AI coding tool (46%, vs Cursor 19%, Copilot 9%) |
| **Startup Adoption** | 75% at smallest companies |
| **Architecture** | TAOR (Think-Act-Observe-Repeat) loop, ~50 lines of orchestration |
| **Models** | Claude Sonnet 4.5/4.6, Claude Opus 4.6 (1M context) |
| **SWE-bench Verified** | Opus 4.5: 80.9%, Opus 4.6: 80.8% |
| **Claude Code scaffold** | 58.0% on SWE-bench Verified |
| **GitHub Actions** | [anthropics/claude-code-action](https://github.com/anthropics/claude-code-action) |

## Architecture

### TAOR Loop

The core loop is approximately 50 lines of orchestration logic:
1. **Think** — Analyze the current state, plan next action
2. **Act** — Execute a tool (read, write, execute, connect)
3. **Observe** — Process tool results
4. **Repeat** — Continue until task is complete

Four capability primitives: **Read**, **Write/Edit**, **Execute** (bash), **Connect** (MCP).

### CLAUDE.md Configuration

Hierarchical instruction system loaded into every conversation:
1. **Global** — `~/.claude/CLAUDE.md`
2. **Project** — `CLAUDE.md` in project root
3. **Directory** — `CLAUDE.md` in subdirectories
4. **User-specific** — `.claude/settings.local.json`

### Memory System

File-based persistent memory at `~/.claude/projects/<project>/memory/`:
- Types: user, feedback, project, reference
- MEMORY.md index file (always loaded, max 200 lines)
- Individual memory files with frontmatter (name, description, type)
- Automatically compressed as context approaches limits

### Auto-Dream (Memory Consolidation)

Background sub-agent that periodically consolidates auto-memory between sessions — the "REM sleep" counterpart to auto-memory's "awake" note-taking. Inspired by the "Sleep-time Compute" paper (arXiv:2504.13171) which showed idle-period pre-computation can reduce test-time compute ~5x while improving accuracy up to 18%.

**Four-phase process:**
1. **Orient** — Survey memory directory, read MEMORY.md index and topic files
2. **Gather Recent Signal** — Grep session transcripts (JSONL) for corrections, save commands, recurring patterns, architectural decisions
3. **Consolidate** — Merge new info into topic files, convert relative dates to absolute, delete contradicted facts, remove stale entries, merge overlapping entries
4. **Prune and Index** — Update MEMORY.md within 200-line threshold, remove obsolete pointers, resolve contradictions

**Constraints:**
- Read-only for source code (can only write memory files)
- Lock file prevents concurrent consolidation
- Runs in background without blocking active sessions
- Gated behind server-side feature flag (`tengu_onyx_plover`)

**Trigger conditions (both must be met):**
- `minHours: 24` — At most once per 24 hours
- `minSessions: 5` — At least 5 sessions since last run

**Access:** `/memory` toggle in Claude Code shows `Auto-dream: on/off`. Manual trigger via telling Claude "dream" or "consolidate my memory files". Undocumented `/dream` slash command exists in binary but not universally rolled out.

**Performance:** ~8-9 minutes to consolidate memory from 913 sessions.

### Sub-Agents

Claude Code spawns specialized sub-agents for parallel work:
- Each sub-agent gets its own context window
- Access to all tools (file editing, terminal commands)
- Types: general-purpose, Explore (codebase search), Plan (architecture), and custom
- Can run in foreground (blocking) or background (async)
- Worktree isolation option for git safety

### Agent Teams

Multi-agent coordination across Claude Code sessions:
- Multiple Claudes working together on the same codebase
- Thread sharing and coordination
- Parallel execution of independent tasks

## Key Features

### Hooks System

Event-driven callbacks for custom automation:

| Event | Description |
|-------|-------------|
| **PreToolUse** | Before tool execution (block/modify) |
| **PostToolUse** | After tool execution (format/validate) |
| **SubagentStop** | When sub-agent completes |
| + 9 more events | Session lifecycle, notifications, etc. |

### Skills and Plugins

- **Skills** — Reusable instruction sets loaded on demand (similar to Antigravity's SKILL.md)
- **Plugins** — Extend Claude Code with custom tools, slash commands, and hooks
- **Slash Commands** — `/commit`, `/review-pr`, custom commands via SDK
- Community ecosystem: 340+ plugins, 1,367+ agent skills

### Extended Thinking

- **Think Tool** — Lets Claude stop and reason before acting
- **Adaptive Thinking** — Claude dynamically determines when and how much to think
- Available in both Sonnet 4.6 and Opus 4.6

### GitHub Actions Integration

Official GitHub Action ([anthropics/claude-code-action](https://github.com/anthropics/claude-code-action)):
- Automated PR review
- Issue-to-PR workflows
- CI/CD integration
- Custom workflow recipes

### Claude Agent SDK

Renamed from Claude Code SDK — programmable in Python and TypeScript:
- Same tools, agent loop, and context management as Claude Code
- Custom slash commands via SDK
- Hook interception and control
- MCP integration
- Sub-agent spawning

## Competitive Position

Claude Code's moat is model-tool co-training: Claude models are specifically trained to work with the tool primitives, making the agent loop more reliable than competitors using generic API calls. The terminal-first approach appeals to senior engineers who prefer command-line workflows.

| Aspect | Claude Code | Cursor | Codex CLI | Gemini CLI |
|--------|-------------|--------|-----------|------------|
| **Surface** | Terminal | IDE | Terminal | Terminal |
| **Model** | Claude only | Multi | GPT only | Gemini only |
| **Agent Loop** | TAOR (50 lines) | IDE-integrated | Shell-first + apply_patch | Standard |
| **Thinking** | Adaptive + Think tool | Limited | Extended (Codex models) | Deep Think |
| **Sub-agents** | Yes (parallel, typed) | Background agents | No | No |
| **Memory** | File-based persistent + auto-dream consolidation | Project rules | AGENTS.md | GEMINI.md |
| **Hooks** | 12+ event types | No | No | Hooks system |
| **Skills/Plugins** | Yes (ecosystem) | .cursorrules | No | Extensions |
| **GitHub CI** | Official Action | BugBot | Codex cloud | No |
| **Open Source** | No | No | Apache 2.0 | Apache 2.0 |
