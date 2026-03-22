# Goal

Add guard clauses to function definitions so functions can have multiple clauses dispatched by conditions on their parameters. Currently guards only work on `match` arms — function-level guards would eliminate nested conditionals in agentic dispatch functions and align with Elixir/Erlang patterns.

# Why

- Agent dispatch functions frequently branch on input shape: `action == "think"` vs `action == "plan"` vs `action == "reflect"`. Currently this requires a match or ternary inside a single function body. Multi-clause functions with guards read as a dispatch table.
- Elixir's `def foo(x) when is_integer(x)` is the model for function-level guards (Erlang, Elixir, Haskell all have this pattern).
- This is syntactic sugar — it desugars to a match on the function arguments. Low implementation risk.

# What Changes

## Syntax: function clauses with `&` guard

```
clamp = (x min max) & (min <= max) {
  x < min ? min : (x > max ? max : x)
}
clamp = (x min max) & (min > max) {
  error "min must be <= max"
}
```

Multiple bindings to the same name create a multi-clause function. At call time, clauses are tried in order. The first clause whose guard evaluates to true is selected. If no clause matches, a runtime error is raised.

## Implementation approach

Multi-clause functions are stored as `LxVal::MultiFunc(Vec<LxFunc>)`. Each `LxFunc` gains an optional `guard: Option<Arc<SExpr>>` field (using `Arc` to match the existing `body: Arc<SExpr>` pattern). Application tries each clause: evaluate the guard in an env where params are bound to the args. If the guard returns true, evaluate the body. If false, try the next clause.

Alternatively, desugar at parse time: if a function name is bound multiple times in the same scope and all bindings are functions with guards, combine them into a single function with a match.

## Parser changes

In `crates/lx/src/parser/expr.rs`, in the `func_def` parser (around line 277), after parsing the parameter list `(params)` and optional return type annotation, check for `&` (`TokenKind::Amp`) followed by a parenthesized guard expression before the body. Store the guard in the `Expr::Func` variant. Note: `&` is already used for match arm guards in the same file (line 406: `just(TokenKind::Amp).ignore_then(expr.clone()).or_not()`) — function guards should use the same pattern. `&` is also used as a low-precedence binary `And` operator (line 468), but this should not conflict since the function guard `&` appears between the param list `)` and the body expression, where a binary operator cannot legally occur.

In `crates/lx/src/parser/stmt.rs`, when a `Binding` re-binds a name that's already bound to a `Func` with a guard, combine them into a `MultiFunc`.

## AST changes

Add `guard: Option<Box<SExpr>>` to `Expr::Func` in `crates/lx/src/ast/mod.rs`.

# Files Affected

**Modified files:**
- `crates/lx/src/ast/mod.rs` — add `guard` field to `Func` variant
- `crates/lx/src/parser/expr.rs` — parse `& (guard_expr)` after params in `func_def` parser
- `crates/lx/src/interpreter/exec_stmt.rs` — detect multi-clause function bindings in `BindTarget::Name` handling
- `crates/lx/src/interpreter/apply.rs` — try clauses in order when applying multi-clause func; also update `eval_func` which constructs `LxFunc`
- `crates/lx/src/value/func.rs` — add `guard` field to `LxFunc`
- `crates/lx/src/value/mod.rs` — add `MultiFunc` variant to `LxVal`
- `crates/lx/src/visitor/walk/walk_expr.rs` — walk guard expression in `walk_func` (line 87)
- `crates/lx/src/visitor/walk/mod.rs` — update `Expr::Func` dispatch in `walk_expr` (line 77-79) to pass the guard
- `crates/lx/src/visitor/mod.rs` — update `visit_func` trait method signature

**New files/dirs:**
- `tests/` directory (does not exist yet; `just test` runs `cargo run -p lx-cli -- test tests/`)
- `tests/function_guards.lx` — tests for guarded function clauses

# Task List

### Task 1: Add guard field to Func AST and parse it

**Subject:** Parse `& (guard)` after function parameter list

