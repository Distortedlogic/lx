# Goal

Close the gaps in the lx type checker (`lx check`) to cover the 5 planned-but-unimplemented features from `spec/toolchain.md` and `spec/diagnostics.md`: exhaustiveness checking for match expressions, mutable capture detection in concurrent contexts, import conflict detection, Trait field type integration, and `--strict` mode. Also fix the 31 residual checker errors on the workspace. 14 concrete tasks.

# Why

- Match expressions with tagged unions have no exhaustiveness warning — a missing variant arm silently falls through to `None` at runtime instead of being caught statically. This is the single most impactful missing check
- `par`/`sel`/`pmap` happily capture `mut` bindings — concurrent mutation of shared state is a correctness bug that the checker should flag
- Two `use` statements can import the same name with no error — the second silently shadows the first, causing subtle bugs when import order changes
- The checker walks `AgentDecl`/`ClassDecl`/`TraitDecl` methods and fields but doesn't cross-reference Trait field types with function parameter annotations — a Trait `{name: Str}` can be constructed with `{name: 42}` and the checker stays silent
- `lx check` has no strict mode — CI can't enforce zero warnings because warnings don't exist (only errors)
- 31 residual errors when running `lx check` on the workspace are a mix of: checker limitations (infinite type on reassignment, negation on pattern-bound vars), missing AST coverage in synth (many `_ => Type::Unknown` arms), and parse errors in brain/flows files. Fixing the checker-side issues will drop this count significantly

# What changes

**Exhaustiveness (Tasks 1-3):** Collect TypeDef variant information during checking. After synthesizing a Match expression over a known union type, compare the arm patterns against the variant list. Emit a warning for missing variants (unless a wildcard `_` arm exists). Wire the warning through diagnostics.

**Mutable capture detection (Tasks 4-5):** Track which bindings are mutable in the checker scope. When entering a `par`/`sel`/`pmap` body, scan free variables in the body expression — if any reference a mutable binding from an outer scope, emit an error.

**Import conflict detection (Task 6):** In `resolve_use`, before binding a name, check if it already exists in the current scope from a prior `use` statement. If so, emit an error with the conflicting import location.

**Trait field type integration (Task 7):** When checking a Trait constructor call (a Trait with non-empty fields applied to a record argument), unify each record field type against the corresponding Trait field type.

**Reduce Unknown escape hatches (Tasks 8-10):** Add synth cases for `Expr::If`, `Expr::Shell`, `Expr::Receive`, `Expr::Send`, `Expr::Ask`, `Expr::StreamAsk`, `Expr::Yield`, `Expr::Refine`, `Expr::Meta`, `Expr::Slice`, `Expr::Index`, `Expr::Section`, `Expr::Interpolated`, `Expr::Reassign`, `Expr::Map`, `Expr::ForEach`, `Expr::Par`/`Sel`/`Pmap` — covering the most common `_ => Type::Unknown` arms that currently suppress useful type information.

**Strict mode (Task 11):** Add `--strict` flag to `lx check` CLI. When enabled, warnings count as errors (nonzero exit).

