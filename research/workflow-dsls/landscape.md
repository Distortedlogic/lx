# Workflow DSLs & Agent Frameworks: Landscape Survey

Research date: 2026-03-20

---

## 1. Workflow Engines & DSLs

### 1.1 Temporal

**Model**: Workflows-as-code with durable execution.

Temporal introduces two core abstractions: **Workflows** (deterministic control flow) and **Activities** (side-effectful work like API calls, file I/O). Workflows are written in standard programming languages (Go, Java, TypeScript, Python, .NET, Ruby) rather than a config DSL. The platform persists every step of execution in an **Event History** — an append-only log that records Commands and Events. If a Worker crashes, the Workflow replays from its Event History, re-executing deterministic control flow while skipping already-completed Activities by returning their memoized results.

**Key primitives**:
- **Signals**: Async write messages to running Workflows (fire-and-forget, no response)
- **Queries**: Synchronous read-only state inspection (never modifies Event History)
- **Updates**: Synchronous tracked write requests (waits for completion, adds to Event History)
- **Child Workflows**: Partitioned sub-workflows with parent-child lifecycle binding
- **Timers**: Durable sleeps maintained by the Temporal Service (survive crashes)
- **Continue-As-New**: Fresh Event History for long-running Workflows (prevents unbounded history growth)
- **Workflow Versioning / Patching**: Determinism-safe code evolution via `patched()` / `getVersion()` guards

**Determinism constraint**: Workflow code must produce identical Command sequences given identical inputs on every replay. Non-deterministic operations (time, random, I/O) must be routed through Activities or SDK-provided deterministic utilities.

**Architecture**: Temporal Server persists Event History. Workers poll for tasks, execute Workflow/Activity code, and report results. The server handles scheduling, timers, retry policies, and deduplication.

**2025-2026 updates**: Temporal Nexus (cross-namespace workflow connections) GA, Multi-Region Replication GA (99.99% SLA), Temporal Cloud on Google Cloud launched, Ruby and .NET SDK pre-release/beta.

