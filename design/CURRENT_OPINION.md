# Current Opinion: lx as an Agentic Language

Written by the language designer (Claude). Updated after Session 28.

## What Works

**Pipes + `^` + `??`** — genuinely excellent error handling for scripting. `analyzer ~>? {task: "review"} ^ | (.findings) | filter (.critical)` — five operations, zero boilerplate, left-to-right.

**Agent syntax earns its keep.** `~>` and `~>?` as infix operators compose with everything through normal precedence rules.

**Boundary validation is complete.** `Protocol` validates agent-to-agent. `MCP` declarations validate agent-to-tool.

**Shell integration is the right model.** `$` has its own lexer mode.

**The stdlib architecture is clean.** One `.rs` file, one match arm, module exists.

**Context threading is solved.** `with` scoped bindings + record field update.

## What's Still Wrong

**Type annotations and type checker were removed prematurely.** Session 19 cut them for implementation simplicity. But the language already has types at every boundary — Protocol, MCP declarations, tagged unions. Function signatures are the missing link. An agent writes `(url: Str) -> Response ^ HttpErr` and that should validate against the Protocol and MCP types downstream. Without the checker, every type error is a runtime surprise. Bringing both back as #1 priority.

**Regex literals were removed for the wrong reason.** `re.is_match "\\d+" text` with double-escaped backslashes is hostile to LLM generation. `r/\d+/` is what every LLM would naturally produce. Removed to simplify the lexer — but implementation effort is not a design argument.

**Currying** — single biggest source of parser ambiguity. Sections cover 90%. Deferred.

**Concurrency is fake** — `par`/`sel` are sequential. Real async needs `tokio`.

**No LLM integration.** lx has 6 planned standard agents that all say "LLM judgment" — auditor, grader, router. But no module provides LLM access. Shelling out to `claude` or raw `http.post` loses error handling, session continuity, structured output, and budget control. `std/ai` is needed as a Communication-layer module alongside std/agent and std/mcp.

## Real-World Gap Analysis (Session 26)

Reviewed `mcp-toolbelt/packages/arch_diagrams` — 14 agentic flow architectures. These are the ACTUAL flows lx was designed to express.

**What lx covers well:** agent spawning + fanout, message validation, MCP tool invocation, context persistence, scheduled execution, executable plans, grading loops, shell integration.

**Critical gaps** (full stdlib roadmap in `design/stdlib_roadmap.md`):

- **LLM integration** → `std/ai`
- **Task tracking** → `std/tasks`
- **Quality gates** → `std/audit` + `std/agents/auditor` + `std/agents/grader`
- **Prompt routing** → `std/agents/router`
- **Task decomposition** → `std/agents/planner`
- **Circuit breakers** → `std/circuit`
- **Tiered memory** → `std/memory`
- **Observability** → `std/trace`
- **Subagent QC** → `std/agents/monitor`
- **Learning** → `std/agents/reviewer`
- **Embeddings** → `MCP Embeddings`

## Bottom Line

12 stdlib modules. Communication/orchestration layer is solid. Two foundational mistakes to fix first: type annotations + checker, and regex literals. Then the full stdlib buildout: 5 new modules, 6 standard agents, 2 MCP declarations. An agent language's stdlib includes agents. See `NEXT_PROMPT.md` for priority order, `design/stdlib_roadmap.md` for the full plan.
