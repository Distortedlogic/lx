# Agent Session Management, State Persistence, and Checkpoint/Resume Patterns

## 1. Multi-Session Agent Architecture

Long-running agent tasks exceed single context windows. The dominant pattern splits work into an **initializer agent** (first session) and **worker agents** (subsequent sessions). Anthropic's harness for the Claude Agent SDK creates an `init.sh` script, a `claude-progress.txt` file, and an initial git commit. Every subsequent session uses the same system prompt but reads progress files and git logs to resume where the last session left off.

Google ADK divides context into **stable prefixes** (system instructions, agent identity, long-lived summaries) and **variable suffixes** (latest user turn, new tool outputs). When history exceeds a threshold, ADK triggers context compaction — an LLM summarizes older events over a sliding window and writes the summary back into the session.

OpenAI's Agents SDK collapses prior transcripts into a single assistant summary wrapped in a `CONVERSATION HISTORY` block, appending new turns as handoffs continue within a run.

## 2. Progress File Patterns

The canonical approach uses **JSON** for machine-readable state that agents modify reliably. Markdown is avoided for structured status tracking because models tend to rewrite entire markdown files rather than surgically updating fields.

Anthropic's feature list format uses JSON with a `passes` boolean agents flip:

```json
{
  "category": "functional",
  "description": "New chat button creates fresh conversation",
  "steps": ["Navigate to app", "Click new chat", "Verify empty state"],
  "passes": false
}
```

Manus uses `todo.md` as a live checklist, constantly rewriting it to push objectives into the model's recent attention span — combating "lost-in-the-middle" degradation. The filesystem serves as externalized memory: unlimited size, persistent, and directly operable.

The **three-file production pattern** (earezki.com, 2026) uses: `current-task.json` (immediate agent state), `memory/today.md` (daily action log), and `MEMORY.md` (long-term standing rules). Every agent reads all three before executing, updates during work, and clears `current-task.json` on completion.

## 3. Checkpoint/Resume Mechanisms

**LangGraph** saves graph state as checkpoints at every execution step, organized into threads. Production backends include `PostgresSaver` and `RedisSaver`. Time Travel enables replaying from any checkpoint without rerunning the entire workflow. Key API: `get_state_history()` lists checkpoints, `get_state()` fetches one, `update_state()` mutates before resuming.

```python
from langgraph.checkpoint.postgres import PostgresSaver
pool = ConnectionPool(conninfo=DB_URI, max_size=10)
with pool.connection() as conn:
    saver = PostgresSaver(conn)
    saver.setup()
```

**Temporal** separates deterministic workflow orchestration from non-deterministic activities (LLM calls, tool use). Workflows record every state change in an event history. On crash, Temporal replays recorded LLM decisions to resume exactly where execution stopped — no re-analysis, no duplicated API calls. OpenAI's Codex web agent runs on Temporal in production.

**AWS Bedrock AgentCore** provides session-level persistence with health status monitoring (`HealthyBusy`/`Healthy`), async task tracking via `add_async_task()`/`complete_async_task()`, and automatic 15-minute idle termination. Sessions can be reinvoked to reuse accumulated context.

## 4. Git as State Store

Git commits serve as natural checkpoints for coding agents. The **RALPH Loop** pattern (Read logs, Assess state, Labor on task, Push commit, Halt) treats git history as external memory between iterations. Agents read recent commit messages to reconstruct context.

**AgentGit** (built on LangGraph) introduces version-control semantics for agent state: `commit` snapshots, `revert` to last stable state on failure, and `branch` for parallel exploration. Failed strategies trigger automatic rollback to the last committed checkpoint.

**Claude Code Checkpoints** create automatic git-based snapshots before every AI-initiated edit, stored on a shadow branch. Recovery via `/rewind` offers three modes: conversation only, code only, or both. These are session-scoped (configurable 30-day retention), distinct from permanent git history.

Commit message conventions for agent workflows use typed prefixes (`feat`, `fix`, `refactor`) with co-author attribution, enabling `git bisect` for regression hunting across agent sessions.

## 5. Session Initialization Protocol

Every new coding agent session should execute this sequence:

1. `pwd` — confirm working directory
2. Read git log — understand recent changes and who made them
3. Read progress/state files — `claude-progress.txt`, `current-task.json`, or equivalent
4. Read feature/task list — select highest-priority incomplete item
5. Run `init.sh` or equivalent — start dev server, restore environment
6. Run tests — catch undocumented regressions before writing new code
7. Begin implementation — only after steps 1-6 succeed

Ordering matters: running tests before reading progress wastes context on failures already documented. Reading git logs before progress files catches commits made outside the agent system.

## 6. State Serialization

**What to persist**: plans, architectural decisions, failure reasons (with stack traces — Manus keeps failed actions in context so the model learns from them), task completion status, environment-specific commands, and file paths.

**What NOT to persist**: raw tool outputs (summarize instead), intermediate reasoning chains, full conversation transcripts (compress to structured summaries), large data payloads (store references/paths instead).

**Format choices**: JSON for machine state (agents parse and modify reliably), append-only logs for audit trails, structured event streams for replay. Avoid formats requiring schema migrations. Codex uses JSONL — one JSON object per event line — enabling streaming writes and partial reads.

Manus enforces **deterministic JSON serialization** and append-only contexts to maximize KV-cache hit rates. Cached tokens cost 10x less than uncached (`$0.30` vs `$3.00` per MTok on Claude Sonnet), making serialization stability a cost optimization.

## 7. Context Window Handoff

**The problem**: context degrades at each handoff. Full message history overwhelms receivers; lossy summarization strips evidence and reasoning; relationships between decisions, artifacts, and facts dissolve.