**Fix residual workspace errors (Tasks 12-14):** Fix infinite type on reassignment (occurs check fires spuriously when reassigning same-type binding). Fix negation on pattern-bound vars (checker doesn't bind pattern vars before checking arm body). Fix remaining parse-error files by adjusting checker to skip unparseable files gracefully.

# Files affected

- `crates/lx/src/checker/mod.rs` — TypeDef variant tracking, import conflict tracking, strict mode plumbing
- `crates/lx/src/checker/synth.rs` — New synth arms, exhaustiveness check after Match, mutable capture analysis, Trait constructor checking
- `crates/lx/src/checker/types.rs` — `Type::Union` population from TypeDef, display updates
- `crates/lx-cli/src/check.rs` — `--strict` flag, warning vs error distinction in output
- `crates/lx-cli/src/main.rs` — Add `strict` flag to `Check` command variant
- `crates/lx/src/checker/exhaust.rs` (new) — Exhaustiveness analysis: variant set construction, pattern coverage, missing variant computation
- `crates/lx/src/checker/capture.rs` (new) — Free variable analysis for concurrent capture detection

# Task List

## Task 1: Track TypeDef variants in checker scope

**Subject:** Record union variant information during type checking
**ActiveForm:** Tracking TypeDef variants in checker

In `crates/lx/src/checker/mod.rs`, add a `type_defs: HashMap<String, Vec<String>>` field to `Checker` that maps type names to their variant names. In `check_stmt` for `Stmt::TypeDef`, populate this map: `self.type_defs.insert(name.clone(), variants.iter().map(|(name, _)| name.clone()).collect())`. Also bind each variant constructor as `Type::Union { name, variants }` instead of `Type::Unknown` — construct `Variant` structs with the appropriate field types from the TypeDef.

Verify: `just diagnose` passes.

## Task 2: Implement exhaustiveness analysis

**Subject:** Check match arm coverage against union variants
**ActiveForm:** Implementing exhaustiveness check

Create `crates/lx/src/checker/exhaust.rs`. Implement `pub fn check_exhaustiveness(type_name: &str, variants: &[String], arms: &[MatchArm]) -> Vec<String>` that returns missing variant names. Walk each arm's pattern: `Pattern::Constructor(name, _)` covers that variant, `Pattern::Wildcard`/`Pattern::Binding(_)` covers all. If all variants are covered or a wildcard exists, return empty. Otherwise return uncovered variant names.

Verify: `just diagnose` passes.

## Task 3: Wire exhaustiveness warnings into Match synthesis

**Subject:** Emit exhaustiveness warnings from match expression checking
**ActiveForm:** Wiring exhaustiveness warnings

In `crates/lx/src/checker/synth.rs`, in the `Expr::Match` arm: after synthesizing the scrutinee, resolve its type. If the resolved type is `Type::Union { name, variants }`, call `exhaust::check_exhaustiveness` with the variant names and the match arms. For each missing variant, call `self.emit(format!("non-exhaustive match on {name}: missing {v}"), expr.span)`. Add `Diagnostic` a `level` field (error vs warning) — exhaustiveness is a warning, not an error. Update `mod.rs` to declare `mod exhaust;`.

Verify: `just diagnose` passes.

## Task 4: Track mutable bindings in checker scope

**Subject:** Record which bindings are mutable for capture analysis
**ActiveForm:** Tracking mutable bindings

In `crates/lx/src/checker/mod.rs`, add `mutables: HashSet<String>` field to `Checker`. In `check_binding`, when the binding is mutable (`b.mutable` is true), insert the name into `self.mutables`. In `push_scope`/`pop_scope`, the mutables set doesn't need scoping — it tracks all mutable bindings in the enclosing function.

Verify: `just diagnose` passes.

## Task 5: Detect mutable captures in concurrent contexts

**Subject:** Error on mutable binding capture in par/sel/pmap
**ActiveForm:** Detecting mutable captures in concurrent contexts

Create `crates/lx/src/checker/capture.rs`. Implement `pub fn free_vars(expr: &SExpr) -> HashSet<String>` that collects all `Expr::Ident` names in the expression tree (minus locally-bound names from inner `Expr::Func` params and `Expr::With` bindings). In `crates/lx/src/checker/synth.rs`, add synth arms for `Expr::Par`/`Expr::Sel`/`Expr::Pmap`. For each sub-expression (the body/callback), compute `free_vars` and intersect with `self.mutables`. For each hit, emit `format!("cannot capture mutable binding `{name}` in concurrent context")`. Register `mod capture;` in `mod.rs`.

Verify: `just diagnose` passes.

## Task 6: Detect import name conflicts

**Subject:** Error on duplicate name imports
**ActiveForm:** Detecting import conflicts

In `crates/lx/src/checker/mod.rs`, add `import_sources: HashMap<String, Span>` to `Checker` that maps imported names to the span of the `use` statement that imported them. In `resolve_use`, before calling `self.bind(name, Type::Unknown)`, check if `import_sources` already contains the name. If so, emit `format!("'{name}' already imported at {existing_span}")`. Otherwise, insert into `import_sources`. This only applies to `UseKind::Selective` and `UseKind::Whole` — aliased imports (`UseKind::Alias`) always use a unique alias.

Verify: `just diagnose` passes.

## Task 7: Check Trait constructor field types

**Subject:** Validate Trait field types at construction sites
**ActiveForm:** Checking Trait constructor field types

In `crates/lx/src/checker/mod.rs`, add `trait_fields: HashMap<String, Vec<(String, Type)>>` to `Checker`. In `check_stmt` for `Stmt::TraitDecl`, if the trait has non-empty fields, resolve each field's type annotation and store in `trait_fields`. In `synth.rs` for `Expr::Apply`, when the function resolves to a known Trait name (look up in `trait_fields`) and the argument is an `Expr::Record`, unify each record field type against the corresponding Trait field type. Mismatches produce an error.

Verify: `just diagnose` passes.

## Task 8: Add synth arms for control flow expressions

**Subject:** Reduce Unknown returns from control flow expressions
**ActiveForm:** Adding synth arms for if/for/receive

In `crates/lx/src/checker/synth.rs`, add synth cases for:
- `Expr::If { cond, then_, else_ }` — check cond is Bool, unify then/else branches (same as Ternary)
- `Expr::ForEach { binding, iter, body }` — synth iter, bind the iteration variable, synth body, return `Type::List` of body type
- `Expr::Receive { arms }` — synth each arm body, unify all arm types (like Match)
- `Expr::Yield(_)` — return `Type::Unknown` (yield returns orchestrator response, unknowable)

Verify: `just diagnose` passes. `just test` passes.

## Task 9: Add synth arms for agent messaging expressions

**Subject:** Type agent send/ask/stream expressions
**ActiveForm:** Adding synth arms for send/ask/stream

In `crates/lx/src/checker/synth.rs`, add synth cases for:
- `Expr::Send { target, msg }` — synth both, return `Type::Unit`
- `Expr::Ask { target, msg }` — synth both, return `Type::Result { ok: Unknown, err: Unknown }`
- `Expr::StreamAsk { target, msg }` — synth both, return `Type::Unknown` (stream type not in checker yet)
- `Expr::Shell(_)` — return `Type::Result { ok: Type::Str, err: Type::Str }`

Verify: `just diagnose` passes. `just test` passes.

## Task 10: Add synth arms for collection/string expressions

**Subject:** Type slice, index, interpolation, map expressions
**ActiveForm:** Adding synth arms for collection ops

In `crates/lx/src/checker/synth.rs`, add synth cases for:
- `Expr::Index { expr, index }` — synth expr; if List return element type, if Record/Map return value type, else Unknown
- `Expr::Slice { expr, .. }` — synth expr, return same type (slice of list is list)
- `Expr::Interpolated(parts)` — synth each part, return `Type::Str`
- `Expr::Section { op, operand }` — return `Type::Func { param: Unknown, ret: <op result type> }`
- `Expr::Reassign { name, value }` — synth value, unify with existing binding type if known, return `Type::Unit`

Verify: `just diagnose` passes. `just test` passes.

## Task 11: Add `--strict` flag to `lx check`

**Subject:** Warnings-as-errors mode for CI
**ActiveForm:** Adding --strict flag

In `crates/lx-cli/src/main.rs`, add `strict: bool` field to the `Check` command variant with `#[arg(long)]`. In `crates/lx/src/checker/mod.rs`, add `level: DiagLevel` field to `Diagnostic` with `enum DiagLevel { Error, Warning }`. In `crates/lx-cli/src/check.rs`, accept the `strict` param. When displaying diagnostics, prefix warnings with "warning" and errors with "error". In strict mode, warnings count toward the error total and cause nonzero exit. In normal mode, warnings are displayed but don't affect exit code.

Verify: `just diagnose` passes.

## Task 12: Fix infinite type on reassignment

**Subject:** Prevent spurious occurs-check failure on same-type reassignment
**ActiveForm:** Fixing infinite type on reassignment

In `crates/lx/src/checker/mod.rs`, in `check_binding` for `BindTarget::Reassign`: the current code looks up the existing type and unifies with the new value type. If the existing type contains a `Var` that was freshly allocated for this same binding, the occurs check can fire spuriously. Fix: resolve the existing type deeply before unifying (`self.table.resolve_deep(&existing)`) to eliminate stale type variables.

Verify: `just diagnose` passes. Run `lx check` on a file with `x <- x + 1` pattern to verify no infinite type error.

## Task 13: Bind pattern variables before checking arm bodies

**Subject:** Pattern-bound variables available in match arm body
**ActiveForm:** Binding pattern vars in match arms

In `crates/lx/src/checker/synth.rs`, in the `Expr::Match` arm: before synthesizing each arm body, walk the arm's pattern and bind extracted variables into a new scope. `Pattern::Binding(name)` binds as `Type::Unknown`. `Pattern::Constructor(_, fields)` binds each field name. `Pattern::Tuple(pats)` / `Pattern::List(pats)` recurse. Push scope before each arm, pop after. This prevents "undefined variable" false positives on pattern-bound names.

Verify: `just diagnose` passes. Run `lx check` on a file using pattern-bound variables.

## Task 14: Graceful handling of parse-error files in workspace check

**Subject:** Skip unparseable files without counting as checker errors
**ActiveForm:** Handling parse errors gracefully

In `crates/lx-cli/src/check.rs`, in `check_workspace`: when `read_and_parse` returns `Err`, distinguish parse errors from I/O errors. For parse errors, report them as parse diagnostics (not type errors) and track them separately in output. Display a "parse" count separate from "type" count in the summary. This makes the 31-error count honest — users can see which are parse issues (fix the .lx files) vs type issues (fix the checker).

Verify: `just diagnose` passes.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
