# SLANG: Deep Dive

## Identity

SLANG (Super Language for Agent Negotiation & Governance) is a minimal DSL for portable agent workflows. Created by Riccardo Tartaglia (Tech Lead, Salerno, Italy; GitHub: `riktar`). 17 GitHub stars, 0 forks, 1 contributor. MIT license, v0.7.5 (March 2026). TypeScript implementation at `github.com/riktar/slang`. npm: `@riktar/slang`.

A solo project with no known production deployments. Published via a single DEV Community blog post (March 13, 2026). No HN or Reddit discussion found.

## The Three Primitives

SLANG's entire execution model rests on three primitives:

**`stake`** -- Produce something and send it somewhere. Triggers an LLM call or tool invocation, optionally binds the result, optionally routes output to recipients.

Variants:
- Local: `let x = stake func()`
- Directed: `stake func() -> @Agent`
- Broadcast: `stake func() -> @all`
- Multi-recipient: `stake func() -> @Agent1, @Agent2`
- Flow output: `stake func() -> @out`

**`await`** -- Wait until someone sends you something. Blocks until a named source delivers a message.

Variants:
- Single: `await data <- @Source`
- Multiple: `await data <- @A, @B`
- Counted: `await results <- @Workers (count: 3)`
- Any: `await input <- @any`
- Wildcard: `await signal <- *`

**`commit`** -- I'm done, here's my result. Signals final output and convergence.

Variants:
- Unconditional: `commit`
- With value: `commit result`
- Conditional: `commit result if result.confidence > 0.8`

A fourth operation, **`escalate`**, delegates to another agent or human: `escalate @Human reason: "explanation"`.

## Why Three Primitives

The design rationale is learnability, LLM-generability, and static analyzability. "SLANG is deliberately not Turing-complete. You can't write arbitrary logic in it. That's by design." The limited surface area means the language is "easy to learn, easy to parse, and easy for LLMs to generate correctly."

The SQL analogy: just as SQL describes WHAT data you want rather than HOW to get it, SLANG describes WHAT agents should do and how they coordinate, not the imperative mechanics.

## Actor Model Foundation

"SLANG's formal foundation is rooted strictly in the Actor Model of concurrent computation combined with Finite State Machines."

1. **Message Passing** -- No shared memory or global state. All inter-agent communication via explicit `stake -> @Target` / `await <- @Source`.
2. **Mailboxes** -- `await` acts as the actor's mailbox, blocking until required inputs arrive.
3. **State Transitions** -- The workflow is "essentially a directed graph"; `output: { ... }` schemas provide typed contracts between nodes.

"It's a formalization of the routing, not the reasoning." The LLM's probabilistic generation happens inside a node; SLANG governs the deterministic structure connecting nodes.

## Non-Turing-Completeness

SLANG has `repeat until` loops but caps them at 100 iterations. No arbitrary recursion, no general-purpose computation, no `spawn()` for dynamic agent creation at runtime. Agent topology is "static, deterministic graph before execution starts."

This means:
- All flows are statically analyzable
- Deadlocks can be detected before execution
- Dependency graphs can be built and visualized
- Termination is guaranteed (modulo budget constraints)

## Syntax

### Agent Definition
```
agent Reviewer {
  role: "Staff engineer focused on security, performance, and best practices"
  model: "claude-sonnet"
  tools: [code_exec]
  retry: 2

  await code <- @Developer
  let result = stake review(code, checks: ["security", "performance"]) -> @Developer
    output: { approved: "boolean", score: "number", notes: "string" }
  commit
}
```

Agents have optional metadata (`role`, `model`, `tools`, `retry`) and a body of operations. `model` enables heterogeneous model routing within a single flow.

### Output Schemas
```
output: { approved: "boolean", score: "number", notes: "string" }
```
Injected into the LLM prompt. Runtime uses multi-stage JSON extraction to enforce. Types are strings: `"string"`, `"number"`, `"boolean"`.

### Control Flow
- Inline conditions: `commit result if result.confidence > 0.8`
- Block conditionals: `when condition { ... } else { ... }`
- Variables: `let` (declare) / `set` (mutate) for agent-local state
- Loops: `repeat until condition { ... }` (bounded at 100 iterations)

### Flow Composition
```
flow "full-report" {
  import "research" as research_flow
  import "article" as article_flow
  agent Orchestrator {
    stake run(research_flow, topic: "AI agents market 2026") -> @Compiler
    stake run(article_flow, topic: "Executive summary") -> @Compiler
  }
  agent Compiler {
    await results <- @Orchestrator (count: 2)
    stake compile(results, format: "executive briefing") -> @out
  }
  converge when: all_committed
}
```

### Convergence
```
converge when: all_committed
converge when: committed_count >= 1
converge when: @Agent1.committed && @Agent2.committed
```

### Budget Constraints
```
budget: tokens(50000), rounds(5), time(60s)
```

