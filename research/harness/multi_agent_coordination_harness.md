# Multi-Agent Coordination at the Harness/Infrastructure Level

Research compiled March 2026. Covers coordination plumbing -- not agent logic itself.

## 1. Coordination Topologies

**Star (orchestrator-worker):** A central orchestrator decomposes tasks and dispatches to workers. Anthropic's research system uses a lead agent that spawns 3-5 subagents in parallel, waits synchronously for results, then synthesizes. Codex caps concurrent threads at 6 (`agents.max_threads`) with per-worker timeouts (`job_max_runtime_seconds`, default 1800s). Best for well-decomposable tasks. Failure mode: orchestrator is a single point of failure; workers cannot self-coordinate.

**Mesh (peer-to-peer):** Agents communicate directly without a coordinator. OpenCode enables full-mesh messaging where any teammate messages any other by name, using JSONL append-only inboxes at `team_inbox/<projectId>/<teamName>/<agentName>.jsonl`. Reduces coordinator burden but requires explicit backpressure (OpenCode currently lacks it). Failure mode: message storms; the Gemini agent that generated ~50 identical "task complete" messages in a loop.

**Pipeline (sequential handoff):** Agents chain linearly -- each receives the prior agent's output. Azure Architecture Center calls this "Pipes and Filters with AI agents." Deterministic ordering, no agent choice about next step. Failure mode: errors in early stages propagate unchecked downstream.

**Blackboard (shared memory):** All agents read/write a shared space. LbMAS (Han & Zhang, July 2025) splits the blackboard into public space (all agents) and private spaces (for debate). A control unit selects which agents act each round based on query + blackboard contents + agent capability descriptions. A dedicated conflict-resolver agent detects contradictions, triggers private-space discussion, and reintegrates revised messages. Achieved 13-57% improvement over baselines at lower token cost (4.7M tokens vs 13.8M for uncontrolled systems on MATH). Failure mode: throughput bottleneck on shared state; without a cleaner agent, tokens balloon.

**Hierarchical (manager-of-managers):** Cursor's successful architecture uses three levels: Planners (explore codebase, create tasks), Workers (execute without coordinating with each other), and Judge agents (decide whether to continue). Their failed attempt at equal-status agents with locking saw 20 agents degrade to 2-3 throughput. Google ADK supports nesting via `AgentTool` wrappers for parent-to-child invocation.

## 2. Message Passing Patterns

**Direct messaging:** OpenCode uses two-layer messaging -- JSONL append-only inbox for persistence plus session injection as synthetic user messages to the LLM. Messages carry id, from, text, timestamp, read status. Delivery receipts follow actor-model patterns (sender learns of reception via reply message).

**Pub/sub and event streams:** Manus coordinates sub-agents through a shared event stream -- a chronological log capturing user messages, agent actions, and observations, concatenated into the prompt each cycle. Confluent's design proposes Kafka-based coordination where orchestrators distribute command messages across topic partitions, workers consume as a consumer group, and outputs go to a second topic.

**Shared context objects:** LangGraph uses a shared state object that carries messages, tool outputs, and embeddings across all graph nodes. Two modes: shared scratchpad (all work visible to all agents) and independent scratchpads (only final responses appended to global state). ADK uses `session.state` with `output_key` -- agents write to unique keys to prevent race conditions.

**Protocol formats:** A2A (Google, April 2025, now under Linux Foundation) defines task lifecycle with Agent Cards for capability discovery, JSON-RPC over HTTP/SSE, and gRPC support in v0.3. MCP provides tool-level interop. The three-layer stack (MCP for tools, A2A for agents, WebMCP for web) is becoming consensus architecture as of early 2026 under the Agentic AI Foundation (OpenAI, Anthropic, Google, Microsoft, AWS, Block).

## 3. Task Routing and Delegation

**Description-based routing:** ADK's AutoFlow transfers execution based on agent descriptions. The orchestrator's instruction field guides routing; sub-agents are matched by their declared capabilities. CrewAI's hierarchical process auto-assigns a manager that delegates based on role descriptions.

