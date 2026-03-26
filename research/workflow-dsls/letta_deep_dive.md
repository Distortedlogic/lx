# Letta (MemGPT): Deep Dive

## Identity

Letta (formerly MemGPT) is a framework for building stateful AI agents with self-managed memory. Created by Charles Packer, Sarah Wooders et al. at UC Berkeley. 21.6k GitHub stars, Apache 2.0, $10M seed funding. The MemGPT paper: "MemGPT: Towards LLMs as Operating Systems" (arXiv:2310.08560, October 2023).

Core thesis: "The most powerful characteristics of a useful AI agent -- personalization, self-improvement, tool use, reasoning and planning -- are all fundamentally memory management problems."

## The OS-Inspired Memory Architecture

The central analogy: just as an OS provides the illusion of unlimited memory by paging between RAM and disk, MemGPT provides the illusion of unlimited context by paging between the LLM's context window ("main context" = RAM) and external databases ("external context" = disk).

Main context is split into three contiguous sections:

1. **System instructions** (read-only, static): MemGPT control flow, memory hierarchy, function schemas. ~1,076 tokens.
2. **Working context / Core Memory** (fixed-size, read/write): Unstructured text writable only via function calls. Split into `<persona>` and `<human>` blocks. ~86 tokens.
3. **FIFO queue** (rolling message history): Messages, system alerts, function call I/O. First index contains a recursive summary of previously evicted messages.

## Three-Tier Memory

### Core Memory (Always in Context)

Fixed-size, editable blocks always present in the LLM's context window. Named sections with label, description, value, and character limit (default 2,000 chars).

**Persona block:** Agent identity, behavioral guidelines, task instructions.
**Human block:** Key details about the current user.

Editing tools:
- `core_memory_append(name, content)` -- add without replacing
- `core_memory_replace(name, old_content, new_content)` -- surgical replacement; empty new_content = delete

### Recall Memory (Conversation History)

Full uncompressed history of ALL events: user messages, system messages, reasoning, tool calls, return values. Indexed for case-insensitive string matching and date-range queries.

- `conversation_search(query, page)` -- string matching across history
- `conversation_search_date(start_date, end_date, page)` -- date-filtered

### Archival Memory (Vector Storage)

Infinite-capacity semantic database. PostgreSQL with pgvector, HNSW-indexed approximate cosine similarity. Overflow for core memory and general-purpose long-term storage.

- `archival_memory_insert(content)` -- "phrase memory contents such that it can be easily queried later"
- `archival_memory_search(query, page)` -- embedding-based semantic search

### Self-Management Loop

The agent decides what to store, retrieve, and evict through its own reasoning:

1. Memory pressure warning at **70% context capacity** -- system message warns agent
2. Agent reasons about what's important (inner monologue)
3. Agent calls `core_memory_append` or `archival_memory_insert` to preserve data
4. Agent may call `core_memory_replace` to compress core memory
5. If agent doesn't act, queue manager force-evicts at **100%** with recursive summarization -- evicted messages summarized with existing summary, creating progressive compression where older messages have less influence

An external memory summary section lists counts of archival/recall memories, giving the agent awareness of what exists outside its context.

## The Tool-Call-as-Reasoning Pattern

MemGPT's fundamental design decision: **all agent output is expressed as tool calls.** Even sending a message to the user requires calling `send_message`. The assistant's `content` field serves as **inner monologue** -- private reasoning not shown to the user, constrained to under 50 words.

"When you send a message, the contents of your message are your inner monologue (private to you only), this is how you think."

## Heartbeat Mechanism

Every tool includes a `request_heartbeat` boolean parameter:

- `true`: Function output added to context, LLM processor immediately re-invoked. Enables multi-step tool chaining within a single user turn.
- `false` (yield): System pauses until next external event.

This is essentially a coroutine yield/resume mechanism. The agent "yields" by default and must explicitly request continuation, preventing infinite loops while enabling multi-step reasoning.

The agent's brain runs "in short bursts" triggered by events: user events, system events (memory pressure, function completions), and timed events (scheduled heartbeats enabling unprompted execution). `pause_heartbeats` can suppress for up to 6 hours.

## Letta V1 Architecture (2026)

### What Changed

The transition from `memgpt_agent` to `letta_v1_agent` is a fundamental shift:

**MemGPT:** Every action is a tool call including messages. Reasoning injected through `thinking` parameter. Continuation via `request_heartbeat`. Termination by default (heartbeat=false).

**V1:** Heartbeats and `send_message` deprecated. Uses "native reasoning and direct assistant message generations." Tool calling no longer required for basic operation.

"Today's LLMs are trained to be adept at agentic patterns such as multi-step tool calling, reasoning interleaved within tool calling, and self-directed termination conditions."

### The Reasoning Dilemma

A critical tension:

**Native reasoning** (V1): Opaque (developers see only summaries), immutable (modifications void API requests), provider-locked. But delivers frontier performance because agents stay "in-distribution."

**Prompted reasoning** (MemGPT): Transparent (full visibility), modifiable, portable across models. But potentially out-of-distribution for heavily post-trained models.

"The performance differential between these approaches remains unclear and likely varies by use case."

### What V1 Lost

1. **Prompted reasoning for non-reasoning models** -- GPT-4o mini etc. no longer generate reasoning. No fallback mechanism.
2. **Heartbeats** -- V1 "no longer understands heartbeat concepts." Custom prompting required for sleep-time compute scenarios.
3. **Tool rule coverage** -- Rules can't apply to AssistantMessage since it's no longer a tool call.

## Server Architecture

