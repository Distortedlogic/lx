# Reference

Stable how-to guides and codebase layout. Changes infrequently.

## Codebase Layout

```
crates/lx/src/
  backends/    mod.rs (traits + RuntimeCtx), defaults.rs (standard impls)
  lexer/       mod.rs, numbers.rs, strings.rs
  parser/      mod.rs, func.rs, infix.rs, paren.rs, pattern.rs, prefix.rs, refine.rs, statements.rs, type_ann.rs
  checker/     mod.rs, synth.rs, types.rs
  interpreter/ mod.rs, agents.rs, apply.rs, collections.rs, eval.rs, modules.rs, patterns.rs, refine.rs, shell.rs
  builtins/    mod.rs, call.rs, str.rs, coll.rs, hof.rs, hof_extra.rs
  stdlib/      mod.rs + 61 module files
  ast.rs, token.rs, value.rs, value_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/main.rs
doc/           35 quick-reference docs for implemented features
spec/          51 specs (10 eliminated by merges in Session 46, 7 added in Session 48)
agent/         Context files (this folder)
tests/         66 test suites (.lx files + 11_modules dir)
  fixtures/    agent_echo.lx, mcp_test_server.py, yield_orchestrator.py, http_test_server.py, yield_simple.lx, yield_multi.lx, yield_pipeline.lx
flows/
  lib/         10 reusable .lx library modules
  examples/    13 .lx programs translating arch_diagrams
  specs/       14 target goal + scenario specs
```

## Adding a Stdlib Module

1. Create `crates/lx/src/stdlib/mymod.rs` with `pub fn build() -> IndexMap<String, Value>` returning functions via `mk("mymod.fn_name", arity, bi_fn)`
2. Register in `crates/lx/src/stdlib/mod.rs`: add `mod mymod;`, add `"mymod" => mymod::build()` in `get_std_module`, add `| "mymod"` in `std_module_exists`
3. Write test in `tests/NN_mymod.lx`
4. Builtins calling lx functions use `crate::builtins::call_value(f, arg, span, ctx)` (see `builtins/hof.rs` for examples, `builtins/call.rs` for implementation)

## Adding Agent Extensions

Extensions to `std/agent` follow the split-file pattern:
1. Create `crates/lx/src/stdlib/agent_feature.rs` with `pub fn mk_feature() -> Value` returning the builtin
2. Register `mod agent_feature;` in `stdlib/mod.rs`
3. Insert into agent module map in `agent.rs`'s `build()`: `m.insert("feature".into(), super::agent_feature::mk_feature())`
4. For `BuiltinFunc` values with pre-applied args: set `arity` = total args (pre-applied + user-supplied), not just user-supplied count
5. Protocols exposed as uppercase keys (e.g., `"Handoff"`) require selective import: `use std/agent {Handoff}`

## Adding Language-Level Features (keywords, AST nodes)

For new keywords like `Trait`, `Protocol`, `with ... as`:
1. **Token**: add variant to `token.rs`'s `TokenKind` enum
2. **Lexer**: add keyword recognition in `lexer/mod.rs` (lowercase → keyword table at ~line 330; uppercase → TypeName special-case at ~line 345)
3. **AST**: add node to `ast.rs`'s `Expr` or `Stmt` enum
4. **Parser**: handle in `parser/prefix.rs` (expressions) or `parser/statements.rs` (declarations) + add to `parse_stmt` dispatch in `parser/mod.rs`
5. **Interpreter**: add eval case in `interpreter/mod.rs` (or `eval.rs` / `agents.rs` for method impls)
6. **Checker**: add synth case in `checker/synth.rs` and stmt case in `checker/mod.rs`
7. **Diag walker**: add walk case in `stdlib/diag_walk.rs`
8. **Module exports**: add export case in `interpreter/modules.rs`
9. **Value** (if runtime representation needed): add variant to `value.rs`, update `structural_eq`, `hash_value`, `value_display.rs`

## Error Messages

When adding errors, follow these rules:

- Show actual value and type: `format!("expected Bool, got {} `{}`", val.type_name(), val.short_display())`
- Use `val.short_display()` (80 char cap), never raw `{val}` in errors
- Undefined variable hints: `keyword_hint()` in `interpreter/mod.rs` maps 30+ cross-language keywords to lx equivalents
- Binding pattern hints: `binding_pattern_hint()` detects `mut`/`let`/`var` and suggests `:=`
- Pattern display: `Pattern` impl Display in `ast.rs` for readable error output

## Running Flows

`flows/examples/*.lx` are lx translations of real agentic architectures from `~/repos/mcp-toolbelt/packages/arch_diagrams/`. Each has a matching spec in `flows/specs/`. `flows/lib/*.lx` are reusable library modules imported by the examples. Run with `just run flows/examples/research.lx`. Most require actual agent subprocesses or MCP servers — they're structural demonstrations, not standalone tests.

## Flow → Module Mapping

| Flow (examples/)    | Uses                                                                         |
| ------------------- | ---------------------------------------------------------------------------- |
| agentic_loop        | std/ai, std/circuit, std/tasks, std/agents/auditor                           |
| agent_lifecycle     | std/ai, std/memory, std/agents/reviewer, std/cron                            |
| fine_tuning         | std/ai, std/trace, MCP Embeddings                                            |
| full_pipeline       | std/ai, std/tasks, std/agents/grader, std/agents/planner, std/agents/monitor |
| security_audit      | std/agents/monitor, std/circuit                                              |
| research            | std/ai, std/agents/router, std/tasks                                         |
| perf_analysis       | std/ai, std/agents/router, std/tasks                                         |
| project_setup       | std/tasks, MCP Workflow                                                      |
| post_hoc_review     | std/ai, std/agents/reviewer, std/memory, std/trace                           |
| discovery_system    | std/ai, std/tasks, std/trace, MCP Embeddings                                 |
| tool_generation     | std/ai, std/tasks, std/agents/auditor                                        |
| defense_layers      | std/agents/monitor, std/circuit, std/trace, capability attenuation           |
| mcp_tool_audit      | std/tasks, std/audit                                                         |
| (any flow)          | std/diag (visualize any flow's structure)                                    |

| Library (lib/)      | Purpose                                                                      |
| ------------------- | ---------------------------------------------------------------------------- |
| catalog             | Tool/capability catalog management                                           |
| dispatch            | Message dispatch helpers                                                     |
| grading             | Output grading utilities                                                     |
| guard               | Guard/validation patterns                                                    |
| memory              | Memory management patterns                                                   |
| react               | ReAct loop implementation                                                    |
| report              | Report generation utilities                                                  |
| scoring             | Scoring/ranking helpers                                                      |
| transcript          | Conversation transcript handling                                             |
| workflow            | Workflow composition patterns                                                |
