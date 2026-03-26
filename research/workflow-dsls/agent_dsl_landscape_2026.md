# Agent DSL Landscape (March 2026)

Beyond the 12 frameworks surveyed in [agent_framework_landscape_2026.md](agent_framework_landscape_2026.md), a parallel ecosystem of **actual DSLs, specification standards, and language-level approaches** to agent programming has emerged. This survey covers systems that introduce genuine language design ideas — custom syntax, type systems, compilation, or formal semantics — rather than Python/TypeScript libraries with decorator APIs.

---

## True DSLs and Custom Languages

### MeTTa (Meta Type Talk) — SingularityNET/ASI Alliance

**URL:** https://github.com/trueagi-io/hyperon-experimental
**Stars:** ~1.8k | **License:** Apache 2.0 | **Status:** Active (core infrastructure of ASI Alliance AGI effort)

A programming language designed for AGI that combines functional, logical, and process-calculus programming. Part of the OpenCog Hyperon platform. Programs are subgraphs of an Atomspace metagraph; the core operation is querying and rewriting portions of Atomspaces.

**Key pattern:** Self-modifying programs. Code can rewrite itself at runtime via metagraph rewriting. Handles recursive self-improvement natively. Replaces Idris as the language for AI-DSL service composition.

**Syntax:** Custom Lisp-like. Programs operate on typed atoms within a shared knowledge graph.

**Relevance:** The only language designed from the ground up for agents that reason about and modify their own programs. The metagraph-as-program-state model is fundamentally different from DAG/pipeline approaches — programs are data structures that agents traverse and rewrite.

### AI-DSL — SingularityNET

**URL:** https://github.com/singnet/ai-dsl
**Stars:** ~150 | **Status:** Phase 2 complete; continued under MeTTa/Hyperon

A description language for autonomous agent interoperability. Agents use AI-DSL to declare their capabilities and I/O types so other agents can discover and compose ad-hoc workflows. Originally Idris-based (dependently typed), migrating to MeTTa.

**Key pattern:** Agent self-description and automatic composition. No predefined I/O format required — the type system handles compatibility checking.

**Relevance:** Addresses agent discovery and composability at the type level. lx's `agent` blocks already define capabilities declaratively; AI-DSL shows what happens when you push that to dependent types and automatic workflow assembly.

### APPL (A Prompt Programming Language)

**URL:** https://github.com/appl-team/appl
**Stars:** ~500 | **Published:** ACL 2025 | **Status:** Active

Extends Python with native prompt constructs. Compositors and Definitions make prompts first-class composable program units. The runtime automatically schedules independent LLM calls for parallel execution.

**Key pattern:** Prompt composition with automatic parallelism. Prompts are structured as composable units, and the scheduler detects independence for concurrent execution without programmer annotation.

**Relevance:** The automatic parallelism detection is interesting — lx could infer which agent invocations are independent and parallelize them without explicit `spawn` directives.

### POML (Prompt Orchestration Markup Language) — Microsoft Research

**URL:** https://github.com/microsoft/poml
**Stars:** ~3k | **Published:** August 2025 | **Status:** Active

HTML-like markup language for structured prompts. Semantic components (`<role>`, `<task>`, `<example>`, `<let>`), CSS-like styling for decoupling content from presentation, Jinja-style templating. Rich data integration — pulls in Word docs, CSVs, images, audio, folders.

**Key pattern:** Separation of prompt content from presentation. CSS-like styling means the same prompt can render differently for different models or contexts without changing the content.

**Relevance:** The content/presentation separation is a real insight. lx prompts could have "style sheets" that adapt formatting to the target model's preferred instruction format — one prompt definition, multiple renderings.

### PromptML (Prompt Markup Language)

**URL:** https://github.com/narenaryan/promptml | https://www.promptml.org/
**Stars:** ~300 | **Status:** Moderate

DSL for defining AI prompts as code with annotations: `@prompt`, `@context`, `@objective`, `@vars`, `@instructions`, `@constraints`. Write one prompt definition, generate multiple natural language variations targeting different LLMs.

**Key pattern:** "AI Prompt As Code (APaC)" — single source of truth for prompts with multi-target generation.

### SynthLang