Letta treats agents as persistent services, not ephemeral library objects:

"While most frameworks are libraries that wrap model APIs, Letta provides a dedicated service where agents live and operate autonomously."

Agents continue to exist when your application isn't running, maintain state in a database, can be accessed from multiple applications simultaneously, and run autonomously on the server. REST API on port 8283. Python and TypeScript client SDKs.

All state (memories, messages, reasoning, tool calls) persisted in PostgreSQL or SQLite. Never lost, even once evicted from context.

### Multi-Agent Communication

Three built-in tools:
- `send_message_to_agent_async` -- fire-and-forget
- `send_message_to_agent_and_wait_for_reply` -- synchronous request/response
- `send_message_to_agents_matching_all_tags` -- broadcast to tagged agents

Five orchestration patterns: supervisor-worker, parallel execution, round-robin, producer-reviewer, hierarchical teams.

Agents can share memory blocks -- multiple agents reference the same block, updates immediately visible across all.

### Agent File Format (.af)

Open format for serializing stateful agents. Packages system prompts, editable memory, tool configurations, LLM settings into single human-readable JSON. Enables checkpointing, version control, portability. Secrets nulled on export.

## Sleep-Time Compute

Dual-agent system for proactive memory management:

**Primary agent:** Handles user interactions, tool execution. Cannot modify its own core memory blocks.
**Sleep-time agent:** Runs asynchronously during idle periods. Manages both agents' memory. No latency constraints. Generates "clean, concise, and detailed memories."

Solves the MemGPT problem: "Memory management, conversation, and other tasks are all bundled into a single agent." The two agents can use different models -- fast model for primary, stronger/slower for sleep-time.

Research shows "Pareto improvement in model performance" on math benchmarks, shifting computational load from user-facing interactions to idle periods.

## Memory vs RAG

The Letta team's sharp distinction: "While retrieval (or RAG) is a tool for agent memory, it is not 'memory' in of itself." True agent memory requires persistent, learnable state. RAG is one tool in the memory toolbox.

Three failures of current approaches:
1. **Context pollution** -- embedding-based RAG "pollutes the context with irrelevant information." Reasoning models "explicitly discourage" excessive in-context learning.
2. **Lack of memory consolidation** -- agents don't reflect on experiences to derive new insights.
3. **Stateless architecture assumption** -- frameworks bake in the assumption that agents are stateless.

## Production Deployments

**Bilt Rewards:** Over 1 million individually-tailored agents in production. Non-technical stakeholders manage agent prompts through the ADE. "All the latency in the end-to-end system is 99+ percent just inference time" -- Letta adds negligible overhead.

**Letta Code:** Ranked #1 model-agnostic open source agent on Terminal-Bench.

## Criticisms

**Runtime lock-in.** "Letta's lock-in is architectural. Switching away means rewriting not just your memory layer but your entire agent infrastructure." Letta doesn't slot into LangGraph/CrewAI/AutoGen as a memory service -- it replaces your stack.

**Learning curve.** "Memory tiers, ADE setup, and agent configuration take hours to internalize -- not minutes."

**V1 tradeoffs.** Sacrificed transparency for performance. Heartbeat mechanism deprecated despite being a powerful continuation primitive.

**No temporal reasoning.** "Archival memory doesn't model time explicitly" -- limiting compliance and audit use cases.

## Tool Rules (MemGPT-era)

Graph-like constraints on agent tool usage:
- **InitToolRule:** Forces specific tool first
- **ToolRule (with children):** Specifies which tools must follow
- **TerminalToolRule:** Marks tool as terminating execution

Only supported by models with structured output support (OpenAI gpt-4o/gpt-4o-mini).

## Relevance to lx

**Three-tier memory as language constructs.** Core (always-in-context, editable), recall (searchable history), archival (semantic vector store) map directly to language-level memory primitives. An lx `memory` block with `core`, `recall`, and `archival` sections where agents have explicit `read`/`write`/`search` operations on each tier.

**Heartbeats as continuation control.** The `request_heartbeat` pattern is a coroutine yield/resume mechanism. In lx: agents "yield" by default between turns, and must explicitly request continuation (`continue` or equivalent). This prevents infinite loops while enabling multi-step reasoning. A cleaner primitive than arbitrary loop limits.

**Tool-call-as-reasoning gives total observability.** Making all output flow through typed tool calls gives the framework complete visibility and control. lx could support this as a mode: `strict` agents where all actions including messages go through typed channels, vs `native` agents where the model reasons and messages freely.

**Context window as managed resource.** The OS metaphor of memory pressure warnings, eviction policies, and recursive summarization treats context as scarce. lx should model context budget explicitly -- agents have a declared context budget, and the runtime manages compaction automatically. The two-threshold system (70% warning, 100% force-evict) is a useful default.

**Sleep-time compute.** Background processing during idle periods maps to async background tasks in lx. An `on_idle` handler or background agent that consolidates memories between invocations.

**The V1 lesson.** As models improve at agentic patterns natively, framework-level control (heartbeats, forced tool calling) becomes overhead. lx should design control structures that gracefully degrade -- use native model capabilities when available, fall back to prompted mechanisms when not. Make this declarative: `reasoning: native | prompted | auto`.

**Shared memory blocks.** Multiple agents referencing the same memory block with immediate visibility is a shared-state pattern. lx could support `shared memory` blocks that multiple agents in a workflow can read/write, with the runtime handling consistency.

**Agent persistence.** Agents as persistent entities (not ephemeral function calls) with state that survives across sessions is fundamental for long-running workflows. lx agents should be serializable/resumable by default.