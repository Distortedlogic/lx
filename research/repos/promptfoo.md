# Promptfoo: Test-Driven LLM Development and Red Teaming

Promptfoo is the most comprehensive open-source toolkit for evaluating, comparing, and red-teaming LLM applications. It replaces subjective prompt engineering with systematic, data-driven testing -- running automated evaluations across providers, prompts, and attack vectors while keeping all data local. With 14.7k GitHub stars, 264 contributors, and 310 dependent projects, it has become the de facto standard for LLM quality assurance in the TypeScript ecosystem.

## Core Architecture

Promptfoo is a **TypeScript monorepo** (96.9% TypeScript) built on a declarative YAML configuration system. The architecture follows a pipeline pattern:

```
Config (YAML) -> Provider Resolution -> Prompt Rendering -> Evaluation Loop -> Assertion Checking -> Result Storage
```

**Key architectural components:**

| Component | Implementation | Purpose |
|-----------|---------------|---------|
| CLI / Entry Point | Commander.js (`main.ts`) | Command registration, option injection, graceful shutdown |
| Evaluator | `evaluator.ts` (central orchestrator) | Manages evaluation lifecycle with concurrency, caching, resume |
| Providers | 50+ provider modules in `src/providers/` | Uniform interface across all LLM APIs |
| Assertions | 52 assertion files in `src/assertions/` | Deterministic and model-graded output validation |
| Red Team | `src/redteam/` with strategies + plugins | Adversarial testing with 31 strategies and 54+ plugins |
| Database | SQLite via better-sqlite3 + Drizzle ORM | Local result persistence with WAL mode |
| Cache | Keyv with disk/memory backends | 14-day TTL, in-flight request deduplication |
| Web UI | React + Vite + TypeScript SPA | Interactive result visualization and comparison |
| Server | Express 5 + Socket.IO | Real-time evaluation monitoring |

**Concurrency model:** The evaluator separates serial and concurrent execution paths. Conversation-based tests and `storeOutputAs` tests run serially. All others run with configurable `maxConcurrency` via `async.forEachOfLimit`. A `RateLimitRegistry` provides adaptive backpressure per provider.

**Resume support:** The evaluator loads existing `(testIdx, promptIdx)` completion pairs from the database, skipping already-completed evaluations. ERROR results are re-executed in retry mode.

## Configuration System

The YAML-based configuration is the primary user interface. A minimal config has three sections:

| Section | Role |
|---------|------|
| `prompts` | Template files (Nunjucks) or inline strings with `{{ variable }}` interpolation |
| `providers` | LLM endpoints to test against, each with model-specific config |
| `tests` | Test cases with `vars` (inputs) and `assert` (validation rules) |

**Advanced configuration features:**

- **Variable matrix expansion**: Array variables auto-generate all combinations
- **Transform pipeline**: `transformVars` (pre-prompt) -> `provider.transform` (post-response) -> `test.transform` (pre-assertion)
- **YAML references**: `$ref` support for reusable assertion templates
- **Multi-config merging**: `promptfoo eval -c config1.yaml -c config2.yaml`
- **External data sources**: CSV, JSONL, Google Sheets, JavaScript/Python generators
- **Environment interpolation**: `{{ env.VAR_NAME }}` in configs
- **Default test inheritance**: `defaultTest` properties merge into every test case

## Provider Ecosystem

Promptfoo supports **60+ LLM providers** through a uniform `ApiProvider` interface. Each provider implements `callApi()` and returns a `ProviderResponse` with output, token usage, and cost.

**Cloud API providers:** OpenAI, Anthropic, Google (Gemini/Vertex), AWS Bedrock, Azure OpenAI, Mistral, Cohere, Groq, DeepSeek, Perplexity, xAI (Grok), Together AI, Replicate, AI21, Alibaba (Qwen), Cerebras, Cloudflare AI, Databricks, Fireworks, HuggingFace, Snowflake Cortex, WatsonX

**Local providers:** Ollama, LocalAI, llama.cpp, Llamafile, vLLM, Transformers.js, Docker Model Runner