**Capability matching:** A2A Agent Cards advertise capabilities in JSON. Client agents query cards to find the best remote agent for a task. Codex's `spawn_agents_on_csv` reads a CSV and spawns one worker per row with role-specific model configs and instructions.

**Dynamic/learned routing:** A NeurIPS 2025 paper introduces a reinforcement-learning "puppeteer" that dynamically routes tasks between agents based on evolving problem states, improving accuracy while reducing costs compared to static routing.

**Handoff patterns:** OpenAI Agents SDK treats handoffs as first-class tool calls with `handoff_input_filter` for context filtering. Agno uses explicit `transfer_task_to_member()`. LangGraph Swarm issues a `Command` that updates shared state and switches `active_agent`. Google ADK performs narrative reframing during handoffs -- prior Assistant messages are re-cast as narrative context (e.g., `[For context]: Agent B said...`) so the receiving agent does not misattribute prior actions to itself.

## 4. Shared Memory and State

**Blackboard systems:** See topology section above. The control unit formula: `ConU(q, B, {D1...Dn}) -> {Ei1...Eij}` selects agents per round. A critic agent detects hallucinations and forces rethinking. A cleaner agent removes redundant messages to control token growth.

**Shared file systems:** Manus uses persistent file-based memory -- `todo.md` checklists, `notes.txt` for intermediate results, working files as external memory when context overflows. Codex workers report results via `report_agent_job_result`; unreported results are marked as errors in exported CSV.

**Concurrency control:** OpenCode uses in-memory read-write locks with writer priority (single-process only, not cross-process). State transitions are validated against explicit maps with `guard: true` (skip if shutdown) and `force: true` (bypass during recovery). Cursor's failed experiment: file-level locking caused 20 agents to degrade to 2-3 effective throughput because agents held locks too long. Their second failed experiment: optimistic concurrency made agents risk-averse, avoiding difficult tasks.

**Git worktrees as isolation:** Codex uses built-in worktree support so multiple agents work on the same repo without conflicts -- each gets an isolated copy. Clash (open-source CLI) uses `git merge-tree` three-way merges via the gix library to detect conflicts between worktree pairs read-only, integrated as a Claude Code `PreToolUse` hook on Write/Edit/MultiEdit operations. Devin 2.0 runs each instance in an isolated VM. Augment Code's Intent uses coordinator/specialist/verifier with filesystem-level worktree isolation.

## 5. Conflict Resolution

**Prevention via isolation:** The dominant 2025 pattern is preventing conflicts rather than resolving them. Git worktrees give each agent an isolated branch. Merge sequentially: pick one agent's work first, rebase remaining branches. Clash detects conflicts before writes happen but does not resolve them.

**Contradictory outputs:** Azure recommends choosing aggregation strategy by task type: voting/majority-rule for classification, weighted merging for scored recommendations, LLM-synthesized summary when results need reconciliation. The blackboard architecture's conflict-resolver agent triggers private-space debate between disagreeing agents.

**Resource ownership:** Augment Code recommends "each database table, API endpoint, or file belongs to exactly one agent." This prevents conflicts structurally rather than resolving them after the fact.

**Optimistic concurrency failure:** Cursor found that optimistic concurrency control caused agents to become risk-averse -- they avoided editing files that other agents might touch, leading to incomplete work on difficult tasks.

## 6. Agent Lifecycle Management

**Spawning:** OpenCode uses fire-and-forget spawning (returns immediately) with auto-wake -- messaging an idle agent restarts its prompt loop. Codex orchestrates spawning centrally, including automatic spawning and explicit CSV-driven batch spawning.

**Monitoring:** OpenCode tracks ten fine-grained execution states (running, waiting, cancelling, etc.) via dual state machines -- coarse lifecycle (ready/busy/shutdown) and fine-grained execution status. Anthropic saves plans to Memory to persist context before 200K token truncation.

**Killing:** OpenCode uses retry-based cancellation: 3 attempts at 120ms intervals via `SessionPrompt.cancel()`, then forced state transition. AWS Bedrock's `StopRuntimeSession` immediately terminates sessions and releases resources.