**URL:** https://github.com/ruvnet/SynthLang
**Stars:** ~300 | **Status:** Moderate

Hyper-efficient prompt language using logographical scripts (Chinese characters) and symbolic constructs inspired by Ithkuil and Lojban. Claims 40-70% token reduction by using information-dense symbolic notation.

**Key pattern:** Token compression through symbolic encoding. A "compression language" for LLM communication rather than a workflow language.

**Relevance:** The token efficiency angle matters for lx's budget system — a compressed prompt format could stretch token budgets significantly.

### PromptLang

**URL:** https://github.com/ruvnet/promptlang
**Stars:** ~200 | **Status:** Experimental

A programming language that exists entirely within LLM prompts. Functions, variables, conditionals, loops defined in a syntax that the LLM interprets as a program. GPT-4 IS the runtime.

**Key pattern:** LLM-as-interpreter. Conceptually the inverse of every other framework — no external runtime, the language model executes the program directly. SLANG's "LLM-as-runtime" mode is a more rigorous version of this idea.

### Impromptu — SOM-Research (Langium-based)

**URL:** https://github.com/SOM-Research/Impromptu
**Stars:** ~150 | **Published:** Software and Systems Modeling (Springer) | **Status:** Active

Model-driven engineering DSL for prompt engineering, built on Langium. Gets syntax highlighting, autocomplete, and LSP support from the Langium framework. Defines multimodal prompts with versioning, chaining, and multi-language support.

**Key pattern:** Formal model-driven engineering applied to prompts. The Langium foundation is notable — a real parser, a real LSP, a real type system, not just string templates.

---

## Declarative Agent Specification Standards

A distinct category from DSLs: standards that define what an agent IS (identity, capabilities, constraints, governance) without specifying how it runs.

### Oracle Agent Spec (Open Agent Specification)

**URL:** https://github.com/oracle/agent-spec
**Stars:** ~800 | **Published:** arxiv 2510.04173 | **Status:** Active (Oracle-backed, integrating with AG-UI)

Framework-agnostic declarative language for defining agentic systems. JSON/YAML serialization. Comes with PyAgentSpec (Python SDK) and WayFlow (reference runtime). Defines standalone agents, structured workflows, and multi-agent compositions.

**Key pattern:** "ONNX for agents" — define once, run on any runtime. Separates agent definition from execution.

**Relevance:** If lx becomes a compilation target, Oracle Agent Spec is a plausible interchange format. Or lx could consume Agent Spec definitions as input.

### Agent Format (.agf.yaml) — Snap

**URL:** https://agentformat.org/ | https://eng.snap.com/agent-format
**Status:** Active (vendor-neutral Standard Committee)

Open standard for AI agent definitions: identity, interface, tools, constraints, governance, execution strategy. Validated against JSON Schema. `agf lint` for validation.

**Key pattern:** Separates agent-owner constraints from organization-level governance policies. Modeled after OpenAPI Initiative governance.

**Relevance:** The owner-vs-org constraint separation is interesting for lx's permission model — agent-level constraints vs workflow-level constraints vs deployment-level policies.

### GitAgent

**URL:** https://github.com/open-gitagent/gitagent | https://www.gitagent.sh/
**Stars:** ~500 | **Status:** Active (Lyzr, March 2026)

Git-native agent packaging: `agent.yaml` (config), `SOUL.md` (personality/instructions), `SKILL.md` (capabilities). Workflows in `workflows/` as YAML. Export to Claude Code, OpenAI, CrewAI, LangChain, Google ADK.

**Key pattern:** "Docker for agents" — the repo IS the agent. Version agent changes via PRs. Built-in compliance mapping (FINRA, SEC).

**Relevance:** The git-native packaging model validates autoresearch's git-as-state pattern. lx programs could adopt a similar repo-as-agent structure for distribution.

### Docker Agent (cagent)

**URL:** https://github.com/docker/docker-agent
**Stars:** ~5k | **Status:** Active (Docker Engineering)

Single YAML file defines agents: models, instructions, tools, delegation rules, sub-agents. Agents packaged as OCI artifacts, pushed/pulled via Docker Hub.

**Key pattern:** Container-native agent distribution. Agents are container images.

