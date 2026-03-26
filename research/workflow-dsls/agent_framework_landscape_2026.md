# Agent Orchestration Framework Landscape (March 2026)

Every major AI lab now ships its own agent framework. The independent ecosystem adds another dozen. This survey identifies which frameworks introduce genuinely novel patterns relevant to designing an agentic workflow language.

## Framework Census

| Framework | Stars | Language | Maintainer | Status |
|-----------|-------|----------|------------|--------|
| AutoGen | 56.2k | Python | Microsoft Research | Maintenance mode → MS Agent Framework |
| CrewAI | 47.2k | Python | CrewAI Inc (Joao Moura) | Active, v1.12 |
| DSPy | 32.9k | Python | Stanford NLP | Active |
| Semantic Kernel | 27.4k | C#/Py/Java | Microsoft | Merging into MS Agent Framework |
| LangGraph | 27.4k | Python/TS | LangChain Inc | Active, v1.0+ |
| smolagents | 26.2k | Python | HuggingFace | Active |
| Mastra | 22.3k | TypeScript | ex-Gatsby team (YC W25) | Active, v1.0 |
| Letta | 21.6k | Python | Letta Inc (ex-MemGPT) | Active, V1 |
| OpenAI Agents SDK | 20.1k | Python/TS | OpenAI | Active, v0.13 |
| Pydantic AI | 15.7k | Python | Pydantic (Samuel Colvin) | Active |
| Julep | ~7k | Python/YAML | Julep AI | Self-host only (hosted shut down Dec 2025) |
| Agency Swarm | 3.9k | Python | VRSEN (Arsenii Shatokhin) | Active |

---

## Tier 1: Novel Patterns Worth Deep Study

### DSPy (Stanford NLP)

Three abstractions: **Signatures** (declarative I/O specs for LLM calls -- field names are semantic), **Modules** (composable units implementing prompting strategies like ChainOfThought, ReAct), and **Optimizers** (automatically synthesize effective prompts, generate few-shot examples, fine-tune weights).

The fundamental insight: **prompt engineering should be automated, not hand-crafted.** You declare what you want (signature), pick a strategy (module), and let the optimizer find the best prompts. Teams inevitably reinvent DSPy's core patterns anyway -- "an ad hoc, informally-specified, bug-ridden implementation of half of DSPy."

Criticisms: steep learning curve (unfamiliar abstractions), compilation cost (100-500 LLM calls, $20-50, 10-30 min), 4.7M monthly downloads vs LangChain's 222M despite technical superiority.

**lx relevance:** The most architecturally interesting framework. Signatures (typed I/O specs for LLM calls) should be a first-class language construct. Optimization should be separable from execution. The criticism that DSPy is "hard because the abstractions are unfamiliar" is exactly what a purpose-built language solves -- the abstractions become the language itself.

### Letta (formerly MemGPT)

**LLM-as-Operating-System** -- the model manages its own memory like an OS manages RAM and disk. Three-tier memory: **Core Memory** (always in context, analogous to RAM -- editable blocks), **Recall Memory** (complete interaction history, searchable), **Archival Memory** (processed knowledge in vector/graph DBs, analogous to disk). Agents actively move data between tiers.

Letta V1 (2026) abandoned the MemGPT-style "everything is a tool call" pattern. Key insight: forced reasoning through tool calls gave transparency and portability but hurt performance on frontier models. V1 uses native model reasoning capabilities instead. **The Reasoning Dilemma:** prompted reasoning gives transparency/portability; native reasoning gives performance. Tension remains unresolved.

**lx relevance:** Three-tier memory as first-class language constructs. Agent self-management of context window. The V1 evolution is a critical design lesson: a language should not force all agent actions through a single mechanism. The reasoning dilemma needs a language-level solution -- make reasoning strategy declarative.

### Julep

**YAML-defined workflow DSL** with 16 step types: Prompt, Tool Call, Evaluate, If-else, Switch, Foreach, Map-reduce, Subworkflow, Wait for Input, Set, Get, Sleep, Return, Yield, Log, Error. Expression syntax with `$` prefix for dynamic evaluation. GitHub Actions-style workflow definitions.

Hosted backend shut down December 2025 -- project viability uncertain.

**lx relevance:** The most directly comparable project. Validates the need for a declarative workflow language for agents. Demonstrates why YAML is the wrong substrate: complex workflows become unwieldy, expression syntax is awkward, no type system, no IDE support, no compile-time validation. The 16 step types are a feature checklist for lx's workflow primitives.

### SLANG (Super Language for Agent Negotiation & Governance)

Three primitives: `stake` (produce and send), `await` (wait for input), `commit` (terminate with result). **Deliberately not Turing-complete** -- limited surface area for learnability and LLM-generability. Actor-model foundation with explicit message passing, no shared state.

Example: `agent Researcher { stake gather(topic) -> @Analyst }` / `agent Analyst { await data <- @Researcher; commit }`

**lx relevance:** Competing design philosophy. The deliberate non-Turing-completeness for LLM generability is a bold claim worth evaluating. Three-primitive minimalism vs lx's richer construct set is a fundamental design tension.

---

## Tier 2: Study Specific Patterns

### OpenAI Agents SDK

Core abstraction: the **handoff** -- agents transfer control to each other explicitly via tool calls. Each agent has instructions, model, tools, and a list of handoff targets. Handoffs are represented as tools to the LLM (e.g., `transfer_to_refund_agent`).

**Guardrails** run in parallel with agent execution and fail fast. Input guardrails before execution, output guardrails after, tool guardrails on every function-tool invocation. Built-in tracing collects LLM generations, tool calls, handoffs, guardrails, and custom events.

