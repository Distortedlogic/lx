-- Memory: ROM. Codebase layout and how-to guides for implementation work.
-- Update when file structure changes or new how-to patterns emerge.

# Reference

## Codebase Layout

```
crates/lx/src/
  ast/         AST node definitions + type annotation AST
  backends/    RuntimeCtx struct, backend traits (Ai/Emit/Http/Shell/Yield/Log/User), default impls
  lexer/       Tokenizer — mod, numbers, strings, keywords, helpers
  parser/      Recursive descent — mod + split files per feature (func, infix, prefix, pattern, statements, etc.)
  checker/     Bidirectional type checker — mod, synth, types
  interpreter/ Tree-walking evaluator — mod + split files (agents, apply, eval, modules, patterns, etc.)
  builtins/    Built-in functions — mod, call, str, coll, hof, convert, register, etc.
  visitor/     AST visitor/walker infrastructure
  stdlib/      40 registered modules across ~86 .rs files (use `std_module_exists` in mod.rs as source of truth)
  token.rs, value.rs, value_display.rs, value_impls.rs, ast_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/  main.rs, manifest.rs, testing.rs, listing.rs, run.rs, agent_cmd.rs
doc/           35 quick-reference docs
spec/          51 spec files
agent/         Context files (this folder)
tests/         72 test suites (71 .lx files + 11_modules dir)
  fixtures/    Test helpers (agent_echo.lx, orchestrators, servers, test flows)
flows/
  lib/         15 reusable .lx library modules
  examples/    14 .lx programs translating arch_diagrams
  specs/       14 target goal + scenario specs
  tests/       Flow satisfaction test suites
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

For new keywords like `Agent`, `Trait`, `Protocol`, `with ... as`:
1. **Token**: add variant to `token.rs`'s `TokenKind` enum
2. **Lexer**: add keyword recognition in `lexer/mod.rs` (lowercase → keyword table at ~line 330; uppercase → TypeName special-case at ~line 345)
3. **AST**: add node to `ast.rs`'s `Expr` or `Stmt` enum
4. **Parser**: handle in `parser/prefix.rs` (expressions) or `parser/statements.rs` (declarations) + add to `parse_stmt` dispatch in `parser/mod.rs`
5. **Interpreter**: add eval case in `interpreter/mod.rs` (or `eval.rs` / `agents.rs` for method impls)
6. **Checker**: add synth case in `checker/synth.rs` and stmt case in `checker/mod.rs`
7. **Diag walker**: add walk case in `stdlib/diag_walk.rs`
8. **Module exports**: add export case in `interpreter/modules.rs`
9. **Value** (if runtime representation needed): add variant to `value.rs`, update `structural_eq`, `hash_value`, `value_display.rs`

## Module Resolution

`interpreter/modules.rs` handles all `use` statements. Resolution order in `eval_use`:

1. **Stdlib** — `std_module_exists(&path)` checks if it's a built-in module
2. **Workspace member** — `resolve_workspace_module(&path)` checks if `path[0]` matches a workspace member name (requires `path.len() >= 2`). Resolves rest of path from member's root dir. Member map lives on `RuntimeCtx.workspace_members` (populated by CLI).
3. **Relative** — `resolve_module_path(source_dir, &path)` handles `./` and `../` prefixes

Key functions: `eval_use` (dispatch), `load_module` (parse + execute + cache), `collect_exports` (extract `+` bindings). Module cache keyed by canonical path prevents double-loading. `loading` set detects circular imports.

## Modifying the CLI

CLI lives in `crates/lx-cli/src/`. `main.rs` has the clap `Command` enum and dispatch.

1. **Add subcommand**: add variant to `Command` enum in `main.rs`, add match arm in `main()`
2. **Add flag to existing command**: add `#[arg]` field to the variant struct
3. **Workspace-aware commands**: use `manifest::find_workspace_root` + `manifest::load_workspace` to discover members. For member filtering: accept `-m`/`--member` flag, filter `ws.members` by name.
4. **Populate RuntimeCtx for workspace imports**: call `manifest::try_load_workspace_members()` and set `ctx.workspace_members` before running any lx code. Without this, `use member/path` won't resolve.

## Error Messages

When adding errors, follow these rules:

- Show actual value and type: `format!("expected Bool, got {} `{}`", val.type_name(), val.short_display())`
- Use `val.short_display()` (80 char cap), never raw `{val}` in errors
- Undefined variable hints: `keyword_hint()` in `interpreter/mod.rs` maps 30+ cross-language keywords to lx equivalents
- Binding pattern hints: `binding_pattern_hint()` detects `mut`/`let`/`var` and suggests `:=`
- Pattern display: `Pattern` impl Display in `ast.rs` for readable error output

## std/diag Architecture

Four files: `diag.rs` (API + mermaid render), `diag_walk.rs` (walker, pre-registration),
`diag_walk_expr.rs` (expression handler with uncurry/classify/handle), `diag_helpers.rs`
(pure helpers). Utility modules (prompt, json, math, etc.) excluded from diagrams to reduce
noise. Pre-registration pass solves forward references. Resource args scanned for tracked
variables across all curried positions.

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
| software_diffusion  | std/ai, std/tasks, std/agents/planner                                        |
| (any flow)          | std/diag (visualize any flow's structure)                                    |

| Library (lib/)      | Purpose                                                                      |
| ------------------- | ---------------------------------------------------------------------------- |
| catalog             | Tool/capability catalog management                                           |
| dispatch            | Message dispatch helpers                                                     |
| github              | GitHub API interaction patterns                                              |
| grading             | Output grading utilities                                                     |
| guard               | Guard/validation patterns                                                    |
| guidance            | Guidance/instruction patterns                                                |
| mcp_session         | MCP session management                                                       |
| memory              | Memory management patterns                                                   |
| react               | ReAct loop implementation                                                    |
| report              | Report generation utilities                                                  |
| scoring             | Scoring/ranking helpers                                                      |
| specialists         | Specialist agent patterns                                                    |
| training            | Training/fine-tuning patterns                                                |
| transcript          | Conversation transcript handling                                             |
| workflow            | Workflow composition patterns                                                |
