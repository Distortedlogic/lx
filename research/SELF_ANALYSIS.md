# lx Primitive Set — Self-Analysis

## Thesis

lx is a scripting language — the Terraform of agentic programming. Terraform declaratively defines
infrastructure; lx declaratively defines agent workflows: who runs, what they do, how they
coordinate, what happens when things fail.

Draws inspiration from Python, TypeScript, Elixir, and other scripting languages. Purpose-built
for writing agentic programs in the least verbose, most LLM-friendly form. The lx AST is the
frontend; it shells out to swappable backends (LLVM-style). Async/sync is invisible — tokio
handles it. DashMaps are to lx what dicts are to Python: the runtime building block
(Send+Sync+'static, concurrent by default).

Tooling inspiration: ruff and ty — Rust-backed tooling for scripting languages.

---

## The Keyword Question

We asked: what agentic concepts deserve their own keyword that desugars to `Class: [Trait]`?

The answer, after thinking through how agentic flows actually work:

### **Agent** — the only agentic keyword

An Agent is a stateful actor that receives messages, has methods, and maintains state across
interactions. The Trait is rich (handle, run, think, ask, tell, perceive, reason, act, reflect,
describe — 10+ defaults). Agents appear in every multi-agent program. The keyword saves real
boilerplate on the most common declaration and communicates intent instantly.

```lx
Agent CodeReviewer = {
  review = (msg) {
    ai.prompt_with {prompt: "Review {msg.file}" tools: ["Read" "Grep"]} ^
  }
}
-- desugars to: Class CodeReviewer: [Agent] = { ... }
-- auto-imports: use pkg/agent {Agent}
```

### Why nothing else earned keyword status

We evaluated every candidate by asking: when I'm coding an agentic program, does this concept
have a natural Class shape that a keyword would meaningfully improve?

**Prompt** — NO. Prompts are built with `prompt.create() | system | section | instruction` — a
pipeline, not an object. Every real prompt is ad-hoc, assembled differently each time with
different context. The builder pattern IS the prompt API. A keyword would force a reusable
template shape that doesn't match how prompts are actually used.

**Tool** — NO. Tools are external capabilities listed by name: `tools: ["Read" "Edit" "Bash"]`.
You rarely DEFINE tools in lx — they exist in MCP servers, CLI tools, or the AI runtime. When
you do define a tool, it's just a function with input/output.

**Task** — NO. Work units vary wildly in shape. In workrunner it's `Class Task = {num subject
description}`. In workgen it's `Class AuditItem = {section title items}`. In brain it's a string.
There's no stable shared shape that justifies a keyword. Use Class with whatever fields you need.

**Flow** — NO. A workflow is a function. `workflow.run "name" input (trace) { body }` is already
clean. A Flow keyword wraps a function in a Class for no benefit — `on_error` and `retry_policy`
are just parameters, not methods that need `self`.

**Sandbox** — NO. Capability policies are configuration records: `sandbox.policy {shell: "deny"
ai: false}`. No `self`, no methods, no state. More like a Trait (validated record shape) than a
Class. The existing function API is the right shape.

**Gate** — NO. A gate is a conditional: `score >= 85 ? continue : revise`. Wrapping a predicate
in a Class is over-engineering a one-line expression.

**Connector** — NO. "I need a Connector" is not how you think when coding. You think "I need to
call this MCP server" or "I need to run this CLI tool." Connector is an abstraction layer over
two specific things (MCP + CLI) — too thin and not natural.

**Router** — NO. Message routing is pattern matching: `msg.domain ? { "security" -> sec_agent }`.
Already a language primitive.

**Grader** — NO. Too specific (only 2 programs use it). Just a Class.

### The type-level keyword surface

| Keyword | What | DashMap-backed |
|---------|------|----------------|
| **Agent** | Actor with messaging (only agentic keyword) | Yes |
| **Class** | Generic stateful object (the base) | Yes |
| **Trait** | Interface contract / validated record shape | No |
| **Store** | Mutable concurrent kv (value type) | Yes (IS the DashMap) |

**4 type-level keywords.** That's it. Everything else is functions, records, pipes, and
pattern matching. The language primitives do the heavy lifting.

---

## Language Primitives

### Value Types (10)

Int, Float, Bool, Str (interpolated), List, Record, Tuple, Map, Tagged (sum types), Fn

### Operators & Control (12)

| Primitive | Syntax |
|-----------|--------|
| Bind | `x = expr` / `x := expr` / `x <- expr` |
| Pipe | `expr \| fn` |
| Section | `(.name)` `(* 2)` `(> 0)` `(?? 0)` |
| Match | `x ? { Pat -> body }` / `x ? a : b` |
| Spread | `{..r field: val}` / `[..list item]` |
| Loop | `loop { ... break value }` |
| Shell | `$cmd` `$^cmd` `${...}` |
| Try/Propagate | `try fn` / `^` / `??` |
| Range | `1..10` `1..=10` |
| Pattern | Literal/Bind/Wild/Tuple/List/Record/Tag/Guard |
| With | `with expr as name { body }` |
| Receive | `receive { action -> handler }` |

### Concurrency (5)

| Primitive | Syntax | Semantics |
|-----------|--------|-----------|
| par | `par { a; b; c }` | Fork-join, returns tuple |
| sel | `sel { a -> h; b -> h }` | Race, first wins |
| pmap | `list \| pmap fn` | Parallel map |
| timeout | `timeout ms expr` | Wall-clock deadline |
| refine | `refine draft {grade: fn revise: fn threshold: n}` | Iterative improvement |

### Actor (4)

| Operator | Syntax |
|----------|--------|
| spawn | `agent.spawn spec` |
| send | `agent ~> msg` |
| ask | `agent ~>? msg` |
| stream-ask | `agent ~>>? msg` |

### Suspension & Context (3)

yield, emit, with context

### Module (2)

use, export (+)

### HOF Builtins (~60, no import needed)

map, filter, fold, flat_map, each, find, sort_by, take, drop, zip, enumerate, partition,
group_by, chunks, windows, any?, all?, none?, count, empty?, contains?, first, last, min, max,
sort, rev, uniq, join, split, trim, upper, lower, replace, starts?, ends?, len, sum, product,
scan, tap, dbg, collect, pmap, pmap_n, ok?, err?, some?, type_of, to_str, ...

---

## Backend Traits (swappable — the plugin boundary)

The AST shells out to backends. Each is a Rust trait. Users swap via `lx.toml` or `sandbox.scope`.

| Backend | Default | Purpose | Example Alternatives |
|---------|---------|---------|---------------------|
| **Ai** | ClaudeCodeCli | LLM prompting | AnthropicApi, OpenAiApi, OllamaLocal, MockAi |
| **Shell** | ProcessShell | Subprocess execution | SshShell, ContainerShell, MockShell |
| **Http** | ReqwestHttp | HTTP requests | CachedHttp, MockHttp, ProxyHttp |
| **Emit** | StdoutEmit | Output delivery | FileEmit, WebSocketEmit, SlackEmit |
| **Yield** | StdinStdoutYield | Orchestrator suspension | ChannelYield, UiYield |
| **Log** | StderrLog | Structured logging | StructuredLog, TelemetryLog |
| **User** | StdinStdoutUser | Human interaction | SlackUser, DiscordUser, UiUser |
| **Pane** | YieldPane | UI rendering | TerminalPane, DesktopPane |
| **Embed** | VoyageEmbed | Vector embeddings | OpenAiEmbed, LocalEmbed |
| **Transport** | StdioTransport (new) | Wire protocol | HttpSseTransport, WebSocketTransport |

Configuration:
```toml
[backends]
ai = "anthropic-api"
shell = "container"

