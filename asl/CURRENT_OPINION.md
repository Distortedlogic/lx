# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude) after 13 implementation sessions. Updated after implementing agent communication syntax.

## What Works

**Pipes are the right primitive.** `data | analyze | filter (.critical) | create_tickets` generates left-to-right, reads like a workflow, composes naturally. This IS genuinely better than nested function calls for LLM generation.

**`^` propagation is perfect for multi-step workflows.** Every agent operation can fail. `analyzer ~>? {task} ^ | process ^` — errors propagate cleanly without try/catch noise. Combined with `??` for defaults, error handling is concise and composable.

**`par`/`sel` map directly to agent orchestration patterns.** "Do these things concurrently" and "race these and take the first result" are exactly what multi-agent workflows need. The syntax is clean.

**Shell integration as a language primitive works.** `$` has its own lexer mode, its own semantics. The language KNOWS about shell commands. This is the right model.

**Agent communication now has its own syntax.** `~>` (send) and `~>?` (ask) are infix operators with their own AST nodes. The parser KNOWS when agent communication is happening. This was the single biggest gap — agents were library calls while shell got language-level syntax. Fixed.

## What's Still Wrong

### 1. ~~Agents are just library calls~~ ✓ FIXED

`~>` and `~>?` are now language-level infix operators with their own tokens (`TildeArrow`, `TildeArrowQ`), AST nodes (`Expr::AgentSend`, `Expr::AgentAsk`), and interpreter dispatch. Agents are records with a `handler` field. The syntax composes with `^`, `|`, `par`/`sel`, and `??`.

### 2. ~~Messages are untyped bags~~ ✓ FIXED

`Protocol` keyword validates record shapes at boundaries. `Protocol ReviewRequest = {task: Str  path: Str  depth: Int = 3}` declares a contract. `ReviewRequest {task: "review" path: "src/"}` validates at application time — missing fields and type mismatches are caught immediately with clear diagnostics. Extra fields allowed (structural subtyping). Defaults filled in. `Any` type for flexible fields.

### 3. Context threading is manual

Agents accumulate state across steps. Currently:
```
state = ctx.load "state.json" ^
state = ctx.set "step" "process" state
state = ctx.set "data" data state
ctx.save "state.json" state ^
```
Every function manually threads `state`. This is exactly the kind of boilerplate that lx's design axioms say to eliminate.

### 4. Workflows are opaque imperative code

A workflow is a series of imperative statements. The runtime can't inspect it, checkpoint it, resume it, or retry individual steps. If step 3 of 5 fails, you start over. For long-running agent workflows, this is a real limitation.

### 5. The tuple semicolon rule is an LLM footgun

`(a; b)` = tuple. `(a b)` = function application. If THE WHOLE POINT is that LLMs write this language, a silent semantic difference based on one character is exactly the kind of bug LLMs will generate constantly.

## What Should Change Next

### Priority A: ~~Agent communication as language syntax~~ ✓ DONE

Implemented in Session 13. `~>` for send, `~>?` for ask. Infix operators at concat/diamond precedence (21/22). Compose naturally with `^`, `|`, `par`/`sel`, `??`.

```
analyzer ~> {action: "log" data: results}
result = analyzer ~>? {task: "review" path: "src/"} ^
analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)
```

### Priority B: ~~Message contracts~~ ✓ DONE

Implemented in Session 14. `Protocol` keyword with runtime structural validation:

```
Protocol ReviewRequest = {task: Str  path: Str  depth: Int = 3}
reviewer ~>? ReviewRequest {task: "review" path: "src/"}
-- validates record shape, fills default depth: 3, then sends to reviewer
```

Validation is a hard contract (runtime error on mismatch, not Err). Protocols are callable values — apply to a record to validate. Extra fields allowed. `Any` type for flexible fields. Exportable with `+`.

### Priority C: Implicit context scope

Instead of manual threading:
```
with ctx.load "state.json" {
  last_run = @last_run ?? "never"
  @step = "processing"
  @data = fetch_data ()
}
```

Or a lighter approach: context as an implicit parameter that agent functions can read:
```
run = (ctx) {
  ctx.step = "processing"
  result = analyze ctx.data
  ctx.result = result
}
```

### Priority D: Resumable workflows

Workflows as inspectable, checkpointable values:
```
flow = workflow "deploy" {
  step "fetch" -> fetch_artifact version ^
  step "test"  -> run_tests it ^
  step "stage" -> deploy_staging it ^
  step "prod"  -> deploy_prod it ^
}

flow | run ?? resume_from "state.json"
```

## Assessment

The core language (pipes, pattern matching, error handling, closures, shell, agent send/ask, message contracts) is genuinely good. Priorities A and B are done — lx has language-level agent communication with structural message validation.

The practical next step: `std/` import infrastructure (prerequisite for any stdlib module), then the core agent stdlib modules (`std/json`, `std/agent`, `std/mcp`, `std/ctx`).

## Cross-References

- Agent spec with `~>` / `~>?` syntax: [spec/agents.md](spec/agents.md)
- Agent stdlib API: [spec/stdlib-agents.md](spec/stdlib-agents.md)
- Design decisions doc: [spec/design.md](spec/design.md)
- Implementation status: [DEVLOG.md](DEVLOG.md)
- Next steps: [../NEXT_PROMPT.md](../NEXT_PROMPT.md)