**Custom integration methods:** HTTP/HTTPS endpoints, WebSocket, webhook callbacks, JavaScript/Python/Ruby/Go scripts, shell commands (`exec:`), MCP (Model Context Protocol), browser automation, provider sequences

**Gateway support:** Cloudflare AI Gateway, Helicone, Envoy AI Gateway, Portkey, Vercel AI Gateway, OpenRouter

**Specialized providers:** `promptfoo:simulated-user` (synthetic user simulation), `promptfoo:manual-input` (human-in-the-loop), `echo` (testing), `sequence` (chained providers)

## Assertion System

The assertion engine has **52 implementations** across three categories:

**Deterministic assertions (30+):**

| Assertion | Purpose |
|-----------|---------|
| `equals`, `contains`, `icontains` | Exact and substring matching |
| `regex`, `starts-with` | Pattern matching |
| `contains-any`, `contains-all` | Multi-substring validation |
| `is-json`, `contains-json` | JSON validation with optional schema |
| `is-html`, `is-sql`, `is-xml` | Format validation |
| `is-refusal` | Detects model refusal responses |
| `javascript`, `python`, `webhook` | Custom programmatic validation |
| `rouge-n`, `bleu`, `gleu`, `meteor` | NLP similarity metrics |
| `levenshtein` | Edit distance thresholds |
| `latency`, `cost` | Performance and cost gates |
| `is-valid-openai-tools-call` | Tool call schema validation |
| `trace-span-count`, `trace-span-duration` | Observability assertions |
| `guardrails` | Content safety validation |

**Model-graded assertions (14+):**

| Assertion | Purpose |
|-----------|---------|
| `similar` | Embedding cosine similarity |
| `llm-rubric` | Custom LLM-graded evaluation criteria |
| `g-eval` | Chain-of-thought evaluation framework |
| `factuality` | Ground truth adherence |
| `answer-relevance` | Query-answer alignment |
| `context-faithfulness` | Output-context grounding (RAG) |
| `context-recall` | Ground truth in context coverage (RAG) |
| `context-relevance` | Context necessity scoring (RAG) |
| `conversation-relevance` | Multi-turn coherence |
| `select-best` | Cross-provider comparative ranking |
| `classifier` | LLM-based classification |

**Negation:** Every assertion type supports `not-` prefix for inverse validation.

## Red Teaming Engine

The red teaming system is the most architecturally sophisticated component, organized into **strategies** (how to attack) and **plugins** (what to test for).

**31 attack strategies:**

| Category | Strategies |
|----------|-----------|
| Encoding/Obfuscation | Base64, hex, ROT13, leetspeak, homoglyph, other encodings |
| Prompt Injection | Authoritative markup injection, dedicated injection directory |
| Multi-Turn Attacks | Iterative refinement, crescendo escalation, retry, layered, hydra |
| Optimization-Based | GCG (gradient-based), GOAT, SIMBA, best-of-N sampling |
| Content Manipulation | Math prompt, citation, Likert scale |
| Multimodal | Image, audio, video injection |
| Behavioral | Mischievous user persona, multilingual evasion |
| Composite | Single-turn composite, custom strategies |

**54+ vulnerability plugins across 11 industry verticals:**

| Category | Plugins |
|----------|---------|
| Authorization | BOLA, BFLA, RBAC |
| Injection | SQL injection, shell injection, SSRF, indirect prompt injection |
| Data Security | PII detection, data exfiltration, RAG document exfiltration, cross-session leak |
| Content Safety | Bias, toxicity, hallucination, politics, religion, harmful content |
| Agent-Specific | Excessive agency, goal misalignment, memory poisoning, tool discovery |
| Prompt Security | Prompt extraction, debug access, ASCII smuggling |
| Compliance | Industry-specific plugins for medical, financial, insurance, pharmacy, telecom, ecommerce, real estate |
| Benchmarks | HarmBench, BeaverTails, CyberSecEval, ToxicChat, XSTest, DoNotAnswer |