**Runaway prevention:** OpenCode's no-automatic-restart pattern: after crashes, busy agents transition to ready but prompt loops do not restart unless a human re-engages. Rationale: "You don't wake up to find four agents burning API credits all night." Recommended rate limits: max_retries=3 per task, cooldown_after_failure=60s, max_actions_per_session=50. Azure recommends limiting group chat to 3 or fewer agents. Codex defaults to max 6 concurrent threads.

**The anti-pattern:** Cursor's equal-status-with-locking approach at 20 agents effectively ran at 2-3 throughput. The fix was hierarchical roles (planner/worker/judge) where workers never coordinate with each other.

## 7. Context Isolation vs. Sharing

**Full context handoff:** LangGraph's shared scratchpad gives all agents full visibility. Simple but verbose and expensive. Anthropic reports multi-agent systems use ~15x more tokens than single chats; agents use ~4x more than chat.

**Minimal context (agents-as-tools):** ADK's default suppresses ancestral history, passing only the latest query and necessary artifacts. Token reduction of 60-80% in typical multi-turn conversations. ADK distinguishes "Agents as Tools" (focused calls, no history) from "Agent Transfer" (controlled inheritance with scope limits).

**Narrative reframing:** When ADK transfers between agents, prior Assistant messages are re-cast as narrative context with tags. Tool calls from other agents are marked/summarized so the receiving agent does not misattribute actions.

**Context compilation pipeline:** ADK transforms Session storage into Working Context through ordered processors: instructions, identity, content selection, caching, planning, code execution. Stable prefixes (system instructions) enable model prefix caching; variable suffixes (latest turns) are dynamic.

**Sub-agent barriers:** OpenCode prevents sub-agents from accessing team coordination channels via permission deny rules and tool visibility hiding (removed from tool list entirely). Teammates relay findings rather than sub-agents broadcasting.

## 8. Fan-Out/Fan-In Patterns

**Parallel dispatch:** Anthropic's research system spawns 3-5 subagents in parallel for complex queries. Parallel tool calling cut research time by up to 90%. Codex's `spawn_agents_on_csv` spawns one worker per CSV row with `max_concurrency` control.

**Result aggregation:** Codex workers call `report_agent_job_result` exactly once; missing reports are marked as errors. Results export to CSV with `job_id`, `item_id`, `status`, `last_error`, `result_json`. AWS scatter-gather stores intermediate results in S3/SQS before an aggregator merges.

**Quorum/voting:** Azure's concurrent pattern supports voting/majority-rule for classification tasks. If 1-2 agents fail, quorum of remaining agents (e.g., 3 of 5) still produces a decision. Multi-agent debate patterns use iterative rounds until consensus.

**Best-of-k:** Anthropic's system with Claude Opus 4 lead + Sonnet 4 subagents outperformed single-agent Opus 4 by 90.2% on internal evaluations. The key is subagents as intelligent filters: each explores independently, returning compressed findings to the lead.

## 9. Error Propagation in Multi-Agent Systems

**Cascading failures:** Research shows error propagation is the primary reliability bottleneck. A single root-cause failure compounds at every subsequent step. In multi-agent systems, transitive trust chains mean wrong output propagates without verification. 14 distinct failure modes across 3 categories: specification/design (41.77%), inter-agent misalignment (36.94%), task verification (remainder).

**Circuit breakers:** Isolate misbehaving agents before contamination. If a tool fails N times, break the circuit and return graceful failure. Defense-in-depth: architectural isolation with trust boundaries, runtime verification with multi-agent consensus, and automated cascade pattern detection with kill switches.

**Partial failure handling:** Anthropic's approach: "letting the agent know when a tool is failing and letting it adapt works surprisingly well." Systems resume from where the error occurred rather than restarting. Manus agents retry up to 3 times, then shift strategies, then escalate to user as last resort.