**Description:** In `crates/lx/src/ast/mod.rs`, add `guard: Option<Box<SExpr>>` to the `Func` variant of `Expr`. The current variant is `Func { params: Vec<Param>, ret_type: Option<SType>, body: Box<SExpr> }`.

In `crates/lx/src/parser/expr.rs`, in the `func_def` parser (around line 277), after parsing the parameter list and optional return type annotation, check if the next token is `&` (`TokenKind::Amp`). If so, consume it and parse the guard expression (in parentheses). Store it in the `guard` field. If no `&`, set `guard: None`. Follow the same pattern used for match arm guards on line 406: `just(TokenKind::Amp).ignore_then(expr.clone()).or_not()`.

Update the single `Func` construction site in `expr.rs` (line 282) to pass `guard: None` (or the parsed guard).

Update the visitor walker in `crates/lx/src/visitor/walk/walk_expr.rs`: update `walk_func` (line 87) to accept and walk the guard expression if present. Update the `Expr::Func` dispatch in `crates/lx/src/visitor/walk/mod.rs` (line 77-79) to pass the guard. Also update the `visit_func` trait method signature in `crates/lx/src/visitor/mod.rs` (line 74).

Run `just diagnose`.

**ActiveForm:** Adding guard field to Func and parsing it

---

### Task 2: Implement multi-clause function combination

**Subject:** Combine same-name guarded function bindings into multi-clause dispatch

**Description:** In `crates/lx/src/value/func.rs`, add a `guard: Option<Arc<SExpr>>` field to `LxFunc` (use `Arc<SExpr>` to match the existing `body: Arc<SExpr>` convention). Current `LxFunc` fields: `params`, `defaults`, `body`, `closure`, `arity`, `applied`, `source_text`, `source_name`. Add a `MultiFunc(Vec<LxFunc>)` variant to `LxVal` in `crates/lx/src/value/mod.rs`.

In `crates/lx/src/interpreter/exec_stmt.rs`, in the `BindTarget::Name(name)` branch: currently, if `has_mut(name)` is false, a child env is created and the name is bound (shadowing any previous binding). To support multi-clause functions, before creating the child env, check whether the current env already has a binding for `name` that is a `Func` (with a guard) or `MultiFunc`. If the new value is also a `Func` with a guard, combine them into a `MultiFunc` instead of shadowing. If the old binding is already a `MultiFunc`, append the new clause. This preserves clause ordering (first defined = first tried).

Also update `eval_func` in `crates/lx/src/interpreter/apply.rs` (line 136-168) to populate the new `guard` field on `LxFunc` (set to `None` for now; the guard from the AST will need to be threaded through).

Run `just diagnose`.

**ActiveForm:** Implementing multi-clause function combination

---

### Task 3: Implement multi-clause dispatch in apply and write tests

**Subject:** Try clauses in order during function application

**Description:** In `crates/lx/src/interpreter/apply.rs`, add handling for `LxVal::MultiFunc(clauses)`:
1. For each clause in order: bind params to args in a temporary env, evaluate the guard in that env.
2. If guard returns true (or is None), evaluate the body and return.
3. If guard returns false, try the next clause.
4. If no clause matches, return `Err("no matching clause for function")`.

For single `LxFunc` with a guard: same logic but only one clause — error if guard fails.

Create `tests/` directory if it does not exist, then create `tests/function_guards.lx`:
1. **Basic guard** — `abs = (x) & (x >= 0) { x }` and `abs = (x) & (x < 0) { 0 - x }`. Verify `abs 5 == 5` and `abs (-3) == 3`.
2. **Guard with multiple params** — `clamp` example from above.
3. **Fallback clause** — guarded clauses + one unguarded clause as catch-all.
4. **No match error** — all clauses have guards, none match. Verify runtime error.
5. **Guard with complex expression** — guard that calls a builtin function.

Run `just diagnose` and `just test`.

**ActiveForm:** Implementing multi-clause dispatch and writing tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/FUNCTION_GUARDS.md" })
```

Then call `next_task` to begin.