**Structured handoff format** (Blake Link's Session Handoff Protocol):
```
Task: [Context-Aware Title] - [Measurable Goal]
Current Status: [Quantifiable metrics, e.g., "30/90 tests passing"]
Key Areas to Focus On: [Prioritized task list]
Recent Accomplishments: [Work NOT to redo]
Development Environment: [Exact commands, paths, tools]
```

Google ADK performs **narrative casting** during handoff — recasting prior assistant messages as narrative context — and **action attribution** — marking tool calls from other agents so the receiver doesn't confuse them with its own capabilities.

**XTrace's analysis**: handoff should produce typed, linked objects with provenance tracking rather than text dumps. Context becomes a queryable asset instead of a lossy artifact.

Claude Code survives `/compact` because CLAUDE.md is re-read from disk and re-injected fresh. Instructions given only in conversation are lost — the rule is: anything that must survive compaction goes into a file.

## 8. Concurrent Agent State

Multiple agents on the same repo need isolation and coordination. Key patterns:

**Task locking via files**: `current-task.json` with agent ID and timestamp. Agents check before claiming work. The three-file pattern uses JSON handoff payloads (`sender`, `recipient`, `task`, `payload`, `timestamp`) as a filesystem message bus.

**Branch isolation**: each agent works on a dedicated feature branch. Devin 2.0 spins up parallel instances, each in an isolated cloud VM with its own IDE. Merge conflicts are resolved at PR time, not during agent execution.

**Shared state keys**: Google ADK has agents write to unique session state keys (`session.state["agent_name_result"]`) to prevent race conditions. When concurrent filesystem writes are needed, atomic writes (write-to-temp, then rename) prevent corruption.

**Coordinator pattern**: projects like Gas Town designate a coordinator agent with full workspace visibility that manages a merge queue for concurrent instances.

## 9. Recovery from Failures

**Idempotent operations**: check `current-task.json` status before execution to prevent duplicate work after restart. If status is `in_progress`, resume from last checkpoint rather than restarting.

**Temporal's approach**: activity retry policies with configurable timeouts. Failed activities retry automatically; the workflow resumes from the last successful step. Guarantees exactly-once behavior.

**Git-based rollback**: AgentGit reverts to last committed state on failure and attempts a different strategy. Claude Code checkpoints enable sub-30-second recovery — press Escape twice, select restoration scope.

**Orphaned state cleanup**: define retention policies (e.g., 30 days for checkpoints), clean up `current-task.json` entries where `started_at` exceeds a timeout threshold, archive completed session JSONL files to a separate directory.

**Manus's error philosophy**: failed actions stay in context intentionally. The model reads stack traces and implicitly updates its beliefs, reducing repeated mistakes — described as "one of the clearest indicators of true agentic behavior."

## 10. Real Implementations

**Claude Code**: CLAUDE.md files loaded every session. Auto-memory in `~/.claude/projects/<project>/memory/` with a `MEMORY.md` index (first 200 lines loaded at startup) and topic files read on demand. Sessions stored on disk; resume via `claude -c` or `claude --resume <id>`. Git-based checkpoints on shadow branches.

**OpenAI Codex**: JSONL session files in `~/.codex/sessions/`. `SessionState` protected by mutex containing `context_manager` (token tracking), `rollout` (event log), and `active_turn`. `SessionConfiguration` frozen at session start for consistency. Resume replays JSONL events. AGENTS.md for persistent instructions. Temporal integration for production durability.

**Devin 2.0**: parallel isolated cloud VMs per session. Playbooks serve as reusable system prompts across sessions. Session API with filtering, attribution, RBAC. Devin Wiki auto-indexes repos every few hours for cross-session knowledge.

**OpenHands**: event-sourced state management — an append-only event log records commands, edits, and results. V1 SDK redesign fixed the divergent-state problem (agent and sandbox processes could desync, corrupting sessions). Supports Docker/Kubernetes ephemeral workspaces.

**Manus**: event stream as session state, `todo.md` as live attention anchor, filesystem as unlimited externalized memory. KV-cache hit rate is the primary production metric. Token logit masking controls tool availability without modifying context (which would break cache). Deterministic serialization preserves cache across turns.

## Sources

- https://www.anthropic.com/engineering/effective-harnesses-for-long-running-agents
- https://code.claude.com/docs/en/memory
- https://agentic-patterns.com/patterns/filesystem-based-agent-state/
- https://earezki.com/ai-news/2026-03-09-the-state-management-pattern-that-runs-our-5-agent-system-24-7/
- https://blakelink.us/posts/session-handoff-protocol-solving-ai-agent-continuity-in-complex-projects/
- https://xtrace.ai/blog/ai-agent-context-handoff
- https://manus.im/blog/Context-Engineering-for-AI-Agents-Lessons-from-Building-Manus
- https://sparkco.ai/blog/mastering-langgraph-checkpointing-best-practices-for-2025
- https://temporal.io/blog/orchestrating-ambient-agents-with-temporal
- https://temporal.io/blog/of-course-you-can-build-dynamic-ai-agents-with-temporal
- https://docs.aws.amazon.com/bedrock-agentcore/latest/devguide/runtime-long-run.html
- https://vanducng.dev/2026/01/12/Google-Context-Engineering-Sessions-Memory-Summary/
- https://deepwiki.com/openai/codex/3.3-session-management-and-persistence
- https://understandingdata.com/posts/checkpoint-commit-patterns/
- https://skywork.ai/skypage/en/claude-code-checkpoints-ai-coding/1976917740735229952
- https://rlancemartin.github.io/2026/01/09/agent_design/
- https://mbrenndoerfer.com/writing/understanding-the-agents-state
- https://arxiv.org/html/2511.03690v1
- https://cognition.ai/blog/devin-2
- https://developers.googleblog.com/architecting-efficient-context-aware-multi-agent-framework-for-production/
