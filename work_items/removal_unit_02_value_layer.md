---
unit: 2
title: Value Layer Removal
scope: lx-value
depends_on: none
---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/value/mod.rs`

### Current (lines 119-120):
```rust
  #[strum(serialize = "ToolModule")]
  ToolModule(Arc<dyn crate::ToolModuleHandle>),
```

### Change:
Delete lines 119-120 entirely. No replacement.

---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/value/display.rs`

### Current (line 83):
```rust
      LxVal::ToolModule(tm) => write!(f, "<ToolModule:{}>", tm.alias()),
```

### Change:
Delete line 83 entirely. No replacement.

---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/value/impls.rs`

### Current (line 130):
```rust
      LxVal::Func(_) | LxVal::MultiFunc(_) | LxVal::BuiltinFunc(_) | LxVal::TaggedCtor { .. } | LxVal::ToolModule(_) => {},
```

### Change:
Replace with:
```rust
      LxVal::Func(_) | LxVal::MultiFunc(_) | LxVal::BuiltinFunc(_) | LxVal::TaggedCtor { .. } => {},
```
Remove ` | LxVal::ToolModule(_)` from the pattern.

---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/tool_module_handle.rs`

### Change:
Delete this entire file.

---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/lib.rs`

### Current (line 7):
```rust
mod tool_module_handle;
```

### Change:
Delete line 7 entirely.

### Current (line 16):
```rust
pub use tool_module_handle::ToolModuleHandle;
```

### Change:
Delete line 16 entirely.

---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/value/serde_impl.rs`

### Change:
No explicit `ToolModule` arm exists. The `Serialize` impl has a catch-all `_ =>` arm on line 62 that currently covers `ToolModule` (among others like `Func`, `MultiFunc`, `BuiltinFunc`, `TaggedCtor`, `TraitUnion`, `Trait`, `Class`, `Channel`). No change needed in this file.

---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/value/methods.rs`

### Change:
No `ToolModule` references. No change needed.

---

## File: `/home/entropybender/repos/lx/crates/lx-value/src/value/func.rs`

### Change:
No `ToolModule` references. No change needed.

---

## Downstream impacts

Removing `LxVal::ToolModule` and `lx_value::ToolModuleHandle` will cause compile errors in the following locations in `lx-eval`:

### 1. `crates/lx-eval/src/tool_module.rs` (lines 90-112)
```rust
impl lx_value::ToolModuleHandle for ToolModule {
```
The `impl lx_value::ToolModuleHandle for ToolModule` block implements the now-deleted trait. This entire `impl` block (lines 90-112) must be removed. The inherent methods on `ToolModule` (lines 25-88) and the struct itself (lines 12-17) remain valid — they do not depend on the trait.

### 2. `crates/lx-eval/src/interpreter/modules.rs` (line 38)
```rust
      let val = LxVal::ToolModule(tm_arc);
```
This constructs the deleted variant. The `use tool` statement handling (lines 25-42) wraps a `ToolModule` in `Arc` and stores it as `LxVal::ToolModule`. This code path must be redesigned — the `ToolModule` can no longer be stored as an `LxVal` variant. The tool module's methods must be exposed differently (e.g., as bound builtin functions in the env, or as a Record of closures).

### 3. `crates/lx-eval/src/interpreter/apply_helpers.rs` (lines 38-46)
```rust
        LxVal::ToolModule(tm) => {
          let method_name = name.as_str().to_string();
          let tm = Arc::clone(tm);
          Ok(LxVal::BuiltinFunc(lx_value::BuiltinFunc {
            name: "tool.call",
            arity: 3,
            kind: lx_value::BuiltinKind::Async(bi_tool_dispatch),
            applied: vec![LxVal::ToolModule(tm), LxVal::str(method_name)],
          }))
        },
```
This field-access match arm dispatches `.method_name` on a `ToolModule` value. References `LxVal::ToolModule` both in the match pattern (line 38) and in the `applied` vec (line 45). Must be removed.

### 4. `crates/lx-eval/src/interpreter/apply_helpers.rs` (lines 112-121)
```rust
fn bi_tool_dispatch(args: Vec<LxVal>, span: SourceSpan, ctx: Arc<dyn lx_value::BuiltinCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>> {
  Box::pin(async move {
    let LxVal::ToolModule(tm) = &args[0] else {
      return Err(LxError::runtime("tool.call: invalid tool module", span));
    };
    let method = args[1].as_str().ok_or_else(|| LxError::runtime("tool.call: invalid method name", span))?;
    let arg = args[2].clone();
    tm.call_tool(method, arg, ctx.event_stream(), "main").await
  })
}
```
The `bi_tool_dispatch` function destructures `LxVal::ToolModule` and calls the trait method `call_tool`. Both the variant and the trait are being removed. This entire function must be removed or redesigned.

### 5. `crates/lx-eval/src/interpreter/mod.rs` (line 43)
```rust
  pub(crate) tool_modules: Vec<Arc<crate::tool_module::ToolModule>>,
```
This field stores `Arc<ToolModule>` for shutdown purposes. The field itself does not reference `LxVal::ToolModule` or the trait, so it compiles fine. However, once the `ToolModuleHandle` trait impl is removed from `tool_module.rs`, the `tm_arc` in `modules.rs` can no longer be cast to `Arc<dyn ToolModuleHandle>`. The `tool_modules` vec and shutdown loop (mod.rs lines 140-142) remain valid since they call inherent `shutdown()` directly.

### Summary of downstream compile errors (4 hard errors):

| File | Line(s) | Error |
|------|---------|-------|
| `lx-eval/src/tool_module.rs` | 90 | `ToolModuleHandle` trait not found |
| `lx-eval/src/interpreter/modules.rs` | 38 | `LxVal::ToolModule` variant not found |
| `lx-eval/src/interpreter/apply_helpers.rs` | 38, 45 | `LxVal::ToolModule` variant not found (2 occurrences) |
| `lx-eval/src/interpreter/apply_helpers.rs` | 114 | `LxVal::ToolModule` variant not found |
