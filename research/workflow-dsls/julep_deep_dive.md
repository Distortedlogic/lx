# Julep: Deep Dive

## Identity

Julep is a declarative YAML-based workflow DSL for AI agents. The most directly comparable project to lx. ~7k GitHub stars, Apache 2.0. **Hosted backend shut down December 31, 2025.** Team pivoted to `memory.store`. Self-host only via Docker Compose (Temporal + PostgreSQL + TimescaleDB + pgVector + LiteLLM + Redis).

Tagline: "GitHub Actions-style workflows" for AI agents. Core thesis: "Workflows should declare what happens, not implement how it happens."

## The 18 Step Types

From the TypeSpec source (`src/typespec/tasks/steps.tsp`), the canonical step types organized by category:

### Common Steps

| Step | Purpose |
|------|---------|
| `prompt` | Send messages to LLM, receive response. Supports ChatML arrays, Jinja templates, tool binding, model settings. `unwrap: true` extracts `response.choices[0].message.content`. |
| `tool_call` | Execute a tool by name with arguments. `arguments: "_"` passes last step's output. |
| `evaluate` | Evaluate Python expressions via sandboxed `simpleeval`. Dict of name-to-expression mappings. |
| `wait_for_input` | Pause execution, wait for external input. Info field for context. |
| `log` | Log a Jinja template message (NOT a Python expression). |

### Key-Value Steps

| Step | Purpose |
|------|---------|
| `set` | Store computed values in workflow state. |
| `get` | Retrieve value by key. |

### Iteration Steps

| Step | Purpose |
|------|---------|
| `foreach` | Sequential iteration. Max 1,000 elements. Single step body. |
| `map_reduce` | Parallel map with optional reduce expression. `parallelism: 1-100` controls batch concurrency. `reduce: $ results + [_]` with `initial: []` accumulator. |
| `parallel` | Run up to 100 steps concurrently. **NOT IMPLEMENTED** -- use map_reduce instead. |

### Conditional Steps

| Step | Purpose |
|------|---------|
| `if_else` | Binary conditional. `then` and `else` each accept ONE step (can be nested). |
| `switch` | Multi-case. `case: "_"` as default. Each case has a `then` step. |

### Control Flow Steps

| Step | Purpose |
|------|---------|
| `sleep` | Pause for duration (max 31 days). Fields: seconds, minutes, hours, days. |
| `return` | Return value dict and end workflow. |
| `yield` | Delegate to named subworkflow with arguments. |
| `error` | Throw error with message string. |

### Doc Search Steps (not in "16" marketing)

| Step | Purpose |
|------|---------|
| `embed` | Embed text (vector operation). |
| `search` | Search agent's document store. |

## Expression System

Two expression systems coexist:

**Python Expressions ($ prefix):** Evaluated via `simpleeval` (sandboxed). `$ _.topic` is Python; `"_.topic"` is a literal string.

Context variables: `_` (previous step output or iteration item), `steps[n].input/output` (zero-indexed), `agent.name/about`, `inputs`, `outputs`, `state`.

Available libraries: `re` (via re2), `json`, `yaml`, `csv`, `math`, `statistics`, `datetime`, `time`, `string`, `base64`, `urllib.parse`, `random`.

Sandbox restrictions: no lambdas, no set comprehensions, no walrus operator. String limit ~100k chars. `**` capped at 4,000,000.

**Jinja Templates:** Used in `log` steps and `prompt` content. `{{ variable }}`, `{% for %}`, etc.

## Workflow Composition

Tasks have a `main` workflow (entrypoint) plus named subworkflows:

```yaml
name: My Task
main:
  - workflow: analyze_step
    arguments:
      data: $ _.raw_data
  - prompt: "Summarize: {{ _ }}"
    unwrap: true

analyze_step:
  - tool: analyzer
    arguments:
      input: $ steps[0].input.data
  - evaluate:
      result: $ _.analysis
```

The `yield` step delegates to subworkflows. Arbitrary additional keys on the task object become named subworkflow arrays.

## Agent Model

Agents define WHO: name, canonical_name, project, model, about, instructions (string or list), metadata, default_settings, tools, default_system_template (Jinja).

Tasks define WHAT: multi-step workflows with step sequences.

Sessions define WHERE: stateful conversation containers with context overflow strategies (`truncate`, `adaptive`, `None`).

Four tool types: function (client-side, workflow pauses), system (backend CRUD), integration (third-party: Brave, Wikipedia, Weather, Email, Spider, Cloudinary, FFmpeg, ArXiv, Browserbase, Google Sheets, MCP), api_call (direct HTTP via httpx).

