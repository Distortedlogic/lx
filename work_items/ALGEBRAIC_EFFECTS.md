# Goal

Add algebraic effects as a language-level construct that unifies `emit`, `yield`, and the pluggable backend trait system into a single mechanism. Effects let lx programs declare what side effects they perform and let callers decide how those effects are handled — making effects testable, composable, and swappable without the Rust-level `Arc<dyn Trait>` ceremony.

# Why

- lx already has effect-like operations (`emit`, `yield`, shell calls, HTTP requests) routed through backend traits. But adding a new effect requires: defining a Rust trait, implementing default + deny backends, wiring into `RuntimeCtx`, updating sandbox policies. That's 5+ files for what should be a one-liner.
- The research in `research/error-handling/landscape.md` identifies algebraic effects as the generalization of exceptions/generators/async/coroutines. Common Lisp's separation of policy (handlers) from mechanism (restarts) is the same principle.
- Effects compose naturally — a handler can intercept an effect, modify it, and re-raise it, or resume with a different value. This enables middleware patterns (logging, retrying, rate-limiting) without wrapping.
- Testing becomes trivial: mock any effect by providing a test handler instead of constructing a mock backend struct.

# What Changes

## New syntax: `effect` declaration and `handle`/`perform`

**Effect declaration** (top-level):
```
effect AskUser = (prompt: Str) -> Str
effect LogAction = (level: Str, msg: Str) -> ()
```

**Perform** (inside functions):
```
answer = perform AskUser "Continue?"
perform LogAction "info" "User said: {answer}"
```

**Handle** (at call site):
```
handle {
  my_workflow input
} with {
  AskUser prompt -> resume "yes"
  LogAction level msg -> { emit "{level}: {msg}"; resume () }
}
```

`resume` continues execution at the `perform` site with the given value. Without `resume`, the handler's return value replaces the entire `handle` block result.

## AST additions

- `Stmt::EffectDecl { name: String, params: Vec<Param>, ret_type: Option<SType> }` — effect type declaration
- `Expr::Perform { effect: String, args: Vec<SExpr> }` — perform an effect
- `Expr::Handle { body: Box<SExpr>, handlers: Vec<EffectHandler> }` — handle effects from body
- `EffectHandler { effect: String, params: Vec<String>, body: SExpr }` — handler arm

## Runtime

Effects are implemented via Rust's async machinery. `perform` suspends the current computation and sends the effect + args to the nearest `handle` block. The handler evaluates its body. If the handler calls `resume(value)`, the suspended computation continues with `value` as the result of `perform`. If the handler doesn't call `resume`, the `handle` block returns the handler's result.

Implementation approach: each `handle` block installs an effect handler table in the interpreter's environment. `perform` walks the env chain to find the nearest handler for the named effect. The handler receives a `resume` closure that, when called, continues the suspended computation.

## Type checking

`perform EffectName args` has the return type declared in the effect declaration. `handle` blocks type-check the body with the assumption that all declared effects are handled, and each handler's return type must match the effect's declared return type (when using `resume`).

# Files Affected

**Modified files:**
- `crates/lx/src/ast/mod.rs` — add EffectDecl, Perform, Handle, EffectHandler variants
- `crates/lx/src/ast/expr_types.rs` — add EffectHandler struct
- `crates/lx/src/lexer/helpers.rs` — add `effect`, `perform`, `handle`, `resume` to `ident_or_keyword`
- `crates/lx/src/lexer/token.rs` — add `Effect`, `Perform`, `Handle`, `Resume` variants to `TokenKind`
- `crates/lx/src/parser/expr.rs` — parse `perform` and `handle` expressions
- `crates/lx/src/parser/stmt.rs` — parse `effect` declarations
- `crates/lx/src/error.rs` — add `EffectSignal` variant to `LxError` enum (alongside existing `BreakSignal`, `Propagate`)
- `crates/lx/src/interpreter/eval.rs` — evaluate Perform and Handle (add arms to the `eval` match)
- `crates/lx/src/interpreter/mod.rs` — add effect handler table to `Interpreter` struct (fields: `env`, `source`, `source_dir`, `module_cache`, `loading`, `ctx`)
- `crates/lx/src/env.rs` — store effect handlers in `Env` struct (fields: `bindings: DashMap<String, LxVal>`, `mutables: HashSet<String>`, `parent: Option<Arc<Env>>`)
- `crates/lx/src/checker/stmts.rs` — type-check effect declarations
- `crates/lx/src/checker/synth.rs` — type-check Perform and Handle expressions
- `crates/lx/src/visitor/walk/mod.rs` — walk new Stmt variant in `walk_stmt`
- `crates/lx/src/visitor/walk/walk_expr.rs` — walk new Expr variants in `walk_expr`
- `crates/lx/src/visitor/mod.rs` — add visitor trait methods for new nodes

**New files:**
- `crates/lx/src/interpreter/effects.rs` — effect handler resolution and resume machinery
- `tests/effects.lx` — tests for algebraic effects (the `tests/` directory does not yet exist; create it)

# Task List

### Task 1: Add effect-related keywords and AST nodes

**Subject:** Define effect, perform, handle, resume keywords and AST variants

