# Implementation Phases

Each phase produces a working, testable increment.

## Completed Phases (1-10)

| Phase | What | Status |
|-------|------|--------|
| 1 | Lexer + literal expressions + arithmetic | DONE |
| 2 | Functions, pipes, sections, currying | DONE |
| 3 | Collections + pattern matching | DONE |
| 4 | Iteration + HOFs (map/filter/fold/etc.) | DONE |
| 5 | Error handling (Result/Maybe, `^`, `??`) | DONE |
| 6 | Shell integration (`$`, `$^`, `${ }`) | DONE |
| 7 | Modules (`use`, `+` exports) + type checker | DONE |
| 8 | Concurrency (`par`, `sel`, `pmap` — sequential impl) | DONE |
| 9 | Standard library (29 modules) | DONE |
| 10 | Toolchain (`lx test`, `lx check`, `lx agent`, `lx diagram`) | DONE (partial — `lx fmt`/`lx repl`/`lx watch` not yet) |

## Completed Stdlib Build-out (Sessions 33-38)

| What | Session | Status |
|------|---------|--------|
| std/ai, std/tasks, std/audit, std/circuit, std/knowledge, std/plan, std/introspect | 33 | DONE |
| Standard agents (auditor/router/grader/planner/monitor/reviewer), std/memory, std/trace | 35 | DONE |
| std/diag + `lx diagram` CLI | 36 | DONE |
| std/saga, RuntimeCtx backends, `refine` expression, `agent.reconcile` | 37 | DONE |
| trace.improvement_rate, trace.should_stop (diminishing returns) | 38 | DONE |

## Remaining Work

Remaining features are tracked in `NEXT_PROMPT.md` (priorities 23-47) and `stdlib_roadmap.md`. Key categories:

**Stdlib extensions** (no parser changes):
- agent.dialogue, agent.intercept, Handoff Protocol, agent.supervise/gate/capabilities
- ai.prompt_structured, plan.run_incremental, agent.mock, agent.dispatch
- trace causal spans, workflow.peers/share, Goal/Task Protocols

**New stdlib modules** (no parser changes):
- std/budget, std/reputation, std/skill, std/context, std/prompt, std/strategy, std/durable

**Parser + interpreter changes**:
- `emit` keyword (EmitBackend exists, AST node + parser needed)
- `|>>` streaming pipe (reactive dataflow)
- `with context` (ambient context propagation)
- `caller` implicit binding + `_priority` (binary)
- `Skill` declarations (new keyword)
- `durable` expression (new keyword)
- Deadlock detection (runtime wait-for graph)

**Toolchain**:
- `lx fmt`, `lx repl`, `lx watch`
- Unicode in lexer (byte vs char indexing bug)