---

## Academic and Research Systems

### PayPal Declarative Agent Workflow Language

**URL:** arxiv 2512.19769
**Status:** Production at PayPal (millions of agent interactions daily)

Declarative system separating agent workflow specification from implementation. Same pipeline definition executes across Java, Python, Go. Claims 60% reduction in development time and 3x deployment velocity. Complex workflows in <50 lines of DSL vs 500+ lines of imperative code.

**Key pattern:** The only production-proven declarative agent DSL in the literature with real deployment metrics. Cross-language execution from a single definition.

**Relevance:** Validates lx's core thesis with production data. The 10:1 code reduction ratio and cross-language execution are concrete targets lx should match or exceed.

### EnCompass — MIT CSAIL + Asari AI

**URL:** arxiv 2512.03571 | NeurIPS 2025
**Stars:** ~200 | **Status:** Active

Agent programming framework separating workflow logic from inference-time search strategy. Introduces "probabilistic angelic nondeterminism" (PAN). `branchpoint()` decorator marks unreliable operations (LLM calls). The program compiles into a search space object.

**Key pattern:** Compile agent programs into searchable execution spaces. Programmers mark "locations of unreliability" and the framework explores execution paths (beam search, MCTS) without changing workflow code.

**Relevance:** Directly relevant to lx's refine loops. Instead of `refine { ... } until { ... }`, lx could support `search { ... }` blocks where the runtime explores multiple execution paths at branch points. The unreliability-marking pattern maps to lx's agent invocations — every LLM call is implicitly a branch point.

### Lambda Prompt Calculus

**URL:** arxiv 2508.12475
**Status:** Research prototype

Dependently typed calculus with probabilistic refinements for syntactic and semantic constraints on LLM prompts. 13 constraint types as refinements. Formal type theory applied to prompt construction.

**Key pattern:** The most theoretically rigorous approach to prompt programming. Guarantees optimization safety via type preservation.

**Relevance:** Theoretical foundation for lx's type system when applied to prompt inputs/outputs. If lx's type system is going to reason about what an LLM can produce, this calculus shows what the formal underpinnings look like.

### AFlow — ICLR 2025 Oral

**URL:** https://github.com/FoundationAgents/AFlow
**Stars:** ~1.5k | **Status:** Active

Automated workflow generation via Monte Carlo Tree Search. Reformulates workflow optimization as code search. Introduces reusable "operators" (Ensemble, Review & Revise). Smaller models outperform GPT-4o on specific tasks at 4.55% of inference cost.

**Key pattern:** Meta-level workflow generation. The workflow itself is the search target, not a fixed program. MCTS explores the space of possible workflows.

**Relevance:** lx programs could be the output format for AFlow-style workflow search. Write a spec in lx, let an optimizer generate the workflow implementation.

### Trace — Microsoft/Stanford (NeurIPS 2024)

**URL:** https://github.com/microsoft/Trace
**Stars:** ~1.2k | **Status:** Active

"AutoDiff for agents." Optimizes agent workflows through execution traces and LLM-based feedback. Traces execution, computes gradients through LLM feedback, optimizes across the workflow.

**Key pattern:** Differentiation through agent execution traces. Complementary to DSPy's optimization but operates at the full-workflow level.

---

## Rust-Native Agent Ecosystem

Directly relevant as lx is built in Rust.

### Rig

**URL:** https://github.com/0xPlaygrounds/rig | https://rig.rs/
**Stars:** ~6k | **Status:** Active

The primary Rust LLM framework. Unified provider interfaces, pipeline abstractions, RAG support. Pipeline API for composing AI and non-AI operations.

### rs-graph-llm

**URL:** https://github.com/a-agmon/rs-graph-llm
**Stars:** ~300 | **Status:** Active

"LangGraph for Rust." Graph-based multi-agent workflows with compile-time correctness guarantees via Rust's type system. PostgreSQL-backed persistence. Multiple execution modes (step-by-step, batch, mixed).

**Key pattern:** Compile-time workflow correctness. Rust's type system catches invalid workflow graphs at build time.

### AutoAgents (liquidos-ai)

**URL:** https://github.com/liquidos-ai/AutoAgents
**Stars:** ~200 | **Status:** Active