## Architecture

Distributed microservices: Traefik gateway → FastAPI agents API → Temporal scheduler → Python workers → LiteLLM proxy → PostgreSQL + TimescaleDB + pgVector.

YAML parsed into Pydantic v2 models via code generation from TypeSpec definitions. Temporal orchestrates execution with `TaskExecutionWorkflow`. Workers poll Temporal for task activities.

Execution state machine: `queued → starting → running → succeeded/failed/cancelled`. Running can transition through `awaiting_input` for human-in-the-loop.

## Why YAML?

Julep's arguments:
- **Deterministic execution** -- workflows are state machines, trivially resumable after failures
- **Language-agnostic** -- Python, TypeScript, REST can drive the same YAML
- **Parallel execution** -- automatic dependency detection enables concurrency
- **Testing simplicity** -- "pure functions: given input, produce output"

The GitHub Actions comparison: just as GHA uses YAML for CI/CD pipelines, Julep uses YAML for agent workflows.

"A customer-support workflow in Python becomes a 1,000-line file where business logic is tangled with orchestration logic. In YAML, the flow is immediately visible."

## What Went Wrong

**Hosted backend shutdown (December 2025).** Team pivoted to `memory.store`. The managed-platform economics (maintaining Temporal + PostgreSQL + pgVector + LiteLLM + Redis per customer) likely didn't work.

**Timing.** Launched 2024 into a crowded market (LangChain/LangGraph, CrewAI, OpenAI tool use, Anthropic function calling). YAML DSL was a differentiator but also a barrier -- developers in 2024 preferred code-first.

**Gaps between spec and implementation.** ParallelStep defined in TypeSpec but not implemented. Try-catch mentioned in docs but not formally modeled. Erodes trust.

**Limited community.** Mostly promotional content, not organic user discussions.

## Missing Step Types (Gaps for Agent DSLs)

1. **Try/Catch/Finally** -- no formal error handling beyond Temporal retries
2. **Loop/While** -- no while-loop; foreach only iterates pre-computed collections; iterative refinement requires recursive subworkflow yields
3. **Emit/Publish** -- no event emission; agent-to-agent requires system tool calls
4. **Assert/Validate** -- no mid-workflow invariant checking
5. **Timeout** -- no per-step timeout (Temporal handles at activity level)
6. **Cancel/Abort** -- no step to cancel other running executions

## Essential Step Types for Agent Workflow DSLs

**Essential (core):** `prompt` (LLM interaction), `tool_call` (capability extension), `evaluate` (data transformation), `if_else` (conditional), `foreach`/`map_reduce` (iteration), `return` (output), `error` (signaling).

**Important (production):** `set`/`get` (workflow state), `yield` (subworkflow composition), `switch` (multi-way branching), `wait_for_input` (human-in-the-loop), `sleep` (scheduling).

**Nice-to-have:** `log` (debugging, could be a tool), `parallel` (subsumable by map_reduce), `embed`/`search` (RAG-specific, could be tools).

## Relevance to lx

**Validates the declarative workflow DSL hypothesis.** Julep proves the concept works for agent orchestration. The step-type taxonomy covers ~90% of agentic workflow needs.

**Demonstrates why YAML is the wrong substrate.** Complex workflows become unwieldy. Expression syntax hits sandbox limits. No type system, no IDE support, no compile-time validation. Single-step branches are awkward. YAML is a data format, not a language.

**The 18 step types are a feature checklist.** lx should provide equivalent primitives as native language constructs: `prompt` → agent invocation, `tool_call` → tool binding, `evaluate` → expressions, conditionals → `if`/`match`, iteration → `for`/`map`, `yield` → `spawn`/subworkflow, `wait_for_input` → `await` on channels, `sleep` → timer primitives, `return`/`error` → control flow.

**What lx does differently:** Proper control flow (while loops, pattern matching, try/catch). First-class agent-to-agent messaging (Julep's biggest gap). Static type checking. No managed-platform dependency -- lx is a language/runtime, not a service.

**The Temporal backing is right.** Durable execution for long-running agent workflows is essential. lx's runtime should provide equivalent guarantees (state persistence, automatic retries, resumability) without requiring users to manage Temporal infrastructure.

**Separation of agent from workflow is sound.** Agents define capabilities; tasks define orchestration. lx already captures this with `agent` blocks vs workflow/pipe definitions.