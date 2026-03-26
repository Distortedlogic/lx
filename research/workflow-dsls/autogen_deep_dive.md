# AutoGen: Deep Dive

## Identity

AutoGen is an open-source multi-agent framework from Microsoft Research for building LLM applications via multi-agent conversations. 56,198 GitHub stars, CC-BY-4.0 license. Original paper: "AutoGen: Enabling Next-Gen LLM Applications via Multi-Agent Conversation" (arXiv:2308.08155, COLM 2024). Won best paper at ICLR 2024 LLM Agent Workshop.

**Current status (March 2026):** Maintenance mode. Last release: v0.7.5 (September 2025). No releases since October 2025. Microsoft announced the **Microsoft Agent Framework** -- the merger of AutoGen and Semantic Kernel -- in October 2025, targeting GA by end of Q1 2026. AutoGen receives bug fixes and security patches only; new features go to Microsoft Agent Framework.

## The 0.2 → 0.4 Rewrite

AutoGen 0.4 was a complete ground-up rewrite adopting an asynchronous, event-driven architecture based on the actor model.

| Aspect | v0.2 | v0.4 |
|--------|------|------|
| Execution | Synchronous | Asynchronous (async/await) |
| Architecture | Monolithic | Layered: Core API + AgentChat + Extensions |
| Communication | Direct method calls | Actor-model message passing |
| Runtime | Single-process | SingleThreadedRuntime + distributed (gRPC) |
| Tool registration | Dual-agent (caller + executor) | Single-agent (tools parameter) |
| Model config | `llm_config` dict | Explicit model client instances |
| Group chat | GroupChatManager class | Team types: RoundRobin, Selector, Swarm, GraphFlow |
| Termination | `max_turns`/`max_round` | 11 composable conditions with `|` and `&` operators |
| Type safety | Loose | Enforced interfaces |
| Observability | Limited | Built-in OpenTelemetry |
| Languages | Python only | Python + .NET (gRPC interop) |

The 0.4 library splits into three packages: `autogen-core` (event-driven actor framework), `autogen-agentchat` (high-level task-driven API), `autogen-ext` (integrations).

## Architecture

### Core API (Low-Level)

Implements the actor model: each agent manages its own state and impacts others only by sending messages.

- `RoutedAgent` base class with `@message_handler` for typed message handling
- `SingleThreadedAgentRuntime` (primary) and experimental `DistributedAgentRuntime`
- Two communication patterns: **direct messaging** (RPC-like, request/response) and **topic broadcasting** (pub-sub)
- Runtime creates agent instances on-demand via factories when messages arrive
- `MessageContext` carries sender, RPC flag, and message ID

### AgentChat API (High-Level)

Built on Core, provides task-driven abstractions:

**Agent types:**
- `AssistantAgent` -- LLM-powered with tools, handoffs, system messages, memory
- `UserProxyAgent` -- proxy for human input
- `SocietyOfMindAgent` -- runs an entire group chat as inner monologue, appears as single agent externally
- `WebSurferAgent` -- pre-built web browsing agent
- Custom agents via `BaseChatAgent` subclass

**Team types:**
- `RoundRobinGroupChat` -- agents take turns sequentially
- `SelectorGroupChat` -- LLM selects next speaker based on conversation context, with customizable selector prompt and candidate filtering
- `Swarm` -- tool-based handoff routing (inspired by OpenAI Swarm); `HandoffMessage` signals transitions
- `MagenticOneGroupChat` -- pre-built 5-agent system for open-ended web/file tasks (Coder, Computer Terminal, File Surfer, Web Surfer, Orchestrator)
- `GraphFlow` (experimental) -- directed graph with conditional branching, parallel fan-out, join patterns, loops

## Conversation Patterns

### v0.2 Patterns

**Two-Agent Chat:** `initiate_chat()` between two agents. Returns `ChatResult` with `summary`, `chat_history`, `cost`.

**Sequential Chat:** Multiple two-agent chats chained via carryover mechanism -- summary from chat N becomes context for chat N+1. Uses `initiate_chats()` (plural).

**Group Chat:** All agents contribute to single thread. `GroupChatManager` orchestrates. Speaker selection: `'auto'` (LLM-based), `'round_robin'`, `'random'`, `'manual'`. Customizable via `allowed_or_disallowed_speaker_transitions` dict.

**Nested Chat:** Packages complex workflows into single agent interface. `register_nested_chats()` with condition function triggers internal sequential chats, uses final summary as reply.

### v0.4 Patterns

**SelectorGroupChat:** LLM-based speaker selection with custom `selector_prompt` (supports `{roles}`, `{history}`, `{participants}` variables) and `candidate_func` for filtering eligible next speakers.

**Swarm:** Tool-based handoff routing. Agents have `handoffs` parameter. `HandoffMessage` signals transitions. Critical: set `parallel_tool_calls=False` to prevent multiple simultaneous handoffs.

**GraphFlow:** Directed graph via `DiGraphBuilder`. Sequential, parallel (fan-out), join (fan-in), conditional branching, loops with `activation_group`/`activation_condition` ("all"/"any") semantics. `MessageFilterAgent` controls per-agent message visibility.

## Termination Conditions

11 composable types combinable with `|` (OR) and `&` (AND) operators:

`MaxMessageTermination`, `TextMentionTermination`, `TokenUsageTermination`, `TimeoutTermination`, `HandoffTermination`, `SourceMatchTermination`, `ExternalTermination`, `StopMessageTermination`, `TextMessageTermination`, `FunctionCallTermination`, `ContentFilter`.

