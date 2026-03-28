# Paperclip: Organizational Control Plane for Multi-Agent Teams

Paperclip demonstrates that **structuring AI agent teams as virtual companies with org charts, budgets, governance gates, and issue-based coordination can scale multi-agent operations without requiring a workflow DSL or execution framework**. By positioning itself as a control plane rather than an agent runtime, it cleanly separates organizational concerns (who does what, within what budget, with what authority) from execution concerns (how agents think, reason, and use tools). Launched March 2, 2026, it reached 36K+ stars in under 4 weeks.

## Repository Overview

| Metric | Value |
|--------|-------|
| **GitHub** | [paperclipai/paperclip](https://github.com/paperclipai/paperclip) |
| **Stars** | ~36,900 |
| **Forks** | ~5,400 |
| **Language** | TypeScript (Node.js backend, React frontend) |
| **License** | MIT |
| **Created** | March 2, 2026 |
| **Database** | PostgreSQL (PGlite for local dev) |
| **ORM** | Drizzle |
| **Schema Size** | 55+ tables |
| **Adapter Types** | 10 (claude_local, codex_local, hermes_local, openclaw_gateway, cursor, gemini_local, pi_local, opencode_local, http, process) |
| **Companion Repo** | [paperclipai/companies](https://github.com/paperclipai/companies) -- 16 pre-built companies, 440+ agents, 500+ skills |
| **Community Integration** | [NousResearch/hermes-paperclip-adapter](https://github.com/NousResearch/hermes-paperclip-adapter) -- 419 stars |

## Architecture

Control plane, not execution layer. Paperclip does not build agents, manage prompts, or define agent behavior. It manages the organization agents work within.

**Monorepo packages**: `server`, `ui`, `packages/db`, `packages/adapters`, `packages/shared`, `packages/plugins`, `cli`

**Key architectural properties**:
- Atomic execution -- task checkout and budget enforcement are atomic
- Persistent agent state -- agents resume across heartbeats without losing context
- Company-scoped isolation -- every entity belongs to exactly one company
- Immutable audit trail -- every tool call, API request, and decision is append-only logged
- Configuration versioning with rollback -- agent configs are revisioned

## Core Abstractions

| Concept | Description |
|---------|-------------|
| **Company** | Top-level isolation unit. Budget, brand, mission. Multiple per deployment. |
| **Agent** | AI worker with identity, adapter type, `reportsTo` hierarchy, budget cap, API key (`pcp_` prefix), config revision history |
| **Issue** | Unit of work. Statuses: backlog, todo, in_progress, in_review, blocked, done, cancelled. Supports subtasks, labels, attachments, full-text search, execution locking via `checkoutRunId` |
| **Goal** | Hierarchical objectives. Every task carries full goal ancestry so agents see the "why" |
| **Heartbeat Run** | Execution unit. Agent wakes, checks out issue, executes via adapter, logs results, sleeps. Tracks tokens (input/output/cache), billing cost, session state, stdout/stderr |
| **Routine** | Cron-scheduled recurring work. Concurrency policies: always_enqueue, skip_if_active, coalesce_into_existing. Catch-up logic (max 25 runs) |
| **Skill** | Reusable capability bundle (code + SKILL.md). Sources: GitHub, local, URL, catalog. Trust levels: markdown_only, assets, scripts_executables |
| **Approval** | Governance gate for consequential actions (e.g., hiring agents). Board members approve/reject; agents cannot self-authorize |
| **Budget Policy** | Per-company/agent/project. 80% utilization -> warning + incident. 100% -> hard stop + agent pause + approval to resume. Hierarchical enforcement (company > agent > project) |
| **Plugin** | Lifecycle-managed extensions. Registers tools, event handlers, background jobs, UI launchers, data sources, metrics. Zod-validated parameter schemas |

## Execution Model -- Heartbeat Loop

No workflow DSL, no DAGs, no pipelines. Work flows through implicit organizational structure:

```
Goal definition -> Issue creation -> Assignment -> Heartbeat execution -> Delegation/completion
```

The heartbeat cycle:

```
Wake (timer/assignment/event)
  -> Check budget
  -> Resolve workspace
  -> Build execution context (instructions, goal ancestry, session state)
  -> Invoke adapter
  -> Stream logs
  -> Persist results
  -> Evaluate session compaction
  -> Release issue lock
  -> Promote next queued task
```

**Concurrency**: Per-agent `maxConcurrentRuns` (max 10). Agent start operations serialized via per-agent promise chains. Issues use optimistic execution locking -- stale locks from terminated runs are adoptable.

**Session management**: Sessions persist across heartbeats via adapter-specific codecs. Session compaction evaluates run count, input tokens, and age limits, generating handoff markdown for context transfer when rotating sessions.

## Agent Coordination

- **Org chart hierarchy**: `reportsTo` relationships forming a tree. System enforces no cycles via chain traversal. CEO delegates to CTO, CMO, etc.
- **Issue-based communication**: Agents talk through issue comments. `@mention` system enables agents to wake each other
- **Pull-based model**: Agents wake on scheduled heartbeats and event triggers (task assignment, mentions). Not push-based messaging
- **Goal context propagation**: Every task carries full goal ancestry up the org chart
- **No direct agent-to-agent messaging**: All coordination is mediated through the issue system, org chart delegation, and comment threads

The CEO agent's SOUL.md establishes 13 operating principles including "Never implement yourself -- even small tasks belong to your reports."

## Error Handling and Recovery

**Two-level error catching**:
- Inner catch: adapter execution failures (run finalized, issue lock released)
- Outer catch: setup failures before adapter invocation (cleanup guaranteed)

**Process loss detection**: Background reaper identifies runs with missing process handles, checks PID liveness via `process.kill(pid, 0)`, auto-retries (max 1 per lost process) with preserved context snapshots.

**Budget enforcement as prevention**: Hard stops pause agents before they can run. Hierarchical budget blocks prevent execution when limits are exceeded.

**Session compaction**: When sessions grow too large (run count, token count, or age), system rotates with handoff markdown to prevent context window overflow.

**Config rollback**: `rollbackConfigRevision()` reverts bad agent configuration changes.

**Stale lock adoption**: New runs can adopt checkout locks from terminated runs rather than being permanently blocked.

## Tool Integration

Paperclip does not manage tools directly. Each agent brings its own tools through its adapter runtime. Paperclip adds organizational tools via Skills (capability bundles materialized into workspace at runtime) and the Plugin SDK (Zod-validated tool registration).

The CEO agent gets management-level tools via the `paperclip-create-agent` skill and API access for hiring agents, creating issues, and managing the company programmatically.

## Configuration -- No DSL

Configuration is markdown + YAML + JSON API:

| Layer | Format |
|-------|--------|
| **AGENTS.md** | Agent instruction bundles injected into context. Role-specific bundles (default vs CEO) with HEARTBEAT.md, SOUL.md, TOOLS.md |
| **Company Portable Format** | COMPANY.md (YAML frontmatter), .paperclip.yaml, agents/\*/AGENTS.md, projects/\*/PROJECT.md, tasks/\*/TASK.md, skills/\*/SKILL.md |
| **Server Config** | 100+ `PAPERCLIP_*` environment variables |
| **Runtime Config** | React dashboard or REST API for adapter configs, budget policies, routines, workspace policies |

## Comparison to Other Frameworks

| Dimension | Paperclip | LangGraph | CrewAI | AutoGen |
|-----------|-----------|-----------|--------|---------|
| **Abstraction** | Control plane / company | State machine / graph | Role-based crews | Conversational teams |
| **Workflow definition** | Implicit via org chart + issues | Explicit directed graph | Task sequences | Multi-turn conversations |
| **Agent runtime** | Bring your own (10 adapters) | Built-in (LangChain) | Built-in | Built-in |
| **Coordination** | Issue delegation + heartbeats | Graph edges + state | Process types | GroupChat selector |
| **Budget control** | First-class with hard stops | None | None | Azure billing |
| **Governance** | Approval gates, immutable audit | None | None | None |
| **Persistence** | Full session + state across reboots | Checkpointing | None | None |
| **Multi-tenancy** | Built-in company isolation | None | None | None |
| **DSL** | None (deliberate) | Python graph API | Python decorators | Python/YAML |
| **Language** | TypeScript/Node.js | Python | Python | Python/.NET |

Paperclip operates at a fundamentally different layer. LangGraph, CrewAI, and AutoGen are agent frameworks that define how agents think and execute. Paperclip is an organizational layer that sits above any of those and manages the company structure. You could run CrewAI or LangGraph agents inside Paperclip via the HTTP adapter.

## Relevance to lx

**No DSL by design** -- Paperclip explicitly chose not to build a workflow definition language, relying on org-chart-based implicit coordination. This leaves the gap that lx fills: declarative workflow definitions that compile to agent coordination patterns.

**Heartbeat pull model vs. lx channels** -- Agents wake on schedules, pull work, execute, sleep. Robust but high-latency. lx's `tell`/`ask` and channel primitives are fundamentally more responsive.

**Issue-mediated only** -- No direct message passing, no channels, no pub/sub between agents. lx's channel-based messaging is strictly more expressive.

**Single metaphor** -- Company/org-chart is the only organizational abstraction. Cannot express pipelines, DAGs, fan-out/fan-in, conditional routing, or state machines. lx's flow/task/agent composability is more general.

**Worth adopting** -- Budget as a first-class constraint (not bolted on) and the adapter pattern (runtime-agnostic control plane) are strong patterns for production agentic systems.

**Company templates as pattern library** -- [paperclipai/companies](https://github.com/paperclipai/companies) with 16 pre-built companies, 440+ agents, 500+ skills shows what a library of reusable workflow patterns could look like for lx.