Rust multi-agent framework on the Ractor actor model. ReAct executors, structured outputs, WASM compilation, configurable guardrails (Block/Sanitize/Audit policies), MCP integration.

**Key pattern:** Actor model (Ractor) for agent lifecycle + WASM deployment + guardrail policies.

### AgentFlow-RS

**URL:** https://crates.io/crates/agentflow-rs
**Status:** Active (new crate)

YAML-based agent definitions compiled to Rust workflows. Multi-LLM fallback chains, MCP support, RAG integration.

**Key pattern:** Declarative YAML → Rust compilation. The closest existing Rust crate to what lx does, but YAML instead of a custom language.

---

## Code-First Frameworks with Novel Patterns

Not DSLs, but introduce patterns worth extracting:

| Framework | Stars | Key Pattern |
|-----------|-------|-------------|
| **ell** | ~7k | Prompt versioning — automatic git-like commit messages for prompt changes via static/dynamic analysis |
| **VoltAgent** | ~5k | Combinator operators (`andThen`, `andAll`, `andRace`, `andWhen`) for declarative workflow composition in TypeScript |
| **BeeAI** (IBM/LF) | ~6k | "Agents with constraints" — deterministic rules that preserve LLM reasoning while enforcing invariants |
| **Langroid** | ~3k | Actor-model agents as message transformers with recursive task delegation |
| **Griptape** | ~3.5k | Three-structure taxonomy: Agent (single), Pipeline (sequential), Workflow (parallel DAG) |
| **Atomic Agents** | ~2.5k | Radical composability — swap components by matching I/O Pydantic schemas |
| **AgentScope** (Alibaba) | ~7k | Actor-based distribution for automatic local-to-distributed deployment transition |
| **Agno** (ex-PhiData) | ~18k | The popular "anti-DSL" — pure Python control flow, no abstractions, 100+ integrations |
| **MetaGPT** | ~50k | SOPs (Standardized Operating Procedures) as the workflow abstraction — agents play organizational roles |
| **CAMEL** | ~10k | Role-playing with inception prompting — agents assume roles and collaborate via structured conversation |

---

## Workflow Engines Adding Agent Support

Traditional orchestration platforms bolting on agent capabilities:

| Engine | Stars | Agent Angle |
|--------|-------|-------------|
| **Kestra 1.0** | ~16k | YAML-native workflows with AI agent steps, AI Copilot generates YAML from natural language |
| **Windmill** | ~14k | 20+ language support — any script becomes an agent tool, sub-20ms workflow overhead |
| **Hatchet** | ~5k | Distributed task queue → agent runtime, durable persistence, fairness/concurrency patterns |
| **Inngest** | ~6k | Serverless-first durable execution, event-driven with throttling/batching/debouncing |
| **Flyte 2.0** | ~6k | ML platform → agent runtime, resource-aware scheduling on Kubernetes |

---

## Architectural Patterns Taxonomy

Across all surveyed systems, agent program specification falls into six distinct approaches:

### 1. Custom Language (compile-time guarantees)
**Examples:** lx, SLANG, MeTTa, BAML
Programs are parsed, type-checked, and compiled. Errors caught before execution.

### 2. YAML/JSON Declaration (configuration-as-code)
**Examples:** Julep, Kestra, Docker Agent, Agent Format, AgentFlow-RS
Agents defined in data formats. Simple to author, hard to extend beyond predefined step types.

### 3. Host Language Extension (decorators/macros)
**Examples:** DSPy, APPL, ell, Pydantic AI, ControlFlow
Python/TypeScript code with framework decorators. Full host language power, no compile-time workflow validation.

### 4. Specification Standard (portable definition)
**Examples:** Oracle Agent Spec, Agent Format, GitAgent
Define what an agent IS, agnostic to runtime. The "ONNX for agents" play.

### 5. Prompt Markup (structured prompts)
**Examples:** POML, PromptML, Impromptu, SynthLang
Languages for defining prompts, not workflows. Focus on prompt portability, compression, or composability.

### 6. Visual Graph (node-and-wire)
**Examples:** Dify (~70k stars), Rivet, n8n (~60k stars)
Drag-and-drop workflow builders. Highest adoption numbers but least programmable.

