# Unit 02: Sync Builtin Borrow Cleanup

## Goal

Remove the verified `&Arc<T>` audit violations from the sync builtin ABI and the many sync builtin call sites that only borrow context. Keep `Arc` only where ownership is actually required across async boundaries or where the code must `Arc::clone` into stored state.

## Preconditions

- Unit 01 should be complete first so the CLI and test surface is already importing from defining crates.

## Verified Findings

- `crates/lx-value/src/builtin_ctx.rs` defines `BuiltinCtx::event_stream(&self) -> &Arc<EventStream>`.
- `crates/lx-value/src/value/func.rs` defines `SyncBuiltinFn = fn(&[LxVal], SourceSpan, &Arc<dyn crate::BuiltinCtx>) -> Result<LxVal, LxError>`.
- `crates/lx-eval/src/runtime/mod.rs` and `crates/lx-eval/src/builtins/call.rs` implement the same borrowed-`Arc` trait surface.
- The exact command below returns a large set of sync builtin functions that borrow `&Arc<dyn BuiltinCtx>` even though they only read from the trait object and never `Arc::clone` the context:

```text
rg -n '^\\s*(pub\\s+)?(async\\s+)?fn .*&Arc<|^\\s*fn .*&Arc<' crates -g '*.rs'
```

- The verified high-volume hit areas are:
  - `crates/lx-eval/src/builtins/*.rs`
  - `crates/lx-eval/src/runtime/channel_registry.rs`
  - `crates/lx-eval/src/stdlib/**/*.rs`
  - `crates/lx-eval/src/interpreter/ambient.rs`
- Not every `&Arc<_>` hit is a violation. The following categories really do need shared-ownership access and must stay `Arc`-based:
  - async builtin function types that cross `.await`
  - `call_value(...)` and helpers that must clone the context for async builtins
  - `Env::child(self: &Arc<Self>)` because it stores a cloned parent `Arc`
  - helper functions that intentionally store or clone an `Arc`, such as MCP tool wrappers or background-task wiring

## Files to Modify

- `crates/lx-value/src/builtin_ctx.rs`
- `crates/lx-value/src/value/func.rs`
- `crates/lx-eval/src/runtime/mod.rs`
- `crates/lx-eval/src/builtins/call.rs`
- `crates/lx-eval/src/runtime/channel_registry.rs`
- Every sync builtin or stdlib module returned by:
  - `rg -l '^\\s*(pub\\s+)?(async\\s+)?fn .*&Arc<dyn BuiltinCtx>|^\\s*fn .*&Arc<dyn BuiltinCtx>' crates/lx-eval/src -g '*.rs'`

## Steps

### Step 1: Change the borrowed trait surface to the underlying type

Update the core trait and sync builtin ABI:

- In `crates/lx-value/src/builtin_ctx.rs`
  - Change `fn event_stream(&self) -> &Arc<EventStream>;`
  - To `fn event_stream(&self) -> &EventStream;`
- In `crates/lx-value/src/value/func.rs`
  - Change `SyncBuiltinFn`
  - From `fn(&[LxVal], SourceSpan, &Arc<dyn crate::BuiltinCtx>) -> Result<LxVal, LxError>`
  - To `fn(&[LxVal], SourceSpan, &dyn crate::BuiltinCtx) -> Result<LxVal, LxError>`

Do not change `AsyncBuiltinFn` or `DynAsyncBuiltinFn`. Those still need owned `Arc<dyn BuiltinCtx>` because they cross async boundaries and may outlive the caller frame.

### Step 2: Update the concrete `BuiltinCtx` implementations

Update the trait implementations in:

- `crates/lx-eval/src/runtime/mod.rs`
- `crates/lx-eval/src/builtins/call.rs`

Make both `event_stream()` implementations return `&EventStream` instead of `&Arc<EventStream>`.

Keep `RuntimeCtxWrapper(pub Arc<RuntimeCtx>)` and `wrap_runtime_ctx(...) -> Arc<dyn BuiltinCtx>` intact. That wrapper is still the correct ownership boundary for async execution.

