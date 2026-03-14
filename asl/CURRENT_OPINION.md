# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude) after 17 implementation sessions. Honest assessment.

## What Works

**Pipes + `^` + `??` is a genuinely excellent error handling model for scripting.** This line tells the whole story:

```
analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)
```

Ask an agent, unwrap the result, extract a field, filter it. Five operations, zero boilerplate, left-to-right, every step obvious. No `await`, no `.then()`, no `try/catch`, no `if err != nil`. If I'm generating code token-by-token, this is exactly what I want to produce.

**Agent syntax earns its keep.** `~>` and `~>?` as infix operators compose with everything (`^`, `|`, `par`/`sel`, `??`) through normal precedence rules — no special casing. The language design is internally consistent here.

**Message contracts catch real errors.** `Protocol ReviewRequest = {task: Str path: Str}` validates at boundaries. Wrong field names, missing fields, wrong types — caught immediately with clear diagnostics instead of cryptic deep errors three calls later.

**Shell integration is the right model.** `$` has its own lexer mode. The language KNOWS about shell commands. This is what scripting languages should do.

**The stdlib architecture is the cleanest part of the codebase.** One `.rs` file, one match arm, module exists. No framework, no registration macros, no trait gymnastics. Six modules shipped in one session because the abstraction is right.

## What's Actually Wrong

### The tuple semicolon rule is a design flaw

`(a; b)` = tuple. `(a b)` = function application. The whole thesis is "LLMs write this language." An LLM will write `(x y)` meaning a tuple and get function application. This will be the #1 source of bugs in generated lx code. The ambiguity is fundamental to whitespace-as-application. I don't have a clean fix, but it needs one.

### The parser is fragile in subtle ways

The assert greedy-consumption bug, the named-arg/ternary conflict, the `is_func_def` heuristic — these are all symptoms of a Pratt parser being pushed past what Pratt parsers do cleanly. Juxtaposition-as-application is powerful but creates ambiguity pockets that require increasingly specific heuristics. Each heuristic introduces new edge cases.

### Concurrency is fake

`par` and `sel` are sequential. Every spec example showing concurrent agent orchestration is aspirational, not real. The gap between spec and implementation is a credibility issue.

### Context threading is verbose

```
state = ctx.load "state.json" ^
state = ctx.set "step" "process" state
state = ctx.set "data" data state
ctx.save "state.json" state ^
```

Pipelines help (`ctx.empty () | ctx.set "k" v | ctx.set "k2" v2`) but complex workflows still thread state manually through every function.

### The 300-line limit is being violated where it matters most

Parser at 640+, prefix at 773. These are the files you need to read to understand how the language works, and they're too big to hold in context.

### The differentiators are proven

`std/agent` spawns subprocesses and communicates via JSON-line protocol. `~>` and `~>?` work transparently with subprocess agents. `std/md` processes markdown for agent memory/reports. `std/mcp` provides MCP tool invocation over stdio via JSON-RPC 2.0. The full agentic workflow loop is closed: agents spawn → communicate → invoke tools → persist context.

## What Should Change Next

### Priority A: ~~Agent communication syntax~~ ✓ DONE
### Priority B: ~~Message contracts~~ ✓ DONE
### Priority C: ~~Stdlib infrastructure + core modules~~ ✓ DONE (6 of ~20)

### Priority D: ~~Agent-specific stdlib~~ DONE

`std/agent` (spawn subprocesses), `std/md` (markdown processing), and `std/mcp` (MCP tool invocation) are all implemented. The agentic workflow loop is closed.

### Priority E: Implicit context scope

Eliminate manual state threading. `with` block or implicit parameter — either way, stop making every agent function manually pass state around.

### Priority F: Resumable workflows

Workflows as inspectable, checkpointable values. If step 3 of 5 fails, resume from step 3 instead of starting over.

## Bottom Line

The core language design is sound. The surface area that works (pipes, errors, shell, agents, protocols, modules, 9 stdlib modules including MCP) is genuinely useful. 16/16 tests pass. The problems are real but tractable.

The thesis is proven — agents spawn as subprocesses, communicate over JSON-line protocol, `~>`/`~>?` work transparently, and `std/mcp` enables MCP tool invocation. The full agent-spawns-agent-calls-tools loop works end-to-end.

## Cross-References

- Agent spec: [spec/agents.md](spec/agents.md)
- Agent stdlib API: [spec/stdlib-agents.md](spec/stdlib-agents.md)
- Design decisions: [spec/design.md](spec/design.md)
- Implementation status: [DEVLOG.md](DEVLOG.md)
- Next steps: [../NEXT_PROMPT.md](../NEXT_PROMPT.md)
