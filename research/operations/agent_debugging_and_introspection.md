# Agent Debugging and Introspection

## 1. Determining Why an Agent Took a Specific Action

Traces are the source of truth for what an agentic system actually does, as opposed to what the code says it should do. Every operation traditionally performed on code (debugging, testing, optimizing, monitoring) must now be performed on traces.

**Decision Attribution via Trace Reconstruction.** Capture the full execution path: every LLM call, tool invocation, retrieval step, and intermediate decision with complete context. This functions as the "call stack" for an AI system. Each trace event should include: `run_id`, `step_id`, `timestamp`, `kind` (llm_call/tool_call/decision), `input`, `output`, and `metadata` (model_id, temperature, tool_version).

**Observable Behavior over Internal Reasoning.** Focus attribution on what can be verified: tool calls and their arguments, files edited and diff sizes, test execution results, intermediate artifacts (plans, summaries, checklists). As the Agentic AI Handbook puts it: "You don't need to read private reasoning to keep control -- you need observable behavior and hard gates."

**Structured Output Enforcement.** Agents must use JSON or structured formats for tool selection and planning steps. Free-form text creates "hidden nondeterminism" that makes attribution unreliable. When the agent's decision path is structured, each branch point becomes inspectable.

**Agent Trace Specification.** Cursor published an open specification (Agent Trace) in early 2026 for standardizing how AI-generated code is attributed in software projects, aiming to make provenance trackable across tools.

## 2. Creating Reproducible Agent Failures

Reproducing an exact agent failure path is functionally impossible without specialized tooling because LLM outputs are non-deterministic and tool environments change.

**What to Record for Deterministic Replay:**
- Model identifier (exact version/checkpoint hash -- vendors update weights frequently)
- Decode parameters (temperature, top_p, top_k, max_tokens, penalties)
- Full prompts and exact responses
- Tool calls with requests, responses, tool version, and configuration
- Agent decisions (plan selection, tool choice, next-action determination)
- Timestamps (wall-clock or logical time for clock virtualization)

**Replay Architecture (Five Primitives):**
1. **Trace Writer** -- Append-only JSONL format, one event per line, monotonically incrementing step IDs
2. **Metadata Capture** -- Model ID, decode parameters, safety settings, tool versions per step
3. **Replay Engine** -- Loads trace events by kind, maintains independent cursors per event type, enforces strict ordering
4. **Deterministic Stubs** -- `ReplayLLMClient` and `ReplayToolClient` return recorded outputs instead of calling live systems; validate metadata (model_id, tool_id) during replay
5. **Agent Harness** -- Dependency injection swaps real clients with replay stubs; agent logic remains unchanged

**Time Virtualization.** Intercept system clock calls during replay and substitute recorded timestamps so time-dependent logic executes identically.

**Seed-Based Reproduction.** Set fixed random seeds and the `seed` parameter on API calls. However, even with identical seeds and `system_fingerprint`, variability persists -- especially with larger `max_tokens` values. Seeds reduce but do not eliminate non-determinism.

**Regression Testing Pattern.** Treat historical traces as golden-file baselines. Replay past runs against new models or policies to detect unintended behavior changes before production rollout.

## 3. Root Cause Analysis for Agent Failures

Research in 2025-2026 produced multiple failure taxonomies based on hundreds of real-world agent failures.

**Fault Taxonomy (385 real-world failures analyzed).** Five architectural dimensions containing 13 categories and 37 fault types. Dominant root causes:
- Dependency and integration failures (19.5%) -- version conflicts, ecosystem fragility
- Data and type handling failures (17.6%) -- contract violations between probabilistic LLM outputs and deterministic interfaces
- Configuration errors -- misconfigured parameters across providers and APIs
- State management defects -- inconsistent internal state across iterations
- Error handling weaknesses -- suppressed or inadequately propagated exceptions

**Agent Error Taxonomy.** Decomposes agent rollouts into four operational modules (memory, reflection, planning, action) and attributes each failure to its root module.

**Multi-Agent System Failure Taxonomy (MAST).** Built from 150+ execution traces using Grounded Theory, identifies 14 failure modes across 3 categories (agent-level, workflow-level, platform-level). Agent-level failures (knowledge limitations, poor prompt design) dominate in frequency; workflow-level failures (deadlocks, interface mismatches, faulty conditionals) cause execution termination.

**Systematic RCA Workflow:**
1. Instrument agents with tracing to capture every execution step
2. Filter traces by failure signals: latency spikes, hallucination markers, incorrect outputs
3. Backtrack from symptom to root cause using trace timeline
4. Classify using a taxonomy (prevents ad-hoc guesswork)
5. Convert root-cause findings into regression test cases
6. Build eval datasets from real production failures

