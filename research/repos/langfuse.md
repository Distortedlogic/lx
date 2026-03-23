# Langfuse: Open Source LLM Observability and Engineering Platform

Langfuse is the leading open-source platform for tracing, evaluating, and managing LLM applications in production. It provides full-stack observability for complex AI systems -- capturing structured logs of every LLM call, retrieval step, and tool invocation -- while offering prompt management, dataset-driven experimentation, and automated evaluation pipelines. With 23.1k GitHub stars, 2.3k forks, and adoption by projects like Langflow (116k+ stars), Open WebUI (109k+ stars), and LobeChat (65k+ stars), Langfuse has become the de facto open-source alternative to proprietary LLM monitoring platforms. Founded by Y Combinator W23 alumni, the project ships multiple releases per week across 6,495+ commits.

## Architecture Overview

Langfuse is a **TypeScript monorepo** with a decoupled web/worker architecture designed for high-throughput trace ingestion. The system separates OLTP (PostgreSQL) and OLAP (ClickHouse) workloads, using Redis for queuing and S3 for durable event storage.

```
Client SDKs (async batch) -> Web Container (API + UI)
                                 |
                         S3 (event persistence) + Redis (queue reference)
                                 |
                          Worker Container (async processing)
                                 |
                     ClickHouse (traces/observations/scores) + PostgreSQL (projects/users/prompts)
```

**Infrastructure components:**

| Component | Technology | Role |
|-----------|-----------|------|
| Web Container | Next.js | Serves UI, REST API, ingestion endpoints |
| Worker Container | Node.js | Async event processing, evaluation execution |
| Transactional DB | PostgreSQL | Projects, users, API keys, prompt versions, RBAC |
| Analytical DB | ClickHouse | Traces, observations, scores, high-volume query workloads |
| Cache / Queue | Redis or Valkey | API key cache, prompt cache, job queue |
| Object Storage | S3 / Blob Storage | Raw event persistence, multi-modal inputs, exports |
| LLM Gateway | Optional | Required for playground, LLM-as-a-judge evaluations |

**Ingestion pipeline:** All traces arrive as batches at the Web container and are immediately written to S3. Only a queue reference is stored in Redis. The Worker asynchronously retrieves events from S3 and ingests them into ClickHouse and PostgreSQL. This design ensures the Web container stays responsive under load and provides event recovery if database writes fail.

**Caching strategy:** API keys are cached in Redis to avoid database hits on every API call. Prompts use a read-through Redis cache so hot prompts never touch the database. These patterns are critical for high-throughput production deployments.

## Monorepo Structure

| Directory | Purpose |
|-----------|---------|
| `web/` | Next.js application (UI + API), Dockerized |
| `worker/` | Async event processor, Dockerized separately |
| `packages/shared/` | Shared code: Prisma schemas, ClickHouse utilities, common types |
| `packages/config-eslint/` | Shared ESLint configuration |
| `packages/config-typescript/` | Shared TypeScript configuration |
| `ee/` | Enterprise features (SCIM, audit logging, data retention) |

The shared package contains both a `prisma/` directory (PostgreSQL schema and migrations) and a `clickhouse/` directory, reflecting the dual-database architecture. The `ee/` folders use a source-available model gated by license key, while everything else is MIT-licensed.

## Data Model

Langfuse's tracing hierarchy has three levels:

**Sessions** contain multiple **Traces**, which contain nested **Observations**. Observations come in three types:

| Observation Type | Purpose | Typical Use |
|-----------------|---------|-------------|
| **Span** | Named logical unit of work | Function execution, pipeline stages, retrieval steps |
| **Generation** | LLM interaction | Model calls with prompt/completion, token usage, cost |
| **Event** | Discrete occurrence | Logging points, state transitions, errors |

**Trace fields:** Unique ID, name, session ID, user ID, metadata (key-value pairs), tags, input/output, timestamps, public visibility flag, release version.

**Generation fields:** Model name, prompt/completion content, token counts (input/output/total), calculated USD cost, latency, time-to-first-token (for streaming), error state.

**Sessions** group traces via a shared `sessionId` (any US-ASCII string under 200 characters). This enables session replay across multi-turn conversations, with bookmarking, annotation, and public sharing capabilities.

**Scores** attach to traces or individual observations. Three scoring mechanisms exist:

- **Model-based** (LLM-as-a-judge): Automated evaluation using a rubric, input context, and the output to evaluate. Costs $0.01-$0.10 per assessment. Supports GPT-4o, Claude Sonnet, Gemini Pro as judges.
- **User feedback**: Collected via SDK/API from end users
- **Human annotation**: Manual labeling through the Langfuse UI

**User tracking:** An optional `userId` on traces enables per-user aggregation of token usage, trace volume, cost attribution, and feedback scores.

**Metadata propagation:** The `propagate_attributes()` function applies key-value metadata (values limited to 200 characters, alphanumeric keys only) to all nested observations within a context, enabling consistent request-level tagging without manual threading.

## SDK Architecture

Langfuse provides first-party SDKs for **Python** and **TypeScript/JavaScript**, plus an OpenTelemetry endpoint for any language.

**Python SDK** uses a decorator-based approach:

- `@observe()` wraps functions to automatically capture inputs, outputs, timings, and errors
- Nesting follows the function call stack automatically -- no manual context management required
- `as_type="generation"` marks LLM-specific observations
- Input/output capture can be disabled per-decorator or globally via `LANGFUSE_OBSERVE_DECORATOR_IO_CAPTURE_ENABLED`
- Interoperable with context managers and manual observations

**TypeScript SDK** builds on OpenTelemetry:

- Requires `LangfuseSpanProcessor` registered with the OpenTelemetry `NodeSDK`
- Uses `startActiveObservation()` for context-managed trace creation
- Automatic parent-child span linking through OTel context propagation, even across async boundaries
- `shouldExportSpan` filter controls which spans are exported (defaults to Langfuse + GenAI/LLM spans)
- `sdk.shutdown()` required for short-lived applications to flush pending events

**Async batching:** All SDKs send tracing data asynchronously in the background. Events are queued locally and flushed in batches, so application response time is unaffected.

**OpenAI instrumentation** wraps the OpenAI client via `observeOpenAI()`, automatically capturing latencies, time-to-first-token on streams, token usage, USD costs, and errors. Configurable with session IDs, user IDs, metadata, and managed prompt references.

## Prompt Management

Langfuse provides centralized prompt version control with two prompt types:

- **Text prompts**: Simple strings with `{{variableName}}` template syntax
- **Chat prompts**: Structured message arrays with roles and content fields

Prompts are immutable once created (type cannot be changed). Version labels (e.g., "production", "staging") control which version is fetched at runtime. The `compile()` function inserts variables at execution time. Prompt versions link to traces, enabling performance analysis segmented by prompt iteration.

Prompts are cached via a Redis read-through cache. Newly created or updated prompts may have brief visibility delays due to caching behavior.

## Evaluation System

Langfuse supports three evaluation modes that can run independently or together:

**LLM-as-a-Judge** evaluations present a judge model with evaluation criteria, input context, and the output to score. The judge produces a structured score plus reasoning. Evaluations can target individual observations (recommended, runs in seconds), full traces (legacy, runs in minutes), or experiment datasets (for reproducible benchmarking). Each evaluation creates its own trace for debugging, filterable by the `langfuse-llm-as-a-judge` environment tag.

**Datasets and Experiments** provide structured test sets with inputs and expected outputs. Scores and evaluations attach continuously to dataset items, enabling regression detection before shipping prompt changes.

**Live evaluators** monitor active production traces in real-time, catching quality degradation as it happens rather than in batch.

## Analytics and Dashboards

Langfuse tracks three metric categories across customizable dashboards:

| Category | Metrics | Dimensions |
|----------|---------|------------|
| Quality | User feedback, model-based scores, human annotations, custom scores | Trace names, models, prompt versions, user segments |
| Performance | Latency, time-to-first-token, error rates | Users, sessions, features, releases |
| Cost | Token consumption, USD cost attribution | Per-user, per-model, per-feature, per-session |

Analytics can be exported to **PostHog** and **Mixpanel** for consolidation with existing product analytics. A programmatic **Metrics API** enables custom dashboards and alerting.

## Integration Ecosystem

**Framework integrations:**

| Framework | Languages | Integration Type |
|-----------|-----------|-----------------|
| LangChain | Python, JS/TS | Native callback handler |
| LlamaIndex | Python | Native callback handler |
| Haystack | Python | Pipeline integration |
| Vercel AI SDK | JS/TS | Telemetry provider |
| LiteLLM | Python, JS/TS proxy | 100+ model proxy |
| OpenAI | Python, JS/TS | Client wrapper |

