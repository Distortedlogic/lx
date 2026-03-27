# &Arc<RuntimeCtx> Root Cause Fix

The `SyncBuiltinFn` type alias at `crates/lx/src/value/func.rs:24` takes `&Arc<RuntimeCtx>` as its third parameter. None of the ~100 sync builtin functions clone the Arc — they only read through it. The `&Arc<T>` adds pointless double indirection. Fix: change to `&RuntimeCtx`.

**Supersedes:** code_cleanup.md Task 6 (which fixes `&Arc<Vec<T>>` in trait_apply and apply — a different but related pattern).

---

## Task 1: Change the SyncBuiltinFn type alias

**File:** `crates/lx/src/value/func.rs`

Line 24: change
```rust
pub type SyncBuiltinFn = fn(&[LxVal], miette::SourceSpan, &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError>;
```
to:
```rust
pub type SyncBuiltinFn = fn(&[LxVal], SourceSpan, &RuntimeCtx) -> Result<LxVal, LxError>;
```

Also add `use crate::runtime::RuntimeCtx;` and `use miette::SourceSpan;` to the file imports (these are also inline import fixes).

---

## Task 2: Update all sync builtin function signatures

Every function registered as a sync builtin has signature `fn bi_*(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> ...`. Change each to `ctx: &RuntimeCtx`.

Affected modules (search for `&Arc<RuntimeCtx>` in function parameters):

- `builtins/register.rs` — all `bi_*` functions
- `builtins/coll.rs` — all `bi_*` functions
- `builtins/str.rs` — all `bi_*` functions
- `builtins/hof.rs` — `call`, `call_predicate`, and all `bi_*` functions
- `builtins/hof_extra.rs` — `extremum_by` and all `bi_*` functions
- `builtins/hof_parallel.rs` — all `bi_*` functions
- `builtins/agent.rs` — all `bi_*` functions
- `builtins/convert.rs` — all `bi_*` functions
- `builtins/shell.rs` — all `bi_*` functions
- `builtins/mod.rs` — `call_value`, `call_value_sync`

For each function: replace `ctx: &Arc<RuntimeCtx>` with `ctx: &RuntimeCtx`. Remove `use std::sync::Arc;` from files where Arc is no longer needed (verify first).

---

## Task 3: Update the two call sites that invoke sync builtins

There are exactly two call sites where a `SyncBuiltinFn` function pointer is invoked:

**Call site 1:** `crates/lx/src/builtins/call.rs:41`
```rust
BuiltinKind::Sync(f) => f(&bf.applied, span, ctx),
```
`call_value` takes `ctx: &Arc<RuntimeCtx>` and MUST keep this signature because it calls `Arc::clone(ctx)` at lines 24, 42, 43, 48 for async builtins and interpreter construction. Only the sync builtin dispatch needs `&RuntimeCtx`.

Fix: dereference at the call site only:
```rust
BuiltinKind::Sync(f) => f(&bf.applied, span, &**ctx),
```

**Call site 2:** `crates/lx/src/interpreter/apply.rs:74`
```rust
BuiltinKind::Sync(f) => Ok(f(&bf.applied, span, &self.ctx)?),
```
`self.ctx` is `Arc<RuntimeCtx>`, so `&self.ctx` is `&Arc<RuntimeCtx>`. Fix: change to `&*self.ctx`:
```rust
BuiltinKind::Sync(f) => Ok(f(&bf.applied, span, &*self.ctx)?),
```

No changes needed to `call_value` or `call_value_sync` signatures — they keep `&Arc<RuntimeCtx>` because they need to clone the Arc for async dispatch.

---

## Task 4: Fix standalone &Arc violations

**File:** `crates/lx/src/interpreter/exec_stmt.rs:166`
Function `maybe_combine_clauses` takes `env: &Arc<Env>`. It does NOT clone the Arc — it only calls `env.get(name)` which auto-derefs. Change parameter to `env: &Env`. Update the call site to pass `&*env` or `env.as_ref()` if the caller has `Arc<Env>`.

**File:** `crates/lx/src/interpreter/trait_apply.rs:82`
Function `try_match_variant` takes `rec: &Arc<indexmap::IndexMap<Sym, LxVal>>`. It does NOT clone the Arc — it only calls `rec.get(...)` which auto-derefs. Change parameter to `rec: &IndexMap<Sym, LxVal>`. Update the call site to pass `&**rec` or `rec.as_ref()` if the caller has `Arc<IndexMap<...>>`.

---

## Verification

Run `just diagnose` after all changes.
