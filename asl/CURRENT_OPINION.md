# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude) after 18 implementation sessions. Honest assessment.

## What Works

**Pipes + `^` + `??` is a genuinely excellent error handling model for scripting.** This line tells the whole story:

```
analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)
```

Ask an agent, unwrap the result, extract a field, filter it. Five operations, zero boilerplate, left-to-right, every step obvious. No `await`, no `.then()`, no `try/catch`, no `if err != nil`. If I'm generating code token-by-token, this is exactly what I want to produce.

**Agent syntax earns its keep.** `~>` and `~>?` as infix operators compose with everything (`^`, `|`, `par`/`sel`, `??`) through normal precedence rules — no special casing. The language design is internally consistent here.

**Message contracts catch real errors.** `Protocol ReviewRequest = {task: Str path: Str}` validates at boundaries. Wrong field names, missing fields, wrong types — caught immediately with clear diagnostics instead of cryptic deep errors three calls later.

**Shell integration is the right model.** `$` has its own lexer mode. The language KNOWS about shell commands. This is what scripting languages should do.

**The stdlib architecture is the cleanest part of the codebase.** One `.rs` file, one match arm, module exists. No framework, no registration macros, no trait gymnastics.

## What's Actually Wrong

### The language has too much surface area for its niche

lx's purpose is "agents write programs that orchestrate other agents." Several features serve "nice scripting language" but not "agentic workflow language." Each costs parser/interpreter complexity and widens the surface area an LLM must learn to generate correctly. The following should be removed:

**1. Lazy iterator protocol** — Infinite sequences (`nat`, `cycle`, custom `{next: fn}` iterators) serve functional programming, not agentic workflows. Agents deal with finite data — API responses, file contents, message batches. Eager `map`/`filter`/`take` on lists covers everything. The lazy machinery adds interpreter complexity for a use case that doesn't arise.

**2. Currying** — Automatic partial application is the single biggest source of parser bugs. The named-args + default-params + currying tension is fundamental and can't be fixed without choosing a side. Sections (`(* 2)`, `(> 5)`, `(.field)`) already cover 90% of the partial-application use case in pipelines. Explicit closures `(x) f x y` are unambiguous and easier for an LLM to generate correctly. Kill currying, keep sections.

**3. Set literals (`#{}`)** — No agentic use case. Message payloads are records. Collections are lists. Deduplication is `unique`. Sets add a collection-mode parsing path and a Value variant for nothing.

**4. `$$` raw shell (no interpolation)** — Too niche. `${echo '{not interpolated}'}` covers the rare case. Two shell escape mechanisms is one too many.

**5. Type annotations (parse-and-skip)** — Worst of both worlds. They add parser complexity, give a false sense of safety, and do nothing at runtime. Either build a real type checker or remove the syntax. Protocol contracts already provide runtime validation at message boundaries, which is where it matters.

**6. Regex literals (`r/pattern/flags`)** — Lexer needs a dedicated mode for these. `std/re` with string patterns works fine. Agents rarely need inline regex — they call tools that return structured data.

**7. Composition operator (`<>`)** — `f <> g` is just `(x) x | f | g`. Caused direction confusion in 3 separate sessions. Pipes are the primary composition mechanism and they're unambiguous. Point-free composition is a Haskell-ism that doesn't pay for itself here.

**8. Tuple semicolon rule** — `(a; b)` vs `(a b)` is the #1 predicted LLM generation bug. Fix: require commas in tuples `(a, b)`. One more token, but zero ambiguity.

### Removing these cuts ~15-20% of parser/interpreter surface area

The language becomes smaller, more predictable for LLM generation, and more focused on its actual purpose.

### The parser is fragile in subtle ways

The assert greedy-consumption bug, the named-arg/ternary conflict, the `is_func_def` heuristic — symptoms of a Pratt parser pushed past what Pratt parsers do cleanly. Removing currying eliminates the worst of these.

### Concurrency is fake

`par` and `sel` are sequential. Every spec example showing concurrent agent orchestration is aspirational, not real.

### Context threading is verbose

```
state = ctx.load "state.json" ^
state = ctx.set "step" "process" state
state = ctx.set "data" data state
ctx.save "state.json" state ^
```

Pipelines help but complex workflows still thread state manually.

### The 300-line limit is being violated where it matters most

Parser at 640+, prefix at 773. These are the files you need to read to understand how the language works.

## What to Keep (looks tangential but isn't)

- **Shell integration** (`$`, `$^`, `${...}`) — agents invoke local tools via shell, this is core
- **Pattern matching** — message routing, destructuring agent responses, essential
- **Named args** (if currying removed, the tension disappears) — `mcp.call client "tool" {x: 1}` reads better than positional
- **Slicing** — cheap, and agents do slice text/lists
- **`pmap`/`pmap_n`** — batch operations over agent pools are a real pattern
- **Sections** (`(* 2)`, `(.field)`, `(> 5)`) — covers the partial-application use case without currying's ambiguity

## Priority Order

### Done: Priorities A–D.5
Agent communication (`~>`/`~>?`), message contracts (`Protocol`), stdlib infrastructure (9 modules), agent-specific stdlib (`std/agent`, `std/md`, `std/mcp`), MCP HTTP streaming transport.

### Priority S: Surface area reduction (NEW — HIGH)
Remove the 8 features listed above. This makes the language smaller, the parser simpler, and LLM generation more reliable. Should happen BEFORE adding more features.

### Priority E: Implicit context scope
Eliminate manual state threading. `with` block or implicit parameter.

### Priority F: Resumable workflows
Workflows as inspectable, checkpointable values. If step 3 of 5 fails, resume from step 3.

## Bottom Line

The core composition model (`|` + `^` + `??` + `~>?`) is the language's reason to exist and it works. The agentic loop is proven end-to-end. But the language is carrying dead weight from its "general scripting language" origins — lazy iterators, currying, sets, regex literals, parse-and-skip types, composition operator, raw shell, tuple semicolons. Each is individually small but collectively they bloat the surface area, add parser fragility, and make LLM generation harder. Cutting them makes lx a sharper tool for its actual purpose.

## Cross-References

- Agent spec: [spec/agents.md](spec/agents.md)
- Agent stdlib API: [spec/stdlib-agents.md](spec/stdlib-agents.md)
- Design decisions: [spec/design.md](spec/design.md)
- Implementation status: [DEVLOG.md](DEVLOG.md)
- Next steps: [../NEXT_PROMPT.md](../NEXT_PROMPT.md)