**Agent frameworks:** AutoGen, CrewAI, smolagents, Goose, Inferable

**Chat/UI platforms:** Flowise, Langflow, Dify, OpenWebUI, LobeChat

**Evaluation tools:** DSPy, Instructor, Mirascope, Promptfoo

**Any language** can send traces via the OpenTelemetry endpoint, making Langfuse language-agnostic at the protocol level.

## Access Control and Multi-Tenancy

Langfuse implements hierarchical RBAC with five roles: **Owner**, **Admin**, **Member**, **Viewer**, and **None**. Organizations contain projects, and project-level roles can override organization-level assignments. API keys are scoped to projects, not users. Users can only grant roles equal to or lower than their own.

## Deployment Options

| Method | Use Case |
|--------|----------|
| Docker Compose | Local development, 5-minute setup |
| Virtual Machine | Simple production deployments |
| Kubernetes / Helm | Production-recommended, horizontal scaling |
| Terraform (AWS, Azure, GCP) | Infrastructure-as-code cloud deployments |
| Langfuse Cloud | Managed SaaS, free tier available |

All infrastructure components must run with UTC timezone. Non-UTC timezones cause queries to return incorrect or empty results.

## Licensing and Commercial Model

| Scope | License |
|-------|---------|
| Core platform (everything outside `/ee`) | MIT |
| Enterprise features (`/ee` folders) | Source-available, license key required |

Enterprise features include SCIM user provisioning, extended audit logging, and data retention policies. All deployment modes (OSS self-host, enterprise self-host, cloud) run identical codebases and schemas -- switching between them requires no code changes.

## Strengths

- **Production-proven at scale**: Battle-tested ingestion pipeline with S3-first persistence prevents data loss during outages. ClickHouse OLAP layer handles analytical queries without degrading ingestion throughput.
- **Developer experience**: Python `@observe()` decorator provides zero-boilerplate tracing. Automatic nesting via call stack eliminates manual context threading.
- **Protocol-level openness**: OpenTelemetry endpoint means any language can integrate, not just Python and TypeScript. This is a significant differentiator against vendor-locked alternatives.
- **Comprehensive evaluation**: Combines automated LLM-as-a-judge, human annotation, and user feedback in a single platform. Dataset-driven experiments catch regressions before deployment.
- **Self-hosting parity**: MIT-licensed core with no feature gates or usage limits. Identical architecture to the managed cloud offering.

## Weaknesses

- **Infrastructure complexity**: Production self-hosting requires PostgreSQL, ClickHouse, Redis, and S3 -- four distinct stateful services to operate. This is a significant operational burden compared to SQLite-based alternatives.
- **TypeScript-only backend**: The entire server stack (web + worker) is TypeScript/Node.js. Teams with Rust, Go, or JVM backends cannot extend or modify the platform in their primary language.
- **Metadata constraints**: Propagated metadata values are limited to 200 characters with alphanumeric-only keys. This is restrictive for complex structured metadata use cases.
- **Evaluation cost opacity**: LLM-as-a-judge evaluations incur their own LLM costs ($0.01-$0.10 per assessment), which can scale unpredictably on high-volume trace streams without explicit budgeting controls.
- **Enterprise feature gating**: SCIM, audit logging, and data retention require commercial licensing. Organizations with strict compliance requirements must pay for these regardless of scale.

## Relevance to Agentic AI Patterns

**Observability for multi-step agents**: Langfuse's nested span model (Trace -> Span -> Generation/Event) maps directly to agent execution patterns where a top-level task decomposes into tool calls, retrieval steps, and LLM reasoning loops. Each step is individually observable with inputs, outputs, timing, and cost.

**Tool orchestration visibility**: When agents invoke external tools, each invocation can be captured as a separate Span with its own metadata. The hierarchical trace view reveals tool selection patterns, retry behavior, and failure cascades that are invisible in flat logging.

**Multi-agent coordination**: Sessions group traces across agent boundaries. When Agent A dispatches work to Agent B (each producing its own trace), the session ID links these into a coherent execution timeline. This is critical for debugging multi-agent systems where failures may originate in a different agent than where symptoms appear.

**Context management audit trail**: The metadata propagation system enables tracking of context window contents across agent steps. By tagging observations with context-relevant metadata, teams can identify where context truncation or loss occurs in long-running agent chains.