### Step 3: Rewrite only the borrow-only sync builtin signatures

For every sync builtin/stdlib function that currently takes `ctx: &Arc<dyn BuiltinCtx>` or `_ctx: &Arc<dyn BuiltinCtx>` and never clones the `Arc`, change the parameter to `&dyn BuiltinCtx`.

This includes the high-volume mechanical sites in:

- `crates/lx-eval/src/builtins/coll.rs`
- `crates/lx-eval/src/builtins/coll_transform.rs`
- `crates/lx-eval/src/builtins/convert.rs`
- `crates/lx-eval/src/builtins/register.rs`
- `crates/lx-eval/src/builtins/shell.rs`
- `crates/lx-eval/src/builtins/str.rs`
- `crates/lx-eval/src/runtime/channel_registry.rs`
- `crates/lx-eval/src/stdlib/channel.rs`
- `crates/lx-eval/src/stdlib/checkpoint.rs`
- `crates/lx-eval/src/stdlib/cron/mod.rs`
- `crates/lx-eval/src/stdlib/diag/mod.rs`
- `crates/lx-eval/src/stdlib/env.rs`
- `crates/lx-eval/src/stdlib/events.rs`
- `crates/lx-eval/src/stdlib/fs.rs`
- `crates/lx-eval/src/stdlib/http.rs`
- `crates/lx-eval/src/stdlib/introspect.rs`
- `crates/lx-eval/src/stdlib/math.rs`
- `crates/lx-eval/src/stdlib/md/mod.rs`
- `crates/lx-eval/src/stdlib/sandbox/mod.rs`
- `crates/lx-eval/src/stdlib/schema.rs`
- `crates/lx-eval/src/stdlib/store/store_dispatch.rs`
- `crates/lx-eval/src/stdlib/stream.rs`
- `crates/lx-eval/src/stdlib/test_mod/mod.rs`
- `crates/lx-eval/src/stdlib/time.rs`
- `crates/lx-eval/src/stdlib/trait_ops.rs`
- `crates/lx-eval/src/interpreter/ambient.rs`

Do not leave mixed spellings behind. After this step, every sync builtin that only borrows the context should use `&dyn BuiltinCtx`.

### Step 4: Preserve the intentional `Arc` ownership boundaries

Do not “fix” the following legitimate `Arc`-based APIs:

- `call_value(...)` in `crates/lx-eval/src/builtins/call.rs`
- async builtin function aliases in `crates/lx-value/src/value/func.rs`
- `Env::child(self: &Arc<Self>)` in `crates/lx-value/src/env.rs`
- wrappers or helpers that must `Arc::clone` into stored state, spawned tasks, or returned values

If a signature needs an `Arc` only because one branch clones it into async work, keep that signature as `Arc`-aware and document it with a short comment only if the reason is not obvious from the code.

### Step 5: Re-run the signature scan and leave only justified exceptions

After the refactor, re-run the exact scan:

```text
rg -n '^\\s*(pub\\s+)?(async\\s+)?fn .*&Arc<|^\\s*fn .*&Arc<' crates -g '*.rs'
```

Any remaining `&Arc<_>` signature must satisfy one of these conditions:

- it clones the `Arc` to take shared ownership
- it stores the `Arc` beyond the current call
- it belongs to an async ABI that needs owned shared state

If a remaining hit does not meet one of those conditions, convert it in this unit instead of leaving it for later.

## Verification

1. Run `just test`.
2. Run `just rust-diagnose`.
3. Run `rg -n 'fn event_stream\\(&self\\) -> &Arc<EventStream>|SyncBuiltinFn = fn\\(&\\[LxVal\\], SourceSpan, &Arc<dyn crate::BuiltinCtx>\\)' crates -g '*.rs'`.
4. Run `rg -n '^\\s*(pub\\s+)?(async\\s+)?fn .*&Arc<|^\\s*fn .*&Arc<' crates -g '*.rs'`.
5. Confirm every remaining `&Arc<_>` signature is one of the explicit ownership exceptions for this unit.