Sources:
- [Temporal Documentation — Workflows](https://docs.temporal.io/workflows)
- [Temporal — Durable Execution](https://temporal.io/blog/what-is-durable-execution)
- [Temporal — Message Passing](https://docs.temporal.io/encyclopedia/workflow-message-passing)
- [Temporal — Event History](https://docs.temporal.io/encyclopedia/event-history)
- [Temporal — Beyond State Machines](https://temporal.io/blog/temporal-replaces-state-machines-for-distributed-applications)
- [Demystifying Determinism in Durable Execution — Jack Vanlightly](https://jack-vanlightly.com/blog/2025/11/24/demystifying-determinism-in-durable-execution)

---

### 1.2 Restate

**Model**: Durable execution via journaled side effects, single Rust binary.

Restate extends the durable execution idea with a lighter-weight, function-oriented model. Instead of separating "workflows" and "activities," Restate wraps regular functions with an SDK context. Every context call (RPC, state mutation, timer, side-effect) generates a journal entry persisted by the Restate Server. On failure, the journal replays — previously completed steps return memoized results without re-execution.

**Architecture**: Single Rust binary with stream-processing internals. Uses Bifrost (embedded replicated log) for event persistence, RocksDB for state indexes, periodic snapshots to object storage. SDKs for TypeScript, Java, Kotlin, Python, Go, Rust.

**Key difference from Temporal**: No separate "Activity" concept — regular functions become durable via SDK context. Lower ceremony, lower latency (no separate Activity Task Queue polling). Restate also acts as a concurrency guard, ensuring consistent state mutations.

Sources:
- [Restate — What is Durable Execution](https://www.restate.dev/what-is-durable-execution)
- [Restate — Building a Modern Durable Execution Engine](https://www.restate.dev/blog/building-a-modern-durable-execution-engine-from-first-principles)
- [Restate GitHub](https://github.com/restatedev/restate)

---

### 1.3 Prefect

**Model**: Python-native workflow orchestration with `@flow` / `@task` decorators.

Prefect lets you turn any Python function into a tracked workflow unit. `@flow` marks top-level workflows; `@task` marks units of work within them. Unlike Airflow, Prefect 2+ is **DAG-free** — you use native Python control flow (if/else, for/while) instead of declaring a static graph. Execution happens locally; Prefect Server/Cloud receives only logs and state updates for observability.

**Key features**:
- Automatic state tracking (Pending → Running → Completed/Failed/Cancelled)
- Retries with configurable backoff
- Result caching and persistence
- Subflows (any `@flow` can call another `@flow`)
- Scheduling, event-based automations
- Prefect 3 (2025): Latest generation with performance improvements over Prefect 2

**Design philosophy**: Minimal ceremony — add decorators to existing Python code to get production-grade orchestration (retries, logging, state tracking) without rewriting.

Sources:
- [Prefect — How It Works](https://www.prefect.io/how-it-works)
- [Prefect Documentation](https://docs.prefect.io/)
- [Prefect GitHub](https://github.com/PrefectHQ/prefect)

---

### 1.4 Apache Airflow

**Model**: Python-defined DAGs with operator-based task execution.

Airflow is the dominant open-source batch workflow orchestrator. Workflows are defined as **Directed Acyclic Graphs (DAGs)** in Python files. Each node is an **Operator** (BashOperator, PythonOperator, sensor operators, etc.). Dependencies are declared explicitly via `>>` chaining.

**Key concepts**:
- **XCom** (Cross-Communication): Tasks push/pull small data payloads for inter-task data passing
- **TaskFlow API** (Airflow 2.0+): `@task` decorator that wraps Python functions, auto-manages XCom push/pull via return values, and wires dependencies from function call graphs
- **Dynamic Task Mapping** (Airflow 2.3+): Runtime-determined task fan-out via `.map()` — enables data-driven parallelism without knowing task count at DAG definition time
- **Sensors**: Special operators that wait for external conditions (file arrival, API response, time)
- **Scheduling**: Cron-based or event-driven scheduling (improved in Airflow 3.0, April 2025)

**Limitations**: Batch-oriented legacy, XCom not designed for large data, static DAG structure (dynamic task mapping partially addresses this), scheduler can be a bottleneck.

Sources:
- [Airflow — DAGs Documentation](https://airflow.apache.org/docs/apache-airflow/stable/core-concepts/dags.html)
- [Airflow — TaskFlow API](https://airflow.apache.org/docs/apache-airflow/stable/core-concepts/taskflow.html)
- [Airflow — XCom](https://airflow.apache.org/docs/apache-airflow/stable/core-concepts/xcoms.html)
- [Airflow — Dynamic Task Mapping](https://airflow.apache.org/docs/apache-airflow/2.3.0/concepts/dynamic-task-mapping.html)

---

### 1.5 Dagger

**Model**: Containerized pipeline steps, multi-language SDKs, module system.

Dagger runs every pipeline step in an isolated container via BuildKit (Docker's build engine). Pipelines are defined in code (Go, Python, TypeScript, or CUE) rather than YAML. Each step is a function; functions are packaged into **Dagger Modules** — reusable, shareable pipeline components.

**Key features**:
- Container-level isolation per step (no host dependencies)
- Aggressive caching via BuildKit layer caching
- Module system: functions auto-extend the Dagger API when loaded
- Run locally or in any CI system identically
- CUE (Configure Unify Execute) — Google's declarative language for data validation/templating, used as Dagger's original configuration layer

**Design insight**: Dagger solves the "works on my machine" CI problem by making every step's environment explicitly containerized and reproducible.

Sources:
- [Dagger Documentation](https://docs.dagger.io/)
- [Dagger GitHub](https://github.com/dagger/dagger)
- [Dagger — Python SDK](https://dagger.io/blog/python-sdk)

---

### 1.6 AWS Step Functions

**Model**: JSON-defined state machines via Amazon States Language (ASL).

Step Functions is AWS's serverless workflow orchestrator. Workflows are defined in JSON using ASL — a declarative state machine specification.

**State types**:
| Type | Purpose |
|------|---------|
| Task | Execute work (Lambda, ECS, API call) |
| Choice | Conditional branching based on input data |
| Parallel | Execute branches concurrently |
| Map | Iterate over array items (fan-out) |
| Wait | Delay execution (fixed time or timestamp) |
| Pass | Pass input to output (transform/inject data) |
| Succeed | Terminal success state |
| Fail | Terminal failure state with error/cause |

**Error handling**: Task, Parallel, and Map states support `Retry` and `Catch` fields.
- **Retry**: Array of retriers with `ErrorEquals`, `IntervalSeconds`, `MaxAttempts`, `BackoffRate`, `MaxDelaySeconds`, `JitterStrategy`
- **Catch**: Array of catchers with `ErrorEquals`, `Next` (fallback state), `ResultPath`
- Built-in errors: `States.ALL`, `States.Timeout`, `States.TaskFailed`, `States.Permissions`, `States.Runtime`, `States.HeartbeatTimeout`, `States.DataLimitExceeded`
- Retries evaluated first; catchers applied only if retries exhausted

**QueryLanguage**: ASL supports JSONPath (legacy) and JSONata for input/output processing.

Sources:
- [Step Functions — States Language](https://docs.aws.amazon.com/step-functions/latest/dg/concepts-amazon-states-language.html)
- [Step Functions — Error Handling](https://docs.aws.amazon.com/step-functions/latest/dg/concepts-error-handling.html)
- [Step Functions — State Machine Structure](https://docs.aws.amazon.com/step-functions/latest/dg/statemachine-structure.html)

---

### 1.7 Argo Workflows

**Model**: Kubernetes-native YAML workflow specifications.

Argo Workflows runs on Kubernetes, defining workflows as custom resources in YAML. Each step runs in its own container (Pod).

**Template types**:
- **Steps**: Sequential stages, each containing parallel sub-steps
- **DAG**: Explicit dependency graph between tasks
- **Container**: Single container execution
- **Script**: Inline script execution
- **Resource**: Kubernetes resource manipulation
- **Suspend**: Pause execution (human approval, external event)

**Artifact passing**: Steps produce output artifacts (files) that downstream steps consume. Artifacts are stored in S3/GCS/Minio and referenced via template expressions like `{{tasks.step-A.outputs.artifacts.output-artifact-1}}`.

**Key features**: Parameterized workflows, conditional execution, retry policies, suspend/resume, workflow templates (reusable components), cluster-wide WorkflowTemplates.

Sources:
- [Argo Workflows — DAG](https://argo-workflows.readthedocs.io/en/latest/walk-through/dag/)
- [Argo Workflows — Artifacts](https://argo-workflows.readthedocs.io/en/latest/walk-through/artifacts/)
- [Argo Workflows GitHub](https://github.com/argoproj/argo-workflows)

---

### 1.8 Luigi

**Model**: Python Task classes with target-based dependency resolution.

Luigi (by Spotify) models workflows as Python classes. Each Task defines:
- `requires()`: Declares upstream dependencies (other Tasks)
- `output()`: Declares the Target (file, database row) this Task produces
- `run()`: Executes the actual work

**Dependency resolution**: Luigi checks if a Task's output Target already exists. If so, the Task is considered complete and skipped. If not, Luigi recursively resolves and runs dependencies first.

**Centralized scheduler**: Prevents duplicate task execution across workers. Single-threaded central scheduler grants run permissions, ensuring the same task never runs simultaneously on multiple workers.

**Design insight**: Target-based completion checking (idempotent by nature) — if the output file exists, the task is done. Simple, but limited for non-file-based workflows.

Sources:
- [Luigi Documentation — Workflows](https://luigi.readthedocs.io/en/stable/workflows.html)
- [Luigi Documentation — Execution Model](https://luigi.readthedocs.io/en/stable/execution_model.html)
- [Luigi GitHub](https://github.com/spotify/luigi)

---

### 1.9 n8n / Zapier / Make

**Model**: Visual workflow builders with trigger/action/webhook model.

These are low-code/no-code automation platforms targeting non-developer users.

| Platform | Model | Hosting | Integrations | Key trait |
|----------|-------|---------|-------------|-----------|
| **Zapier** | Linear trigger → action chain | Cloud-only | 7,000+ | Largest integration library, simplest UX |
| **Make** (fka Integromat) | Visual canvas, branching/parallel | Cloud-only | 3,000+ | Visual drag-and-drop, powerful data transformation |
| **n8n** | Visual node editor | Self-hosted or cloud | 400-1,000+ built-in | Open-source, code-capable (JS/Python in nodes), HTTP request node for any API |

**Common architecture**: Trigger node starts workflow (webhook, schedule, event) → sequence of action nodes → conditional branching → output. Each node is a self-contained integration point.

**Relevance to lx**: These represent the opposite end of the abstraction spectrum — maximum accessibility, minimum programmability. They demonstrate that the trigger/action/webhook model is highly intuitive for simple automations but breaks down for complex logic, state management, and error handling.

Sources:
- [n8n vs Zapier vs Make Comparison](https://www.digidop.com/blog/n8n-vs-make-vs-zapier)
- [n8n — How it compares to Zapier](https://n8n.io/vs/zapier/)

---

## 2. Agent Frameworks

### 2.1 LangChain / LangGraph

**LangChain** is the dominant LLM application framework. Core abstractions:
- **Chains**: Compositions of prompts, models, parsers, and retrievers wired via LCEL (LangChain Expression Language) into DAG-like pipelines
- **Agents**: LLM-driven decision loops that dynamically choose which tools to invoke (ReAct pattern)
- **Tools**: Functions with name, description, and schema that agents can call (APIs, databases, code execution)
- **Memory**: Conversation-level (short-term) and episodic (long-term) context retention
- **LCEL**: Declarative pipe-based composition (`prompt | model | parser`) implementing the Runnable interface

**LangGraph** is LangChain's graph-based agent orchestration layer:
- Agents modeled as **stateful graphs** — nodes are functions, edges are conditional transitions
- **State**: Typed state object flows through the graph, accumulated/modified at each node
- **Checkpointing**: Saves graph state at each step to PostgreSQL/Redis — enables pause/resume, crash recovery, time-travel debugging
- **Human-in-the-loop**: `interrupt()` pauses execution, persists state, waits for human input (approve/reject/modify), then resumes
- Supports branching, looping, parallel execution, and sub-graphs
- "Durable execution" mode for long-running agent workflows

Sources:
- [LangGraph GitHub](https://github.com/langchain-ai/langgraph)
- [LangChain — Open Source Framework](https://www.langchain.com/langchain)
- [LangGraph — Human-in-the-Loop](https://docs.langchain.com/oss/python/langchain/human-in-the-loop)

---

### 2.2 CrewAI

**Model**: Role-based multi-agent orchestration with organizational metaphors.

CrewAI assigns distinct **roles** to agents (Manager, Worker, Researcher), each with specialized instructions, tools, and decision-making authority. Agents form a **Crew** that collaborates on **Tasks**.

**Orchestration modes**:
- **Sequential**: Agents execute tasks one-after-another in defined order
- **Hierarchical**: Manager agent dynamically assigns/delegates tasks to workers, tracks outcomes, can override junior decisions
- Supports parallel and conditional execution

**Key concepts**:
- Agents have role, backstory, goal, and available tools
- Tasks have description, expected output, assigned agent, and required tools
- Context sharing between tasks enables information flow
- Agent delegation: agents can assign sub-tasks to other agents

Sources:
- [CrewAI Documentation — Tasks](https://docs.crewai.com/en/concepts/tasks)
- [CrewAI GitHub](https://github.com/crewAIInc/crewAI)
- [CrewAI Framework 2025 Review](https://latenode.com/blog/ai-frameworks-technical-infrastructure/crewai-framework/crewai-framework-2025-complete-review-of-the-open-source-multi-agent-ai-platform)

---

### 2.3 AutoGen (Microsoft)

**Model**: Multi-agent conversation framework with actor-based architecture.

AutoGen enables LLM applications via multi-agent conversations. Agents are customizable, conversable entities that combine LLMs, human inputs, and tools.

**v0.4 architecture (Jan 2025)** — three layers:
1. **Core**: Actor model foundation — async message exchange, event-driven agents, scalable runtime
2. **AgentChat**: High-level task-driven API — AssistantAgent, UserProxy, RoundRobinGroupChat, SelectorGroupChat
3. **Extensions**: First- and third-party capability plugins

**Key patterns**:
- **Group Chat**: Multiple agents converse via GroupChatManager that selects next speaker
- **Nested Chats**: Inner agents within outer agents provide specialized sub-conversations
- **Code Execution**: AssistantAgent generates code; UserProxy executes it and returns results
- Message delivery decoupled from message handling (modularity)

Sources:
- [AutoGen v0.4 — Microsoft Research](https://www.microsoft.com/en-us/research/blog/autogen-v0-4-reimagining-the-foundation-of-agentic-ai-for-scale-extensibility-and-robustness/)
- [AutoGen — Multi-Agent Conversation](https://microsoft.github.io/autogen/0.2/docs/Use-Cases/agent_chat/)
- [AutoGen GitHub](https://github.com/microsoft/autogen)

---

### 2.4 Semantic Kernel (Microsoft)

**Model**: AI orchestration SDK with plugins and function calling.

Semantic Kernel is Microsoft's SDK for building AI agent systems, functioning as a dependency injection container for AI services and plugins.

**Key abstractions**:
- **Plugins**: Collections of functions exposed to AI (wrapping existing APIs)
- **Kernel Functions**: Functions with semantic descriptions (name, input/output types, side effects) enabling AI to understand and invoke them
- **Planners** (deprecated): Originally used prompts to select function sequences; replaced by native **function calling** (model chooses which functions to invoke)
- **Memory**: Vector database connectors (Qdrant, Pinecone, Weaviate) for semantic memory

**Design evolution**: Moved from prompt-based planning (Handlebars, Stepwise planners) to model-native function calling — more reliable, leveraging models' built-in tool-use capabilities.

Sources:
- [Semantic Kernel — Plugins](https://learn.microsoft.com/en-us/semantic-kernel/concepts/plugins/)
- [Semantic Kernel — Planning](https://learn.microsoft.com/en-us/semantic-kernel/concepts/planning)
- [Semantic Kernel Documentation](https://learn.microsoft.com/en-us/semantic-kernel/)

---

### 2.5 Swarm (OpenAI) → Agents SDK

**Model**: Lightweight multi-agent coordination via routines and handoffs.

Swarm (now superseded by the OpenAI Agents SDK) introduced two primitives:
- **Routines**: Natural-language instruction sets (system prompts) with associated tools
- **Handoffs**: Functions that transfer the active conversation from one agent to another

**Architecture**: Stateless — no persistent state between calls. Each agent has instructions, a role, and available functions. When a function returns an Agent object, the framework performs a handoff (like a phone transfer). Minimal ceremony, highly controllable, easily testable.

**Design insight**: Swarm proved that multi-agent coordination can be reduced to two primitives (routines + handoffs) without complex orchestration infrastructure. The OpenAI Agents SDK evolved this into a production-ready framework.

Sources:
- [Swarm GitHub](https://github.com/openai/swarm)
- [OpenAI Cookbook — Orchestrating Agents](https://developers.openai.com/cookbook/examples/orchestrating_agents)

---

### 2.6 DSPy (Stanford NLP)

**Model**: Programming (not prompting) LLMs via signatures, modules, and optimizers.

DSPy replaces brittle prompt engineering with composable Python modules that describe desired behavior declaratively. The framework then optimizes prompts, examples, and model weights automatically.

**Three core abstractions**:

1. **Signatures**: Declarative input/output specifications (e.g., `"question -> answer: float"`). Define *what* a module should do, not *how*.

2. **Modules**: Building blocks that implement strategies for invoking LLMs:
   - `Predict` — basic prediction
   - `ChainOfThought` — reasoning before answering
   - `ProgramOfThought` — code-based reasoning
   - `ReAct` — reasoning + tool use
   - `MultiChainComparison` — compare multiple reasoning chains
   - `Parallel` — concurrent execution
   - `Refine` — iterative improvement
   - `BestOfN` — select best of multiple generations

3. **Optimizers**: Automatically improve module behavior:
   - Few-shot: `BootstrapFewShot`, `LabeledFewShot`, `KNNFewShot`
   - Prompt optimization: `MIPROv2`, `GEPA`, `COPRO`
   - Weight optimization: `BootstrapFinetune`
   - Ensemble: `Ensemble`, `BetterTogether`

**Compilation pipeline**: Bootstrap (run program, collect traces) → Propose (generate candidate instructions from traces) → Search (evaluate combinations, guided by surrogate model). Optimizers compose — output of one feeds into another.

Sources:
- [DSPy Documentation](https://dspy.ai/)
- [DSPy GitHub](https://github.com/stanfordnlp/dspy)
- [Stanford HAI — DSPy](https://hai.stanford.edu/research/dspy-compiling-declarative-language-model-calls-into-state-of-the-art-pipelines)

---

### 2.7 Mastra

**Model**: TypeScript-native AI agent framework with workflows, tools, and memory.

From the team behind Gatsby. Mastra provides AI primitives in TypeScript:

- **Agents**: Autonomous entities with instructions, tools, conversation history, and memory
- **Workflows**: Graph-based state machines with discrete steps, each having inputs, outputs, and execution logic. Support branching, parallel execution, conditional logic, and error recovery.
- **Tools**: Schema-described functions that agents can invoke for external system interaction
- **Memory**: Pluggable storage layer (PostgreSQL, LibSQL, MongoDB) for execution state, conversation history, and session data. Supports pause/resume via persisted state.
- **Evals**: Built-in evaluation framework for measuring agent quality

**Design insight**: Mastra is the TypeScript counterpart to Python-centric frameworks like LangChain — same primitives (agents, tools, memory, workflows) in a TypeScript-native package.

Sources:
- [Mastra Documentation](https://mastra.ai/docs)
- [Mastra GitHub](https://github.com/mastra-ai/mastra)
- [Mastra on Y Combinator](https://www.ycombinator.com/companies/mastra)

---

## 3. Positioning Map

```
                    Config/Declarative ──────────────────── Code/Imperative
                           │                                      │
  High abstraction    Step Functions     Airflow (DAGs)      Temporal
  (orchestrator)      Argo Workflows     Luigi               Restate
                      n8n/Zapier/Make                         Prefect
                           │                                      │
                           │                                      │
  Agent-oriented      CrewAI (YAML       LangGraph            DSPy
  (LLM-native)       configs)           AutoGen              Mastra
                      Swarm/Agents SDK   Semantic Kernel      lx
                           │                                      │
                    Less expressive ──────────────────── More expressive
```

lx occupies the bottom-right quadrant: code-first, imperative, agent-oriented, maximally expressive. Its closest architectural relatives are Temporal (durable execution, workflows-as-code) and LangGraph (stateful graph execution for agents), but lx is a purpose-built language rather than a library in an existing language.