**Description:** In `crates/lx/src/lexer/token.rs`, add `Effect`, `Perform`, `Handle`, `Resume` variants to the `TokenKind` enum. In `crates/lx/src/lexer/helpers.rs`, add `"effect"`, `"perform"`, `"handle"`, `"resume"` mappings to the `ident_or_keyword` function.

In `crates/lx/src/ast/mod.rs`, add to the `Stmt` enum: `EffectDecl { name: String, params: Vec<Param>, ret_type: Option<SType> }`. (No `span` field -- spans are stored in the `Spanned<Stmt>` wrapper.)

In `crates/lx/src/ast/expr_types.rs`, add: `EffectHandler { effect: String, params: Vec<String>, body: SExpr }`.

In `crates/lx/src/ast/mod.rs`, add to the `Expr` enum:
- `Perform { effect: String, args: Vec<SExpr> }`
- `Handle { body: Box<SExpr>, handlers: Vec<EffectHandler> }`

Update visitor walker to walk new node children.

Run `just diagnose`.

**ActiveForm:** Adding effect keywords and AST nodes

---

### Task 2: Parse effect declarations, perform, and handle

**Subject:** Parser support for effect syntax

**Description:** In `crates/lx/src/parser/stmt.rs`, parse `effect Name = (params) -> RetType` as `Stmt::EffectDecl`. Add the effect statement parser to the `choice` in `stmt_parser`.

In `crates/lx/src/parser/expr.rs`, parse `perform EffectName args...` as `Expr::Perform` — consume the effect name as an identifier, then parse arguments. Add it to the `atom` choice alongside `emit_expr` and `yield_expr`.

Parse `handle { body } with { EffectName params -> handler_body; ... }` as `Expr::Handle`. The `with` block uses the same arm parsing as `match` (see `match_arms` in `pratt_expr`) but the pattern position is `EffectName param_bindings`.

Run `just diagnose`.

**ActiveForm:** Parsing effect syntax

---

### Task 3: Implement effect handler table and perform resolution

**Subject:** Runtime effect handler installation and lookup

**Description:** Create `crates/lx/src/interpreter/effects.rs`. Define `EffectHandlerEntry { effect_name: String, params: Vec<String>, body: SExpr, env: Arc<Env> }`.

In `crates/lx/src/env.rs`, add `effect_handlers: Option<Vec<EffectHandlerEntry>>` to the `Env` struct (alongside existing `bindings`, `mutables`, `parent` fields). The `handle` expression installs handlers in a child env. `perform` walks the env chain (via `parent: Option<Arc<Env>>` pointers) to find the nearest handler for the named effect.

In the interpreter, `Expr::Handle` evaluation:
1. Create child env with the handler entries.
2. Evaluate body in this child env.
3. If body completes normally, return its result.
4. If a `perform` is encountered, execution transfers to the handler.

For the initial implementation, use a simple approach: `perform` returns a special `EffectSignal` error variant on `LxError` (like `LxError::BreakSignal`) that carries the effect name, args, and a resume continuation. The `handle` block catches this signal, matches the effect name, evaluates the handler body, and if `resume` is called, re-invokes the body with the perform site replaced.

Simpler alternative: use `tokio::sync::oneshot` channels. `perform` sends the effect on a channel and awaits the response. `handle` receives from the channel, runs the handler, sends back the resume value. This naturally supports the async execution model.

Run `just diagnose`.

**ActiveForm:** Implementing effect handler resolution

---

### Task 4: Implement resume and write effect tests

**Subject:** Resume continuation and test suite for algebraic effects

**Description:** Complete the `resume` implementation:
- In the handler body, `resume(value)` sends the value back to the suspended `perform` site.
- Without `resume`, the handler's return value becomes the `handle` block's result (short-circuit).

Create `tests/effects.lx` with tests:
1. Basic effect + handle — `perform` returns the handler's resume value.
2. Effect without resume — handler short-circuits the handle block.
3. Multiple effects — handle block with two different effect handlers.
4. Nested handle — inner handler shadows outer for the same effect name.
5. Testing pattern — mock an effect in tests by providing a test handler.

Run `just diagnose` and `just test`.

**ActiveForm:** Implementing resume and writing effect tests

---

### Task 5: Add type checking for effects

**Subject:** Type-check effect declarations, perform, and handle expressions

**Description:** In `crates/lx/src/checker/stmts.rs`, add a `Stmt::EffectDecl` arm to `check_stmt`: register the effect name with its parameter types and return type in the checker's scope (the `Checker` struct has `scope: Vec<HashMap<String, Type>>` — use `self.bind()`).

In `crates/lx/src/checker/synth.rs`, add arms to the `synth` match:
- `Expr::Perform`: look up the effect name in scope, check args against param types, return the declared return type.
- `Expr::Handle`: check the body, check each handler body — the handler params have the types from the effect declaration, and if `resume` is called, its argument must match the effect's return type.

Run `just diagnose` and `just test`.

**ActiveForm:** Type-checking algebraic effects

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
mcp__workflow__load_work_item({ path: "work_items/ALGEBRAIC_EFFECTS.md" })
```

Then call `next_task` to begin.