This composability is one of AutoGen's strongest design decisions -- termination logic is declarative and combinable rather than imperative.

## Tool System

v0.4: `BaseTool` auto-generates JSON schemas from function signatures and type annotations. `FunctionTool` wraps Python functions. Tool call lifecycle: model generates JSON calls → framework parses `FunctionCall` objects → `tool.run_json(args, cancellation_token)` executes → `FunctionExecutionResult` returned → results sent back to model. Multiple tools execute concurrently via `asyncio.gather()`.

Built-in: `PythonCodeExecutionTool`, `LocalSearchTool`/`GlobalSearchTool` (GraphRAG), `HttpTool`, `LangChainToolAdapter`, MCP server integration.

## Code Execution

Docker isolation by default: launches container, executes code, terminates container per cycle. `DockerCommandLineCodeExecutor` is primary. Local execution via `use_docker=False` (development only). v0.4 adds `ACADynamicSessionsCodeExecutor` for Azure Container Apps.

Safety explicitly acknowledged: "Executing LLM-generated code poses a security risk to your host environment."

## Memory and State

`Memory` protocol: `add`, `query`, `update_context`, `clear`, `close`.

Implementations: `ListMemory` (chronological list), `ChromaDBVectorMemory` (similarity search), `RedisMemory` (distributed vector), `Mem0Memory` (cloud/local). Memory injected as `SystemMessage` objects, separate from conversation history.

State persistence: `save_state()` / `load_state()` on agents and teams. `BufferedChatCompletionContext(buffer_size=N)` limits message history.

## SocietyOfMindAgent

Runs an entire group chat as an inner monologue. The group chat's result becomes the agent's response to its caller. This enables hierarchical composition: a team can appear as a single agent within a larger team. Inner agents and their conversation are invisible to the outer context -- true encapsulation of multi-agent reasoning.

## The AG2 Fork Controversy

September 2024: original creators (primarily Chi Wang, Qingyun Wu) left Microsoft, created fork. November 2024: AG2 organization (`ag2ai/ag2`, 4,318 stars) takes control of PyPI packages `pyautogen`, `autogen`, and `ag2`. Microsoft blocked from pyautogen PyPI temporarily (since resolved).

Five distinct PyPI packages now exist: AG2 controls `autogen`, `pyautogen`, `ag2` (all identical, continuing 0.2 as v0.3.2). Microsoft controls `autogen-agentchat`, `autogen-core`.

The dispute fractured the community and introduced confusion about which "AutoGen" to use. AG2 maintains backward compatibility with 0.2; Microsoft's 0.4 is a complete rewrite with no migration path.

## Criticisms

**Non-deterministic behavior.** "The same prompt can trigger wildly different multi-agent dialogues, which destroys production reliability." Debugging described as "nearly impossible."

**Complex task breakdown.** "Limitations become apparent with complex tasks and multihop questions. Multihop questions require pulling information from disparate sources, making it difficult for AutoGen to provide accurate and consistent answers."

**Cost.** "Reliance on GPT-4 Turbo, which comes at a significant cost. Token rate can increase based on task complexity, potentially hitting token limits."

**Production readiness.** "While AutoGen serves as an excellent framework for research and prototype development, when it comes to building customer-facing applications, it falls short in reliability and accuracy."

**Ecosystem fragmentation.** The split into Microsoft AutoGen, AG2, and Microsoft Agent Framework caused "confusion and instability, forcing teams to choose between two different codebases with distinct feature sets and development philosophies."

## Production Deployments

Novo Nordisk (multi-agent framework for technical data insights). Enterprise interest spans 20+ sectors. 890,000 PyPI downloads by May 2024. Supply-chain optimization: 3x-10x reduction in manual interactions. Coding productivity: "more than 4x reduction in coding effort."

## Relevance to lx

**Actor model for agents is validated.** AutoGen 0.4's rewrite from synchronous method calls to async actor-model message passing confirms that agents should be independent message-processing entities. lx's agent model already follows this pattern.

**Composable termination conditions.** The 11 termination types with `|`/`&` operators is one of AutoGen's best designs. lx should support declarative termination conditions on agent loops that compose algebraically: `max_turns(10) | timeout(30s) | text_match("DONE")`.

**SocietyOfMindAgent as encapsulated multi-agent.** An entire team appearing as a single agent to its caller is hierarchical composition. lx should support this: a `crew` or `team` block that externally looks like a single agent but internally runs multi-agent coordination.

**GraphFlow validates graph-based orchestration.** Directed graphs with conditional branching, fan-out, join, and loops are necessary for real agent workflows. AutoGen added this late (experimental status). lx should provide this natively from the start.

**The rewrite lesson.** AutoGen's 0.2 → 0.4 rewrite broke everything. The synchronous, tightly-coupled 0.2 design couldn't scale. The lesson: async message passing and layered architecture (core runtime + high-level API) from the beginning, not as a rewrite.

**Ecosystem fragmentation is a warning.** The AG2 fork, five competing PyPI packages, and Microsoft Agent Framework merger show what happens when governance and architecture diverge. lx being a single-maintainer language avoids this, but should plan for extensibility without fragmentation.

**Conversation patterns as primitives.** Two-agent chat, group chat with speaker selection, nested chat, sequential chat with carryover -- these are recurring patterns that lx should provide as composable building blocks rather than requiring users to implement from scratch.