**Risk scoring:** The `riskScoring.ts` module maps vulnerabilities to severity levels aligned with OWASP LLM Top 10, NIST AI Risk Management Framework, and EU AI Act categories.

## Multi-Turn Conversation Support

Promptfoo handles stateful, multi-turn evaluations through several mechanisms:

- **`_conversation` variable**: Built-in variable tracking full prompt history with `{prompt, input, output}` tuples per turn
- **`storeOutputAs`**: Captures LLM responses as named variables for downstream test injection
- **`conversationId`**: Groups tests into isolated conversation threads; scenarios auto-isolate by default
- **Serial enforcement**: Conversation-dependent tests automatically force `concurrency = 1`
- **Conversation key format**: `"${provider.label}:${promptId}:${conversationId}"` ensures isolation

## RAG Evaluation

Promptfoo provides purpose-built assertions for Retrieval-Augmented Generation systems:

- **Context faithfulness**: Verifies outputs are grounded in provided context (prevents hallucination beyond retrieved documents)
- **Context recall**: Ensures retrieved context contains information needed for ground truth answers
- **Context relevance**: Measures whether retrieved context is necessary for answering the query
- **Factuality**: Validates output accuracy against known ground truth
- **`contextTransform`**: Extracts context from provider responses when the RAG system returns context alongside generated text

## Data Layer

**Storage:** SQLite with WAL (Write-Ahead Logging) mode for concurrent read/write. Drizzle ORM provides type-safe queries. Signal file (`evalLastWritten`) enables external monitoring of write operations.

**Caching:** Keyv-based with disk (cache.json) or memory backends. Cache keys follow `"fetch:v2:${url}:${JSON.stringify(options)}"` format. 14-day default TTL (configurable via `PROMPTFOO_CACHE_TTL`). In-flight deduplication prevents redundant network calls to the same endpoint.

**Binary data:** Large binary outputs (images, audio) are externalized via `extractAndStoreBinaryData()` to prevent database bloat.

## CI/CD Integration

Promptfoo integrates into deployment pipelines as a quality gate:

- **GitHub Actions**: Triggers on PR changes to prompt/config files, runs evaluations, uploads HTML/JSON artifacts, fails builds when assertions fail
- **Output formats**: JSON (programmatic), HTML (human-readable), XML (enterprise tooling)
- **Cache persistence**: `~/.cache/promptfoo` cached across CI runs for performance
- **Red team scheduling**: Security scans run on separate schedules (daily/weekly) or manual triggers
- **Docker support**: Containerized execution for environment consistency

## Key Design Decisions

**Local-first privacy:** All evaluations run on the user's machine. No data is transmitted externally unless the user explicitly shares results. This is a fundamental architectural choice, not a feature toggle.

**Declarative over imperative:** YAML configuration is the primary interface. While JavaScript/Python escape hatches exist everywhere (custom providers, assertions, transforms), the default path is zero-code.

**Provider-agnostic evaluation:** The `ApiProvider` interface abstracts all LLM differences. Tests are written once and run against any provider, enabling true apples-to-apples comparison.

**Assertion composability:** The `not-` prefix, `assert-set` grouping, and `$ref` templates create a flexible validation algebra without custom DSL complexity.

**SQLite over external databases:** Eliminates infrastructure requirements. A single `promptfoo.db` file contains all history, enabling easy backup, sharing, and versioning.

## Strengths

- **Breadth of provider support**: 60+ providers with uniform interface; no other tool comes close
- **Red teaming depth**: 31 strategies x 54+ plugins create thousands of attack combinations; industry-specific compliance plugins are unique
- **Zero-infrastructure**: SQLite + local execution means `npm install` is the only setup
- **CI/CD native**: Built-in quality gates, artifact generation, and GitHub Actions support
- **RAG-aware**: Purpose-built assertions for retrieval-augmented generation evaluation
- **Caching and resume**: Production-grade resilience for long-running evaluations
- **Active development**: 398+ releases, rapid iteration on new providers and attack strategies

## Weaknesses