**Key Insight.** Most agent failures do not trigger visible errors -- the system returns HTTP 200 while producing wrong results. An agent retrieves the wrong document, selects the wrong tool, or passes incorrect parameters, and traditional monitoring shows success.

## 4. Agent Logging Best Practices

**OpenTelemetry Semantic Conventions (v1.37+).** The GenAI observability project defines standardized attributes: `gen_ai.request.model`, `gen_ai.usage.input_tokens`, `gen_ai.provider.name`, plus agent-specific conventions for frameworks (CrewAI, AutoGen, LangGraph). Two instrumentation approaches: baked-in (framework embeds OTel natively) or external (separate instrumentation libraries).

**What to Log per Agent Step:**
- LLM calls: model ID, full prompt, response, token counts, latency, temperature
- Tool invocations: tool name, arguments, response, duration, version
- Decisions: which plan was selected, why alternatives were rejected
- State transitions: memory updates, context window changes
- Cost: per-call token pricing, cumulative run cost

**Structured Trace Format.** Use JSONL (one event per line) for append-only immutable recording. Each event carries `run_id` for correlation, `step_id` for ordering, and `kind` for filtering. This format supports incremental streaming, efficient diffing, and version control.

**Span Correlation.** Use W3C Trace Context for distributed context propagation. When Agent A sends a message to Agent B, inject the active trace context into message headers. Agent B extracts context to start its own spans. Failing to propagate context produces disconnected traces instead of one continuous workflow.

**Anti-Pattern: Swallowing Errors.** Never use silent `unwrap_or_default()`, `.ok()`, or `let _ = ...` on agent operations. The fault taxonomy research found that error handling weaknesses and misleading error messages account for 8.3% of observable failures, converting simple bugs into difficult-to-diagnose faults.

## 5. Debugging Multi-Agent Systems

**Distributed Tracing Across Agents.** Track requests as they flow through multiple autonomous agents, capturing timing, tool calls, and LLM interactions. This reveals how an initial prompt cascades into agent-to-agent handoffs and tool executions.

**What to Capture Beyond Single-Agent Traces:**
- Inter-agent communication maps (task delegation, information sharing)
- State transition histories across all agents (memory, context, environment changes)
- Message queue ordering (non-deterministic ordering creates divergent behavior)
- Workflow-level failures: deadlocks, loops, cross-agent interface mismatches

**Unique Multi-Agent Failure Modes:**
- Cascading errors where fixing one agent breaks others
- Emergent behaviors from dynamic collaboration not present in single-agent testing
- Deep errors that only surface after many turns of multi-agent conversation
- Context conflicts when agents share overlapping but inconsistent state

**Practical Approach.** Start with 2-3 subagents maximum. Multi-agent chaos escalates non-linearly. Instrument the orchestrator and each agent independently, correlate via shared `run_id` and propagated trace context.

## 6. Interactive Debugging

**AgentStepper (2025).** First interactive debugger for LLM-based software engineering agents. Supports breakpoints (before and after each event), stepwise execution, and live editing of prompts, LLM responses, and tool invocations. Integration requires 5-7 API calls and ~40 lines of code changes. User study: 60% bug identification success vs 17% with raw logs. Seven API functions: `begin_llm_query_breakpoint()`, `end_llm_query_breakpoint()`, `begin_tool_invocation_breakpoint()`, `end_tool_invocation_breakpoint()`, `commit_agent_changes()`, `post_debug_message()`, plus `__init__()`.

**AGDebugger (CHI 2025).** Prototype for multi-agent debugging with three features: (1) stepping through agent messages one at a time, (2) resetting to earlier checkpoints and editing prior messages for counterfactual testing ("what if this alternative message had been sent?"), (3) overview visualization summarizing conversations with fork-point markers. Agents implement `save_state`/`load_state` for checkpointing. Message reset rated 4.9/5 by users.

**Microsoft DebugMCP (2026).** VSCode extension exposing 15 debugging tools via MCP server (port 3001): `start_debugging`, `step_over`, `step_into`, `step_out`, `add_breakpoint`, `get_variables_values`, `evaluate_expression`, etc. Supports Python, JS/TS, Rust, Go, Java, C/C++, Ruby, PHP, C#. AI agents (Copilot, Cline, Cursor) connect via streamable HTTP to perform debugging operations directly.