**lx relevance:** Handoffs-as-tool-calls reuse an existing mechanism rather than inventing a new one -- important design lesson. Guardrails-as-parallel-validators is a pattern lx should adopt.

### smolagents (HuggingFace)

~1,000 lines of code. Two agent types: **CodeAgent** (generates and executes Python code) and **ToolCallingAgent** (standard JSON tool calls). CodeAgent is primary -- the LLM writes Python that executes, with `final_answer()` raising an exception to terminate.

Code-as-action reduces steps/LLM calls by ~30% and achieves superior benchmark performance because code composes naturally (function nesting, variable reuse, loops).

Security is serious: `import_modules()` allows arbitrary code execution, `LocalPythonExecutor` is "not a security sandbox."

**lx relevance:** Code is a better action representation than JSON. For lx, agents should "act" by writing lx programs, not by emitting JSON tool calls. lx's constrained language design inherently avoids the arbitrary code execution vulnerability.

### Semantic Kernel (Microsoft)

Central orchestrator with plugin system. Merging with AutoGen into Microsoft Agent Framework. Five pre-built orchestration patterns: **Sequential**, **Concurrent**, **Handoff**, **Group Chat**, and **Magentic** -- all sharing a unified interface so you can swap patterns without rewriting agent logic.

Process Framework (GA planned Q2 2026) extends into deterministic business workflow orchestration. A2A messaging as an open standard plus MCP support. Multi-language (C#, Python, Java).

**lx relevance:** The five orchestration patterns as a taxonomy. Swappable orchestration without rewriting agent logic is a design goal lx should target.

### Pydantic AI

Type safety as the core value: end-to-end type checking so code that passes type-checking works at runtime. **Dependency injection** for agents -- a type-safe way to customize behavior, especially for unit tests and evals. **Composable capabilities** bundle tools, hooks, instructions, and model settings into reusable units.

**lx relevance:** Type-safety-first validates lx's approach. Dependency injection for agents separates behavior from configuration, making testing trivial. Composable capability bundles are similar to agent "traits" or "profiles."

---

## Tier 3: Monitor

### LangGraph

Directed graphs where nodes are functions and edges define transitions (including conditional). Supports cycles (not DAG-only), which is critical for agent loops. Checkpointing with pluggable persistence (Redis, Postgres). Human-in-the-loop as first-class. Subgraphs for composable agent hierarchies.

Criticisms: steep learning curve, overhead for simple workflows, re-implements control flow at runtime with worse tooling than a real language.

**lx relevance:** Validates that DAG-only is insufficient for agents. The criticism that "you're re-implementing control flow in a worse runtime" is exactly what lx solves. Checkpointing/persistence model worth studying.

### Mastra

TypeScript-native from ex-Gatsby team. Workflows as graph-based state machines with typed step boundaries. YC W25, $13M funding, 22.3k stars.

No durable execution state by default -- if a process crashes, workflows cannot resume. TypeScript-only.

**lx relevance:** Weakness (no durable execution) is an opportunity. Typed step boundaries validate lx's type system approach.

### Agency Swarm (VRSEN)

Organizational metaphor: agents as employees with explicit communication topology (directed graph of who can talk to whom). `SendMessage` tool for structured inter-agent communication. No ambient conversation; every message intentional and traceable.

**lx relevance:** Explicit communication topology is declaring a message-passing graph at the language level.

---

## Emerging DSL Efforts

### PayPal's Declarative Agent Workflow Language (arXiv 2512.19769)

Language-agnostic DSL separating agent logic from implementation. Configuration-driven: adding tools or changing behavior = spec change, not code deployment. Claims 60% reduction in dev time, 3x deployment velocity. Complex workflows in ~50 lines vs 500+ imperative. Supports A/B testing of agent strategies natively.

### Google ADK (Agent Development Kit)

Python, TypeScript, Go, Java. Event-driven runtime. ADK Python 2.0 Alpha adds graph-based workflows. Sessions and Memory Bank reached GA early 2026.

### Anthropic Agent SDK

The same agent loop that powers Claude Code, packaged as a library. Five-layer architecture: MCP → Skills → Agent → Subagents → Agent Teams. Tool Search Tool for dynamic tool discovery (85% token reduction while maintaining full tool library access).

### Microsoft Agent Framework

Merger of AutoGen + Semantic Kernel. Graph-based workflows + agents. RC status, targeting 1.0 GA by end of Q1 2026. Five orchestration patterns from Semantic Kernel + AutoGen's conversational patterns in a unified framework.

---

## Key Design Ideas for lx

1. **From DSPy:** Signatures as first-class typed I/O specs for LLM calls. Optimization separable from execution.
2. **From Letta:** Tiered memory (core/recall/archival) as language constructs. Agent self-management of context.
3. **From Julep:** 16 step types as a feature checklist (prompt, tool_call, evaluate, if-else, switch, foreach, map-reduce, wait_for_input, set/get, sleep, return, yield, log, error).
4. **From OpenAI Agents SDK:** Handoffs as a primitive. Guardrails as parallel validators.
5. **From smolagents:** Code > JSON for action representation. The language IS the action format.
6. **From Semantic Kernel:** Swappable orchestration patterns without rewriting agent logic.
7. **From SLANG:** Deliberate non-Turing-completeness for learnability and LLM generability.
8. **From Pydantic AI:** Dependency injection for agent testing. Composable capability bundles.
9. **From PayPal DSL:** A/B testing of agent strategies as a native capability.
10. **From AutoGen:** Composable termination conditions. SocietyOfMindAgent (team-as-agent encapsulation).