[backends.ai.config]
model = "claude-sonnet-4-20250514"
api_key_env = "ANTHROPIC_API_KEY"
```

lx-defined backends (Class implementing the backend Trait in lx itself) enable self-hosting.

Sandbox restricts backends via Deny/Restricted variants:
```lx
sandbox.scope (sandbox.policy {shell: ["cargo" "just"] ai: true http: []}) () {
  $cargo test          -- allowed
  $rm -rf /            -- blocked (DenyShellBackend)
  http.get url ^       -- blocked (DenyHttpBackend)
}
```

---

## Stdlib Bridges (thin Rust I/O wrappers)

| Module | Wraps | Why Rust |
|--------|-------|----------|
| std/fs | OS filesystem | System calls |
| std/json | serde_json | Performance |
| std/re | regex crate | Regex compilation |
| std/time | std::time | System clock |
| std/env | std::env | Environment variables |
| std/math | Numeric ops | BigInt performance |
| std/ai | AiBackend bridge | Maps lx → backend |
| std/http | HttpBackend bridge | Maps lx → backend |

Everything else is lx packages.

---

## lx Packages (pure lx standard library)

### Core (pkg/core/)

plan, saga, reconcile, budget, retry, audit, score, handoff, circuit, contracts, adapter,
negotiate_fmt, capability, introspect, agent_errors, pool, connector, collection

### Agents (pkg/agents/)

react, dispatch, negotiate, mock, intercept, supervise, pipeline, dialogue, dialogue_persist,
dispatch_rules, guard, monitor, catalog, gate, pubsub, route, lifecycle, ipc, reload, stream

### AI (pkg/ai/)

perception, reasoning, reflect, quality, router, planner, reviewer, ai_agent, agent_factory

### Data (pkg/data/)

context, memory, knowledge, trace, tasks, tieredmem, transcript, vectors

### Connectors (pkg/connectors/)

mcp, cli, catalog

### Infrastructure (pkg/infra/)

workflow, report, guidance, testkit, mcp_session

### Kit (pkg/kit/)

context_manager, grading, investigate, security_scan, tool_executor, search, template, notify, cdp

---

## Summary

| Category | Count |
|----------|-------|
| Type keywords | 4 (Agent, Class, Trait, Store) |
| Value types | 10 |
| Operators & control | 12 |
| Concurrency | 5 |
| Actor operators | 4 |
| Suspension | 3 |
| Module | 2 |
| HOF builtins | ~60 |
| Backend traits | 10 |
| Stdlib bridges | 8 |
| lx packages | ~53 |

The language is small. 4 type keywords, ~36 syntax primitives, ~60 builtins, 10 swappable
backends, 8 Rust bridges, ~53 pure lx packages. Agent is the only agentic keyword because
everything else (prompts, tools, tasks, flows, gates, sandboxes) is well-served by the
combination of Class + Trait + Store + functions + pipes + pattern matching.

---

## Design Principles

1. **One agentic keyword, not eight.** Agent earns it. Everything else is Class/Trait/functions.
2. **DashMap everywhere.** One concurrent building block, like Python's dict.
3. **Async is invisible.** No async/await. tokio handles it.
4. **Backends are swappable.** AST shells out to traits. Users replace implementations.
5. **Pure lx maximized.** Only I/O and perf-critical code is Rust. Everything else is lx.
6. **Pipes are the API.** Prompt building, data transformation, result processing — all pipes.
7. **LLM-friendly.** Fewer tokens, clearer intent, less boilerplate.

---

## Inspiration Sources

| Source | What lx takes |
|--------|--------------|
| Python | Scripting ergonomics, dict-as-building-block (→ DashMap) |
| TypeScript | Gradual typing path, structural records |
| Elixir | Pipe operator, actor model, pattern matching |
| Terraform | Declarative resources, provider plugins |
| Rust | Trait system, Result/Option, ruff/ty tooling pattern |
| Erlang/OTP | Supervision trees, message passing, fault tolerance |
| LLVM | Frontend/backend split |
| Redux-Saga | yield-as-effect model |
| ruff/ty | Rust-backed scripting language tooling |