**Evaluation as guardrails**: Live evaluators running LLM-as-a-judge on production traces can serve as real-time quality gates for agent outputs. Combined with dataset-driven experiments, this creates a feedback loop: production failures generate test cases, experiments validate fixes, and live evaluators confirm the fix in production.

**Cost attribution for agent swarms**: Per-user, per-session, and per-trace cost tracking enables precise cost modeling for agentic systems. Teams can identify which agent patterns are cost-efficient and which generate excessive token usage through redundant reasoning or unnecessary tool calls.

**Harness pattern support**: Langfuse's scoring system supports the harness pattern where an outer evaluator monitors and corrects inner agent behavior. The outer harness can query Langfuse's API for recent trace scores, detect quality degradation, and adjust agent parameters -- creating a closed-loop optimization system.

## Comparison with Alternatives

| Feature | Langfuse | Promptfoo | LangSmith | Arize Phoenix |
|---------|----------|-----------|-----------|---------------|
| Primary focus | Production observability + evaluation | Pre-deployment testing + red teaming | Full lifecycle (LangChain-native) | ML observability (broader than LLM) |
| License | MIT (core) | MIT | Proprietary | Apache 2.0 |
| Self-hosting | Full feature parity | Full (SQLite-based) | Limited | Full |
| Language support | Any (via OTel) | TypeScript + providers | Python + JS | Python |
| Real-time monitoring | Yes (live evaluators) | No (batch-oriented) | Yes | Yes |
| Prompt management | Yes (versioned + cached) | Yes (file-based) | Yes (hub) | No |
| Infrastructure needs | PostgreSQL + ClickHouse + Redis + S3 | SQLite (single file) | N/A (SaaS) | PostgreSQL |

## Practical Takeaways

- **Start with the Python `@observe()` decorator** for the lowest-friction integration path. It captures the full call hierarchy without explicit span management.
- **Use sessions for multi-turn and multi-agent systems**. The `sessionId` is the primary mechanism for correlating work across agent boundaries.
- **Deploy ClickHouse for production**. The dual-database architecture is not optional complexity -- ClickHouse handles the analytical query patterns that would cripple PostgreSQL at scale.
- **Set up LLM-as-a-judge evaluators early**. Automated quality scoring on production traces catches regressions faster than manual review. Budget $0.01-$0.10 per evaluated trace.
- **Use the OpenTelemetry endpoint for polyglot systems**. Teams running Rust, Go, or Java services alongside Python can instrument everything through a single protocol.
- **Export analytics to existing tools** via the PostHog/Mixpanel integration rather than building custom dashboards. Langfuse's analytics are strong but not a replacement for a full product analytics stack.
- **Plan for four stateful services** when self-hosting. PostgreSQL, ClickHouse, Redis, and S3 each need backup, monitoring, and scaling strategies. Use the Helm chart for Kubernetes deployments.

## Sources

- [Langfuse GitHub Repository](https://github.com/langfuse/langfuse)
- [Langfuse Documentation - Tracing](https://langfuse.com/docs/tracing)
- [Langfuse Documentation - Self-Hosting](https://langfuse.com/docs/deployment/self-host)
- [Langfuse Documentation - Scores Overview](https://langfuse.com/docs/scores/overview)
- [Langfuse Documentation - Model-Based Evals](https://langfuse.com/docs/scores/model-based-evals)
- [Langfuse Documentation - Prompt Management](https://langfuse.com/docs/prompts/get-started)
- [Langfuse Documentation - Sessions](https://langfuse.com/docs/tracing-features/sessions)
- [Langfuse Documentation - User Tracking](https://langfuse.com/docs/tracing-features/users)
- [Langfuse Documentation - Metadata](https://langfuse.com/docs/tracing-features/metadata)
- [Langfuse Documentation - Analytics](https://langfuse.com/docs/analytics/overview)
- [Langfuse Documentation - Open Source](https://langfuse.com/docs/open-source)
- [Langfuse Documentation - RBAC](https://langfuse.com/docs/rbac)
- [Langfuse Documentation - Python Decorators](https://langfuse.com/docs/sdk/python/decorators)
- [Langfuse Documentation - TypeScript SDK](https://langfuse.com/docs/sdk/typescript/guide)
- [Langfuse Documentation - OpenAI Integration](https://langfuse.com/docs/integrations/openai/js/get-started)
- [Langfuse Blog - Introducing Langfuse 2.0](https://langfuse.com/blog/2024-04-introducing-langfuse-2.0)