### Post-Convergence Side Effects
```
deliver: save_file(path: "report.md", format: "markdown")
deliver: webhook(url: "https://hooks.example.com/reports")
```

## Dual-Mode Execution

The same `.slang` file runs in two modes:

1. **Zero-setup mode:** Paste into ChatGPT/Claude/Gemini with a system prompt. The LLM simulates agents sequentially.
2. **Production mode:** CLI/API with parallel execution, real tools, checkpointing, deadlock detection, 300+ models via OpenRouter.

This is the core portability claim -- the specification IS the program, the runtime is swappable.

## Implementation

TypeScript (88.4%). Pipeline: `Source → Lexer → Parser → AST → Resolver → Graph → Runtime → Result`.

- Recursive-descent parser with error recovery
- Resolver does dependency mapping and deadlock identification
- Runtime: agent scheduling, message passing, parallel dispatch (Promise-based)
- Multi-provider LLM via OpenRouter (300+ models)
- Full LSP (diagnostics, autocompletion, go-to-definition)
- Browser playground (React 19 + Vite 6)
- 266 tests as of v0.7.2
- Structured error codes: L1xx (lexer), P2xx (parser), R3xx (resolver), E4xx (runtime)
- Tool call safety limit: 10 per operation

## Error Handling

- Agent-level `retry: N` with exponential backoff on LLM failure
- `escalate @Human reason: "..."` for delegation
- Budget exhaustion terminates with `budget_exceeded` status
- Deadlock detected statically; flow fails with `deadlock` status
- No try/catch, no compensation, no saga patterns

## What You Cannot Express in SLANG

- **No dynamic agent creation** -- topology fixed at parse time. No `spawn()`.
- **No general computation** -- anything beyond simple conditionals and bounded loops must go into tool handlers.
- **No channel abstraction** -- messages addressed agent-to-agent by name. No channel mobility (pi-calculus), no multicast beyond `@all`.
- **No hierarchical composition at runtime** -- `import` is static. Sub-flows cannot be dynamically selected based on runtime data.
- **No error recovery strategies** -- `retry` is LLM-failure retry only. No try/catch, compensation, or saga patterns.
- **Weak type system** -- output schemas are string-described types. No union types, enums, nested object schemas, or validation beyond LLM extraction.

## Comparison to Other Agent DSLs

| Dimension | SLANG | Julep | lx (target) |
|-----------|-------|-------|-------------|
| Format | Custom syntax | YAML | Custom syntax |
| Primitives | 3 (stake/await/commit) | 18 step types | Rich (agents, tools, pipes, state) |
| Turing-complete | No (by design) | Yes (via expressions) | Yes (by design) |
| State | Agent-local variables only | Session-level persistent | Multi-tier (core/archival) |
| Typing | String-described schemas | Runtime Pydantic validation | Static type system |
| Agent topology | Static (parse-time) | Static (YAML-defined) | Dynamic (spawn at runtime) |
| Maturity | Solo dev, 17 stars, v0.7 | Funded startup (shut down), ~7k stars | In development |

## Relevance to lx

**Dual-mode execution is novel.** The same program works as an LLM prompt (paste into Claude) AND as a real runtime program. lx programs could potentially serve a similar dual purpose -- the program text is readable enough that an LLM could "execute" it by understanding the specification, even without a real runtime.

**Readability-first syntax with semantic function names.** `stake gather(topic: "AI trends") -> @Analyst` reads like natural language. lx should aim for similar readability. Function names as semantic labels rather than code references.

**Static deadlock detection.** Because the agent topology is static and all communication is through explicit `await`/`stake` pairs, deadlocks are detectable before execution. lx should provide static analysis for common multi-agent coordination bugs.

**The `converge` termination model.** Declarative convergence conditions (`all_committed`, `committed_count >= N`, per-agent conditions) are more expressive than simple "all tasks done." lx should support flexible workflow termination conditions.

**Budget constraints as first-class.** `budget: tokens(50000), rounds(5), time(60s)` at the flow level is a useful pattern. lx should support resource budgets on workflows and individual agents.

**Non-Turing-completeness as a deliberate choice.** SLANG argues this enables learnability and LLM-generability. lx takes the opposite approach (Turing-complete by design) -- but the tension is worth acknowledging. lx's richness enables more powerful workflows at the cost of a larger language surface. The key question: can lx maintain readability and LLM-generability despite being Turing-complete?

**The limitations validate lx's richer design.** No dynamic agent creation, no general computation, no error recovery, weak types -- these are exactly the gaps lx fills. SLANG proves that three primitives can express simple coordination patterns, but real-world agent workflows need more: dynamic topology, proper error handling, strong types, runtime composition.

**Actor model without shared state.** SLANG's strict message-passing with no shared memory is clean but limiting. lx should support both: explicit message passing AND shared memory blocks (like Letta's shared memory) for agents that need it, with the type system enforcing safety.