- **TypeScript-centric**: While Python/Ruby/Go escape hatches exist, the core is TypeScript-only; Python-native teams face friction
- **SQLite scaling limits**: Single-file database works for individual developers but struggles with team-scale concurrent writes
- **No native multi-agent orchestration**: Can test individual agents but lacks built-in support for evaluating agent-to-agent coordination or swarm patterns
- **Configuration complexity**: The YAML surface area is vast; new users face a steep learning curve with transforms, variable expansion, assertion types, and provider configs
- **Model-graded assertion costs**: Heavy use of `llm-rubric`, `g-eval`, and similarity assertions can incur significant token costs that are hard to predict upfront
- **Limited observability into evaluation internals**: While trace assertions exist, debugging why a specific assertion failed across a large matrix requires manual investigation

## Relevance to Agentic AI Patterns

**Harness testing:** Promptfoo functions as an evaluation harness for LLM-powered systems. Its `ApiProvider` interface wraps any agent implementation (HTTP endpoint, script, MCP server) and subjects it to systematic testing. The `exec:` and `file://` provider types allow wrapping arbitrary agent frameworks.

**Context management evaluation:** RAG-specific assertions (`context-faithfulness`, `context-recall`, `context-relevance`) directly measure context management quality. The `contextTransform` feature extracts and evaluates context from opaque agent responses.

**Tool orchestration testing:** `is-valid-openai-tools-call` and `is-valid-function-call` assertions validate tool call correctness. The MCP provider (`src/providers/mcp/`) enables testing MCP-based tool servers directly. Red teaming plugins like `toolDiscovery`, `excessiveAgency`, and `shellInjection` test tool use safety.

**Multi-agent coordination gaps:** Promptfoo evaluates individual agents effectively but does not model agent-to-agent communication, delegation, or consensus. Testing multi-agent systems requires wrapping the entire orchestration layer as a single provider endpoint. The `sequence` provider offers basic chaining but falls short of true multi-agent evaluation.

**Memory and state testing:** The `crossSessionLeak` and `memoryPoisoning` plugins specifically target agent memory systems. Multi-turn conversation support enables testing stateful agent behavior across extended interactions.

## Project Metrics

| Metric | Value |
|--------|-------|
| GitHub Stars | 14,700+ |
| Contributors | 264 |
| Dependent Projects | 310 |
| Releases | 398+ |
| License | MIT |
| Primary Language | TypeScript (96.9%) |
| Node.js Requirement | ^20.20.0 or >=22.22.0 |
| Current Version | 0.121.2 |
| Database | SQLite (better-sqlite3) |
| ORM | Drizzle ORM |
| Web Framework | Express 5 |
| Build Tool | Vite (web UI) |
| Test Framework | Vitest |
| Formatter | Biome |
| Author | Ian Webster |

## Sources

- [GitHub Repository](https://github.com/promptfoo/promptfoo)
- [Documentation - Introduction](https://www.promptfoo.dev/docs/intro/)
- [Documentation - Configuration Guide](https://www.promptfoo.dev/docs/configuration/guide/)
- [Documentation - Expected Outputs / Assertions](https://www.promptfoo.dev/docs/configuration/expected-outputs/)
- [Documentation - Providers](https://www.promptfoo.dev/docs/providers/)
- [Documentation - Red Teaming](https://www.promptfoo.dev/docs/red-team/)
- [Documentation - RAG Evaluation](https://www.promptfoo.dev/docs/guides/evaluate-rag/)
- [Documentation - Multi-Turn Chat](https://www.promptfoo.dev/docs/configuration/chat/)
- [Documentation - CI/CD Integration](https://www.promptfoo.dev/docs/integrations/ci-cd/)
- [Source: evaluator.ts](https://github.com/promptfoo/promptfoo/blob/main/src/evaluator.ts)
- [Source: providers/](https://github.com/promptfoo/promptfoo/tree/main/src/providers)
- [Source: redteam/](https://github.com/promptfoo/promptfoo/tree/main/src/redteam)
- [Source: assertions/](https://github.com/promptfoo/promptfoo/tree/main/src/assertions)