**Deployment coordination:** Anthropic uses rainbow deployments for their stateful agent webs -- gradually shifting traffic from old to new versions while maintaining both simultaneously, preventing mid-process disruption to active research sessions.

## 10. Real Coordination Implementations

**Claude Research:** Orchestrator-worker with lead agent + 3-5 parallel subagents. Synchronous execution (waits for each batch). Subagents receive objective, output format, tool guidance, task boundaries. Token usage explains 80% of performance variance. Rainbow deployments for stateful agent webs.

**Codex:** Central orchestration with worktree-based isolation. Max 6 concurrent threads, 30-min per-worker timeout. CSV-driven batch fan-out. Workers inherit sandbox policy. Results aggregated via `report_agent_job_result`.

**Devin 2.0:** Isolated VMs per instance. Bidirectional file sync with local project. Multi-agent dispatch where one Devin assigns sub-tasks to other instances. Cloud-based virtual environments with full Ubuntu workspace.

**Manus:** Event-stream-based coordination. File-based state (`todo.md`, working files). Sub-agents in isolated sandboxes (Ubuntu containers). Wide Research uses fully general-purpose Manus instances as subagents (not role-constrained). $100M ARR in 8 months.

**AutoGen 0.4:** Conversation-centric coordination via GroupChatManager. Three-step loop: speaker selection (LLM-based), response, broadcast. Supports RoundRobinGroupChat (static), SelectorGroupChat (dynamic), and custom orchestrators.

**CrewAI:** Role-based (Manager/Worker/Researcher). Sequential, hierarchical, and consensus processes. Manager auto-assigned in hierarchical mode. Planning mechanism generates step-by-step workflow injected into all tasks. 1.4B agentic automations in production.

**LangGraph:** Graph-based execution with shared state and per-thread checkpointing. Supervisor as "agent whose tools are other agents." Swarm handoffs via Command objects. Time-travel debugging, human-in-the-loop interrupts. ~400 companies in production, ~90M monthly downloads.

**Cursor (multi-agent coding):** Failed at equal-status + locking (throughput collapse) and optimistic concurrency (risk aversion). Succeeded with Planner/Worker/Judge hierarchy. Workers push changes when done; Judges decide continuation.

## Sources

- https://www.anthropic.com/engineering/multi-agent-research-system
- https://developers.openai.com/codex/multi-agent/
- https://developers.googleblog.com/developers-guide-to-multi-agent-patterns-in-adk/
- https://developers.googleblog.com/en/architecting-efficient-context-aware-multi-agent-framework-for-production/
- https://learn.microsoft.com/en-us/azure/architecture/ai-ml/guide/ai-agent-design-patterns
- https://arize.com/blog/orchestrator-worker-agents-a-practical-comparison-of-common-agent-frameworks/
- https://arxiv.org/html/2507.01701v1
- https://arxiv.org/html/2503.13657v1
- https://dev.to/uenyioha/porting-claude-codes-agent-teams-to-opencode-4hol
- https://github.com/clash-sh/clash
- https://mikemason.ca/writing/ai-coding-agents-jan-2026/
- https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/
- https://cloud.google.com/blog/products/ai-machine-learning/agent2agent-protocol-is-getting-an-upgrade
- https://gist.github.com/renschni/4fbc70b31bad8dd57f3370239dccd58f
- https://medium.com/@takafumi.endo/agent-native-development-a-deep-dive-into-devin-2-0s-technical-design-3451587d23c0
- https://www.augmentcode.com/guides/why-multi-agent-llm-systems-fail-and-how-to-fix-them
- https://docs.aws.amazon.com/prescriptive-guidance/latest/agentic-ai-patterns/parallelization-and-scatter-gather-patterns.html
- https://blog.langchain.com/langgraph-multi-agent-workflows/
- https://dev.to/askpatrick/rate-limiting-your-own-ai-agent-the-runaway-loop-problem-nobody-talks-about-3dh2
- https://blog.promptlayer.com/multi-agent-evolving-orchestration/
- https://adversa.ai/blog/cascading-failures-in-agentic-ai-complete-owasp-asi08-security-guide-2026/
