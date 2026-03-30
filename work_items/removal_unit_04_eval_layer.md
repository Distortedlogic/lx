---
unit: 4
title: Eval Layer Removal
scope: lx-eval
depends_on: [1, 2, 3]
---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/interpreter/modules.rs`

### Current (lines 1-16, imports):
```rust
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use indexmap::IndexMap;

use crate::stdlib::wasm::load_plugin;
use lx_ast::ast::{BindTarget, Core, Program, Stmt, StmtTypeDef, UseKind, UseStmt};
use lx_desugar::folder::desugar;
use lx_parser::parser::parse;
use lx_span::source::FileId;
use lx_value::LxError;
use lx_value::LxVal;
use miette::SourceSpan;
```
### Change:
Remove line 1 (`use std::env;`) and line 8 (`use crate::stdlib::wasm::load_plugin;`). These are only used by `find_plugin_dir` and `load_wasm_plugin`, both of which are being deleted.

---

### Current (lines 25-43, `UseKind::Tool` branch in `eval_use`):
```rust
    if let UseKind::Tool { command, alias } = &use_stmt.kind {
      let cmd_str = command.as_str();
      if cmd_str.ends_with(".lx") {
        let val = self.build_lx_tool_module(cmd_str, span).await?;
        let env = self.env.child();
        env.bind(*alias, val);
        self.env = Arc::new(env);
        return Ok(());
      }
      let alias_str = alias.as_str();
      let tm = crate::tool_module::ToolModule::new(cmd_str, alias_str).await.map_err(|e| LxError::runtime(e, span))?;
      let tm_arc = Arc::new(tm);
      self.tool_modules.push(Arc::clone(&tm_arc));
      let val = LxVal::ToolModule(tm_arc);
      let env = self.env.child();
      env.bind(*alias, val);
      self.env = Arc::new(env);
      return Ok(());
    }