### Where lx Sits

lx is the only system in Category 1 that targets agent orchestration (SLANG is non-Turing-complete by design, MeTTa targets AGI self-modification, BAML targets individual LLM calls). The competitive landscape validates the need: Categories 2-6 each sacrifice something lx provides — type safety, composability, expressiveness, or programmability.

---

## Key Takeaways for lx

**The specification standards (Oracle, Agent Format, GitAgent) define what the market expects an agent declaration to look like.** lx's `agent` blocks should be expressible in these formats for interop, even if lx provides richer semantics internally.

**EnCompass's branchpoint model deserves a language primitive.** Marking LLM calls as "unreliable" and compiling workflows into searchable execution spaces is more powerful than retry loops. lx could support `explore { ... }` blocks where the runtime tries multiple paths at agent invocations.

**PayPal's production numbers validate the 10:1 thesis.** <50 lines of DSL vs 500+ lines of imperative code, 60% dev-time reduction. lx should track and demonstrate similar ratios.

**The Rust agent ecosystem is small but growing.** Rig (~6k stars) is the primary framework. rs-graph-llm and AutoAgents prove compile-time workflow correctness is achievable in Rust. lx's runtime could integrate with or learn from these crates.

**Prompt markup is a separate concern from workflow orchestration.** POML's content/presentation separation and SynthLang's token compression are orthogonal to agent coordination. lx should treat prompt formatting as a pluggable layer, not a core language feature.

**Automatic parallelism detection (APPL) beats manual annotation.** If lx can infer which agent invocations are independent, it should parallelize them automatically rather than requiring explicit `spawn` or `parallel` blocks.

## Sources

- MeTTa/Hyperon: https://github.com/trueagi-io/hyperon-experimental
- AI-DSL: https://github.com/singnet/ai-dsl
- APPL: https://github.com/appl-team/appl (ACL 2025)
- POML: https://github.com/microsoft/poml
- PromptML: https://www.promptml.org/
- SynthLang: https://github.com/ruvnet/SynthLang
- PromptLang: https://github.com/ruvnet/promptlang
- Impromptu: https://github.com/SOM-Research/Impromptu (SoSyM journal)
- Oracle Agent Spec: https://github.com/oracle/agent-spec (arxiv 2510.04173)
- Agent Format: https://agentformat.org/
- GitAgent: https://github.com/open-gitagent/gitagent
- Docker Agent: https://github.com/docker/docker-agent
- PayPal DSL: arxiv 2512.19769
- EnCompass: arxiv 2512.03571 (NeurIPS 2025)
- Lambda Prompt Calculus: arxiv 2508.12475
- AFlow: https://github.com/FoundationAgents/AFlow (ICLR 2025 Oral)
- Trace: https://github.com/microsoft/Trace (NeurIPS 2024)
- Rig: https://github.com/0xPlaygrounds/rig
- rs-graph-llm: https://github.com/a-agmon/rs-graph-llm
- AutoAgents: https://github.com/liquidos-ai/AutoAgents
- AgentFlow-RS: https://crates.io/crates/agentflow-rs
- ell: https://github.com/MadcowD/ell
- VoltAgent: https://github.com/VoltAgent/voltagent
- BeeAI: https://github.com/i-am-bee/beeai-framework
- Langroid: https://github.com/langroid/langroid
- Griptape: https://github.com/griptape-ai/griptape
- Atomic Agents: https://github.com/BrainBlend-AI/atomic-agents
- AgentScope: https://github.com/agentscope-ai/agentscope
- Agno: https://github.com/agno-agi/agno
- MetaGPT: https://github.com/FoundationAgents/MetaGPT
- CAMEL: https://github.com/camel-ai/camel
- Kestra: https://github.com/kestra-io/kestra
- Windmill: https://github.com/windmill-labs/windmill
- Hatchet: https://github.com/hatchet-dev/hatchet
- Inngest: https://github.com/inngest/inngest
- Flyte: https://github.com/flyteorg/flyte
- Dify: https://github.com/langgenius/dify
- Rivet: https://github.com/Ironclad/rivet
- n8n: https://github.com/n8n-io/n8n