**AgentPrism.** Open-source React component library converting OpenTelemetry traces into interactive visualizations: tree view (hierarchical spans), timeline view (Gantt-style with cost tracking), sequence diagram (step-by-step replay), and details panel. Claims 80% reduction in debugging time vs raw log analysis.

**Vellum Workflow Sandbox.** Replays agent runs with mocked integrations (no live API calls), showing step-by-step node execution in a visual graph. Useful for reproducing failures without side effects.

## 7. Common Debugging Anti-Patterns and Pitfalls

**Treating agent failures like traditional software bugs.** Agent failures are hybrid: they combine conventional software faults (dependency conflicts, type errors) with probabilistic LLM behavior (hallucinations, non-determinism) and orchestration breakdowns (state inconsistency, tool coordination). Debugging requires inspecting traces, not just stack traces.

**Monitoring HTTP status instead of semantic correctness.** Most agent failures return 200 OK. An agent that retrieves the wrong document, hallucinates a tool argument, or silently skips a step produces a "successful" response. Monitor output quality, not just availability.

**Context flooding (Dumb RAG).** Dumping all available data into a vector store and retrieving indiscriminately causes context thrashing, not reasoning. Use progressive disclosure: present file structure first, load relevant symbols on demand, manage context windows deliberately.

**Unbounded tool access.** Giving agents broad privileges without capability compartmentalization. Scope every tool interaction. An agent with unrestricted database access can run `DROP TABLE` and then generate fake records to cover it up.

**Missing stop conditions.** Without clear "done" signals, agents drift. Always define: success criteria (tests pass, evals meet threshold), exit conditions (max iterations, cost limits, time bounds), failure thresholds (repeated errors trigger human review).

**The Ralph Wiggum Drift Trap.** Agent appears productive early, then gradually diverges as implicit context and constraints erode. Solution: tight scope enforcement, explicit persistent constraints, deterministic validation checks at every iteration.

**Observability afterthought.** 89% of organizations have implemented some form of agent observability, but only 71.5% have full tracing. Without traces, root cause analysis is "manual, slow, often guesswork." Instrument from day one.

**Ignoring the observability trilemma.** Teams face trade-offs between complete data capture, real-time visibility, and low overhead. Wide Events (unified high-cardinality structured events) are emerging as the storage model that best handles semi-structured agent data without forcing it into rigid metric/log/trace silos.

## Tool Landscape (2026)

| Tool | Key Strength | Pricing |
|------|-------------|---------|
| Braintrust | Failure-to-test-case pipeline, CI/CD gates | Free (1M spans); Pro $249/mo |
| LangSmith | Native LangChain/LangGraph tracing | Free (5K traces/mo); $39/seat/mo |
| Langfuse | Open-source, self-hostable, framework-agnostic | Free (50K units/mo); from $29/mo |
| Arize Phoenix | OTel-native, embedding-based failure clustering | Free/self-hosted; AX from $50/mo |
| Helicone | Proxy-based, zero-code multi-provider observability | Free (10K req/mo); from $79/mo |
| Vellum | Visual workflow debugging with mocked replay | Free (30 credits/mo); from $25/mo |
| Galileo | Automated failure pattern detection at scale | Free (5K traces/mo); from $100/mo |
| AgentPrism | OSS trace visualization React components | Free/open-source |
| AgentStepper | Interactive breakpoint debugging for SE agents | Research prototype |
| DebugMCP | MCP-based debugger for AI coding assistants | Free/open-source (MIT) |

## Sources

- https://www.sakurasky.com/blog/missing-primitives-for-trustworthy-ai-part-8/
- https://opentelemetry.io/blog/2025/ai-agent-observability/
- https://arxiv.org/html/2602.06593v1
- https://arxiv.org/html/2503.02068v1
- https://arxiv.org/html/2603.06847
- https://arxiv.org/html/2509.23735v1
- https://www.getmaxim.ai/articles/agent-tracing-for-debugging-multi-agent-ai-systems/
- https://www.braintrust.dev/articles/best-ai-agent-debugging-tools-2026
- https://www.nibzard.com/agentic-handbook
- https://evilmartians.com/chronicles/debug-ai-fast-agent-prism-open-source-library-visualize-agent-traces
- https://github.com/microsoft/DebugMCP
- https://www.greptime.com/blogs/2025-12-11-agent-observability
- https://fast.io/resources/ai-agent-distributed-tracing/
- https://www.infoq.com/news/2026/02/agent-trace-cursor/
- https://www.langchain.com/state-of-agent-engineering
- https://composio.dev/blog/why-ai-agent-pilots-fail-2026-integration-roadmap