```
### Change:
Delete this entire block (lines 25-43). The `use tool` evaluation path is being removed. The `UseKind::Tool` variant will be removed from the AST in Unit 3 (lx-ast/lx-parser), so this branch becomes unreachable. Also remove line 85 (`UseKind::Tool { .. } => unreachable!(),`) from the `match &use_stmt.kind` below, since the variant won't exist.

---

### Current (lines 52-53, `wasm/` prefix dispatch in `eval_use`):
```rust
    } else if let Some(plugin_name) = str_joined.strip_prefix("wasm/") {
      self.load_wasm_plugin(plugin_name, span)?
```
### Change:
Delete these 2 lines. The `wasm/` module import path is being removed.

---

### Current (lines 91-120, `load_wasm_plugin` and `find_plugin_dir` methods):
```rust
  fn load_wasm_plugin(&self, name: &str, span: SourceSpan) -> Result<ModuleExports, LxError> {
    let cache_key = PathBuf::from(format!("__wasm_{name}"));
    {
      let cache = self.module_cache.lock();
      if let Some(exports) = cache.get(&cache_key) {
        return Ok(exports.clone());
      }
    }
    let plugin_dir = self.find_plugin_dir(name);
    let dir = plugin_dir.ok_or_else(|| LxError::runtime(format!("wasm plugin '{name}' not found"), span))?;
    let exports = load_plugin(name, &dir, span)?;
    self.module_cache.lock().insert(cache_key, exports.clone());
    Ok(exports)
  }

  fn find_plugin_dir(&self, name: &str) -> Option<PathBuf> {
    if let Some(ref source_dir) = self.source_dir {
      let local = source_dir.join(".lx").join("plugins").join(name);
      if local.join(crate::PLUGIN_MANIFEST).exists() {
        return Some(local);
      }
    }
    if let Some(home) = env::var_os("HOME") {
      let global = PathBuf::from(home).join(".lx").join("plugins").join(name);
      if global.join(crate::PLUGIN_MANIFEST).exists() {
        return Some(global);
      }
    }
    None
  }
```
### Change:
Delete both methods entirely (lines 91-120).

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/interpreter/apply_helpers.rs`

### Current (lines 1-4, imports):
```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

```
### Change:
Delete lines 1-2 (`use std::future::Future;` and `use std::pin::Pin;`). These are only used by `bi_tool_dispatch` which is being removed.

---

### Current (lines 38-47, `LxVal::ToolModule(tm)` match arm in `eval_field_access`):
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
### Change:
Delete this entire match arm (lines 38-47). The `LxVal::ToolModule` variant is being removed from `LxVal` in Unit 2.

---

### Current (lines 112-121, `bi_tool_dispatch` function):
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
### Change:
Delete the entire `bi_tool_dispatch` function (lines 112-121).

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/interpreter/lx_tool_module.rs`

### Current: entire file (104 lines)
### Change:
Delete this file entirely. It contains `build_lx_tool_module` and `resolve_lx_tool_path`, both exclusively used by the `UseKind::Tool` branch that is being removed.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/interpreter/mod.rs`

### Current (line 12):
```rust
mod lx_tool_module;
```
### Change:
Delete this line.

---

### Current (line 43):
```rust
  pub(crate) tool_modules: Vec<Arc<crate::tool_module::ToolModule>>,
```
### Change:
**Keep this field.** It will be reused by Unit 5's `load_declared_tools` to store MCP tool modules for shutdown.

---

### Current (line 71, in `new()`):
```rust
      tool_modules: vec![],
```
### Change:
**Keep this line.** The field is retained.

---

### Current (line 88, in `with_env()`):
```rust
      tool_modules: vec![],
```
### Change:
**Keep this line.** The field is retained.

---

### Current (lines 140-142, shutdown loop in `exec()`):
```rust
    for tm in &self.tool_modules {
      tm.shutdown().await;
    }
```
### Change:
**Keep these 3 lines.** The shutdown loop is retained for MCP tool modules loaded by Unit 5's `[tools]` config.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/interpreter/eval.rs`

### Current (lines 117, 156, 240 — `tool_modules: vec![]` in `eval_par`, `eval_sel`, `eval_timeout`):
### Change:
**Keep all three lines.** The `tool_modules` field is retained.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/interpreter/apply.rs`

### Current (line 236, in `eval_func` Interpreter construction):
```rust
            tool_modules: vec![],
```
### Change:
**Keep this line.** The `tool_modules` field is retained.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/tool_module.rs`

### Current (lines 90-112, `impl ToolModuleHandle for ToolModule` block):
```rust
impl lx_value::ToolModuleHandle for ToolModule {
  fn call_tool<'a>(
    &'a self,
    method: &'a str,
    args: LxVal,
    event_stream: &'a EventStream,
    agent_name: &'a str,
  ) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>> + 'a>> {
    Box::pin(self.call_tool(method, args, event_stream, agent_name))
  }

  fn shutdown(&self) -> Pin<Box<dyn Future<Output = ()> + '_>> {
    Box::pin(self.shutdown())
  }

  fn command(&self) -> &str {
    &self.command
  }

  fn alias(&self) -> &str {
    &self.alias
  }
}
```
### Change:
Delete this entire impl block (lines 90-112). The `ToolModuleHandle` trait is being removed in Unit 2 (lx-value). Also remove the now-unused imports on lines 1-2:
```rust
use std::future::Future;
use std::pin::Pin;
```
And remove the unused import on line 8:
```rust
use lx_value::{EventStream, LxError, LxVal};
```
Replace with:
```rust
use lx_value::{LxError, LxVal};
```
(The `EventStream` type is still used in the `call_tool` inherent method signature on line 31, so actually keep it. Check: line 31 reads `pub async fn call_tool(&self, method: &str, args: LxVal, event_stream: &EventStream, agent_name: &str)` -- yes, `EventStream` is still needed.)

Correction: Keep the `EventStream` import. Only remove `use std::future::Future;` and `use std::pin::Pin;` (lines 1-2). These are only used by the `impl ToolModuleHandle` block's return types.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/stdlib/wasm.rs`

### Current: entire file (145 lines)
### Change:
Delete this file entirely. It contains the WASM plugin loading system (`load_plugin`, `call_plugin_fn`, `PLUGINS` static, and all extism integration).

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/stdlib/wasm_marshal.rs`

### Current: entire file (225 lines)
### Change:
Delete this file entirely. It contains JSON/LxVal marshaling exclusively for the WASM plugin system.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/stdlib/mod.rs`

### Current (lines 31-32):
```rust
pub(crate) mod wasm;
pub(crate) mod wasm_marshal;
```
### Change:
Delete both lines.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/lib.rs`

### Current (line 1):
```rust
pub use lx_span::{LX_MANIFEST, PLUGIN_MANIFEST};
```
### Change:
Remove `PLUGIN_MANIFEST` from the re-export. Result:
```rust
pub use lx_span::LX_MANIFEST;
```
No external consumers reference `lx_eval::PLUGIN_MANIFEST` (verified by grep). The `crate::PLUGIN_MANIFEST` usages within lx-eval are all in `modules.rs` (`find_plugin_dir`) and `wasm.rs`, both of which are being deleted.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/Cargo.toml`

### Current (line 17):
```toml
extism.workspace = true
```
### Change:
Delete this line. All extism usage is in `stdlib/wasm.rs` which is being deleted.

---

## Files with no changes needed

- **`exec_stmt.rs`** -- No references to `tool_modules`, `ToolModule`, `wasm`, or `extism`. No changes needed.
- **`traits.rs`** -- No references to any of the removed symbols. No changes needed.
- **`messaging.rs`** -- No references. No changes needed.
- **`builtins/`** -- No references. No changes needed.

---

## Downstream impacts

- **`LxVal::ToolModule` variant removal (Unit 2)** must land before this unit. Lines in `apply_helpers.rs` (lines 38-47, 114) and `modules.rs` (line 38) reference `LxVal::ToolModule`. If Unit 2 removes the variant first, these lines will fail to compile, which is the intended sequencing.
- **`UseKind::Tool` variant removal (Unit 3)** must land before this unit. Lines in `modules.rs` (lines 25, 85) reference `UseKind::Tool`. If Unit 3 removes the variant first, these lines will fail to compile.
- **`ToolModuleHandle` trait removal (Unit 2)** must land before this unit. Line 90 of `tool_module.rs` implements the trait. If Unit 2 removes the trait first, the impl block will fail to compile.
- **`tool_module.rs` remains as `pub mod tool_module`** in `lib.rs` (line 9). The `ToolModule` struct, `new()`, `call_tool()`, and `shutdown()` inherent methods are preserved for reuse by `[tools]` config loading in Unit 5.
- **No downstream crates** import `lx_eval::PLUGIN_MANIFEST` or `lx_eval::tool_module::ToolModule` directly (verified by grep). Removing the `PLUGIN_MANIFEST` re-export and the `ToolModuleHandle` impl are safe.
- **Workspace `Cargo.toml`** may still list `extism` in `[workspace.dependencies]`. That is fine; it only becomes dead weight. It can be cleaned up in a final sweep if no other crate uses it.
