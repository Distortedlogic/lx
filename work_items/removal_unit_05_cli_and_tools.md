---
unit: 5
title: CLI Layer Removal + [tools] Addition + Tests
scope: lx-cli (plugin.rs, main.rs, manifest.rs, run.rs, tests/plugin.rs), lx-eval (runtime/mod.rs, interpreter/default_tools.rs), lx/src/lib.rs, workspace Cargo.toml, test fixtures, program files
depends_on: [4]
---

## File: `/home/entropybender/repos/lx/crates/lx-cli/src/plugin.rs`
### Change: DELETE entire file (244 lines)

---

## File: `/home/entropybender/repos/lx/crates/lx-cli/tests/plugin.rs`
### Change: DELETE entire file (213 lines)

---

## File: `/home/entropybender/repos/lx/tests/wasm_plugin.lx`
### Change: DELETE entire file (5 lines)

---

## File: `/home/entropybender/repos/lx/tests/fixtures/plugins/test_upper/`
### Change: DELETE entire directory (contains `plugin.toml` and `test_upper.wasm`)

---

## File: `/home/entropybender/repos/lx/Cargo.toml` (workspace root)
### Current (line 41):
```
extism = { version = "1.20.0" }
```
### Change:
Delete line 41 entirely.

---

## File: `/home/entropybender/repos/lx/crates/lx-cli/src/main.rs`
### Current (line 10):
```rust
mod plugin;
```
### Change:
Delete line 10 (`mod plugin;`).

### Current (lines 87-99):
```rust
  Plugin {
    #[command(subcommand)]
    action: PluginAction,
  },
}

#[derive(Subcommand)]
enum PluginAction {
  Install { path: PathBuf },
  List,
  Remove { name: String },
  New { name: String },
}
```
### Change:
Delete the `Plugin` variant from the `Command` enum (lines 87-90) and delete the entire `PluginAction` enum (lines 93-99).

### Current (lines 135-140):
```rust
    Command::Plugin { action } => match action {
      PluginAction::Install { path } => plugin::install(&path),
      PluginAction::List => plugin::list(),
      PluginAction::Remove { name } => plugin::remove(&name),
      PluginAction::New { name } => plugin::new_plugin(&name),
    },
```
### Change:
Delete lines 135-140 entirely.

### Current (line 17):
```rust
use std::path::{Path, PathBuf};
```
### Change:
`PathBuf` is no longer used after removing `PluginAction`. Change to:
```rust
use std::path::Path;
```

### Current (lines 175-180):
```rust
  let ws_members = manifest::try_load_workspace_members();
  let dep_dirs = manifest::try_load_dep_dirs_no_dev();
  let mut ctx_val = if std::io::stdin().is_terminal() { RuntimeCtx { ..RuntimeCtx::default() } } else { RuntimeCtx::default() };
  ctx_val.workspace_members = ws_members;
  ctx_val.dep_dirs = dep_dirs;
  apply_manifest_backends(&mut ctx_val, path);
```
### Change:
Add `apply_manifest_tools` call:
```rust
  let ws_members = manifest::try_load_workspace_members();
  let dep_dirs = manifest::try_load_dep_dirs_no_dev();
  let mut ctx_val = if std::io::stdin().is_terminal() { RuntimeCtx { ..RuntimeCtx::default() } } else { RuntimeCtx::default() };
  ctx_val.workspace_members = ws_members;
  ctx_val.dep_dirs = dep_dirs;
  apply_manifest_tools(&mut ctx_val, path);
  apply_manifest_backends(&mut ctx_val, path);
```

### Addition: add `apply_manifest_tools` function before `apply_manifest_backends`:
```rust
fn apply_manifest_tools(ctx: &mut RuntimeCtx, file_path: &str) {
  let file_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
  let Some(root) = manifest::find_manifest_root(file_dir) else {
    return;
  };
  let Ok(m) = manifest::load_manifest(&root) else {
    return;
  };
  let Some(tools) = m.tools else {
    return;
  };
  for (name, spec) in tools {
    let decl = match spec {
      manifest::ToolSpec::Lx { path } => {
        lx::prelude::ToolDecl::Lx { path: root.join(path) }
      },
      manifest::ToolSpec::Mcp { command } => {
        lx::prelude::ToolDecl::Mcp { command }
      },
    };
    ctx.tools.insert(name, decl);
  }
}
```

---

## File: `/home/entropybender/repos/lx/tests/keywords.lx`
### Current (lines 48-56):
```lx
-- CLI (deprecated, use `use tool` instead)
CLI TestCli = {
  command: "echo"
  tool_defs: [{name: "hello", subcommand: "hello"}]
}
c = TestCli {}
assert c.command == "echo"
assert (c.tool_defs | len == 1)
assert (methods_of c | any? (== "run"))
```
### Change:
Delete lines 48-56 entirely.

### Current (lines 68-76):
```lx
-- MCP (deprecated, use `use tool` instead)
MCP TestServer = {
  command: "echo"
  args: ["test"]
}
srv = TestServer {}
assert srv.command == "echo"
assert srv.args == ["test"]
assert (methods_of srv | any? (== "run"))
```
### Change:
Delete lines 68-76 entirely.

### Current (line 2):
```lx
-- verifies Agent, Tool, Store, Guard, CLI, HTTP, MCP, export prefix, composition, equivalence
```
### Change:
```lx
-- verifies Agent, Tool, Store, Guard, HTTP, export prefix, composition, equivalence
```

---

## File: `/home/entropybender/repos/lx/programs/brain/tools.lx`

This file uses the `MCP` keyword (line 14: `MCP CognitiveTools = { ... }`). After Unit 1 removes the `MCP` keyword, this file will fail to parse.

### Change:
Delete the `MCP CognitiveTools` block (lines 14-24). The tools it declares (Read, Write, Edit, Glob, Grep, Bash, Agent, WebSearch, WebFetch) are the default tools already loaded by `load_default_tools()` — this block is redundant. The rest of the file (tool_costs, exported functions) does not reference `CognitiveTools` and continues to work.

Result: lines 14-24 become deleted. The `tool_costs` map on line 26 (which becomes line 14) continues to work since it uses string keys, not references to the deleted MCP block.

Update the header comment on line 2 to remove MCP reference:
### Current (line 2):
```lx
-- Source: bridges cognitive intent to concrete actions via MCP/shell/AI
```
### Change:
```lx
-- Source: bridges cognitive intent to concrete actions via shell/AI tools
```

---

## File: `/home/entropybender/repos/lx/pkg/git/git.lx`

This file uses the `CLI` keyword (lines 4 and 20: `CLI +Git = { ... }`, `CLI +Gh = { ... }`). After Unit 1 removes the `CLI` keyword, this file will fail to parse.

### Change:
Delete the two `CLI` blocks (lines 4-18 and lines 20-30). The exported functions below (lines 32-60: `+status`, `+branch`, `+root`, `+log`, `+diff`, `+add`, `+commit`, `+push`, `+pull`) do not reference the `Git` or `Gh` bindings — they use direct shell commands (`$git ...`, `$sh -c "git ..."`). No other changes needed.

Result: file becomes lines 1-3 (header comments) followed by the exported functions (currently lines 32-60).

---

## File: `/home/entropybender/repos/lx/crates/lx-cli/src/manifest.rs`
### Current (lines 8-17):
```rust
#[derive(Deserialize)]
pub struct RootManifest {
  pub workspace: Option<WorkspaceSection>,
  pub package: Option<PackageSection>,
  pub test: Option<TestSection>,
  pub backends: Option<BackendsSection>,
  pub stream: Option<StreamSection>,
  pub dependencies: Option<HashMap<String, DepSpec>>,
  #[serde(rename = "deps")]
  pub deps_table: Option<DepsTable>,
}
```
### Change:
Add `tools` field:
```rust
#[derive(Deserialize)]
pub struct RootManifest {
  pub workspace: Option<WorkspaceSection>,
  pub package: Option<PackageSection>,
  pub test: Option<TestSection>,
  pub backends: Option<BackendsSection>,
  pub stream: Option<StreamSection>,
  pub dependencies: Option<HashMap<String, DepSpec>>,
  #[serde(rename = "deps")]
  pub deps_table: Option<DepsTable>,
  pub tools: Option<HashMap<String, ToolSpec>>,
}
```

### Addition: add `ToolSpec` enum after `DepSpec` enum:
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ToolSpec {
  Lx { path: String },
  Mcp { command: String },
}
```

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/runtime/mod.rs`
### Current (lines 21-41, RuntimeCtx struct):
### Change:
Add `ToolDecl` enum before the struct:
```rust
#[derive(Debug, Clone)]
pub enum ToolDecl {
  Lx { path: PathBuf },
  Mcp { command: String },
}
```

Add `tools` field to `RuntimeCtx` after `dep_dirs`:
```rust
  pub tools: HashMap<String, ToolDecl>,
```
`HashMap` implements `Default`, so `SmartDefault` fills it automatically.

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/stdlib/sandbox/sandbox_scope.rs`
### Current (lines 21-34, explicit RuntimeCtx construction):
### Change:
Add `tools: rtx.tools.clone(),` after the `dep_dirs` line:
```rust
    dep_dirs: rtx.dep_dirs.clone(),
    tools: rtx.tools.clone(),
    tokio_runtime: rtx.tokio_runtime.clone(),
```

---

## File: `/home/entropybender/repos/lx/crates/lx-eval/src/interpreter/default_tools.rs`

### Change:
Add `load_declared_tools()` method to the `impl Interpreter` block, after `load_default_tools()`. Add required imports.

Replace entire file with:
```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use indexmap::IndexMap;
use lx_desugar::folder::desugar;
use lx_parser::parser::parse;
use lx_span::source::FileId;
use lx_span::sym::intern;
use lx_value::{EvalSignal, LxError, LxVal, mk_dyn_async};
use miette::SourceSpan;

use crate::runtime::ToolDecl;

use super::Interpreter;

const DEFAULT_TOOL_SOURCES: &[&str] =
  &["tools/bash", "tools/read", "tools/write", "tools/edit", "tools/glob", "tools/grep", "tools/web_search", "tools/web_fetch"];

impl Interpreter {
  pub async fn load_default_tools(&mut self) -> Result<(), LxError> {
    let saved_arena = Arc::clone(&self.arena);
    for &module_name in DEFAULT_TOOL_SOURCES {
      let Some(source) = crate::stdlib::lx_std_module_source(module_name) else { continue };
      let span = SourceSpan::from(0..0);
      let (tokens, comments) = lx_parser::lexer::lex(source).map_err(|e| LxError::runtime(format!("std/{module_name}: {e}"), span))?;
      let result = parse(tokens, FileId::new(0), comments, source);
      let surface = result.program.ok_or_else(|| LxError::runtime(format!("std/{module_name}: parse error"), span))?;
      let program = desugar(surface);
      self.arena = Arc::new(program.arena.clone());
      let stmts = program.stmts.clone();
      for sid in &stmts {
        self.eval_stmt(*sid).await.map_err(|e| match e {
          EvalSignal::Error(e) => e,
          EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
          EvalSignal::AgentStop => LxError::runtime("agent stopped", span),
        })?;
      }
    }
    self.arena = saved_arena;
    Ok(())
  }

  pub async fn load_declared_tools(&mut self) -> Result<(), LxError> {
    let span = SourceSpan::from(0..0);
    let tools: Vec<(String, ToolDecl)> = self.ctx.tools.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    for (name, decl) in tools {
      match decl {
        ToolDecl::Lx { path } => {
          let resolved = if path.is_absolute() {
            path
          } else {
            self.source_dir.as_ref().map(|d| d.join(&path)).unwrap_or(path)
          };
          let exports = self.load_module(&resolved, span).await?;
          let record = LxVal::record(exports.bindings);
          let env = self.env.child();
          env.bind(intern(&name), record);
          self.env = Arc::new(env);
        },
        ToolDecl::Mcp { command } => {
          let tm = crate::tool_module::ToolModule::new(&command, &name)
            .await
            .map_err(|e| LxError::runtime(format!("tool '{name}': {e}"), span))?;
          let tm_arc = Arc::new(tm);
          self.tool_modules.push(Arc::clone(&tm_arc));
          let val = self.build_mcp_tool_record(&name, &tm_arc);
          let env = self.env.child();
          env.bind(intern(&name), val);
          self.env = Arc::new(env);
        },
      }
    }
    Ok(())
  }

  fn build_mcp_tool_record(&self, alias: &str, tm: &Arc<crate::tool_module::ToolModule>) -> LxVal {
    let agent_name: Arc<str> = Arc::from(self.agent_name.as_deref().unwrap_or("main"));
    let call_counter = Arc::new(AtomicU64::new(1));
    let event_stream = Arc::clone(&self.ctx.event_stream);
    let module_name: Arc<str> = Arc::from(alias);

    let mut fields = IndexMap::new();
    fields.insert(intern("command"), LxVal::str(&tm.command));
    fields.insert(intern("alias"), LxVal::str(&tm.alias));

    let tm_call = Arc::clone(tm);
    let agent_call = Arc::clone(&agent_name);
    let counter_call = Arc::clone(&call_counter);
    let es_call = Arc::clone(&event_stream);
    let mod_call = Arc::clone(&module_name);

    let call_tool_fn = mk_dyn_async(
      "mcp_tool.call",
      2,
      Arc::new(move |args: Vec<LxVal>, call_span: SourceSpan, _ctx: Arc<dyn lx_value::BuiltinCtx>| {
        let tm = Arc::clone(&tm_call);
        let agent = Arc::clone(&agent_call);
        let counter = Arc::clone(&counter_call);
        let es = Arc::clone(&es_call);
        let module = Arc::clone(&mod_call);
        Box::pin(async move {
          let method = args[0].as_str().ok_or_else(|| LxError::runtime("mcp_tool.call: first arg must be method name (Str)", call_span))?;
          let arg = args.get(1).cloned().unwrap_or(LxVal::Unit);
          let call_id = counter.fetch_add(1, Ordering::Relaxed);

          let mut call_fields = IndexMap::new();
          call_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
          call_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
          call_fields.insert(intern("method"), LxVal::str(&method));
          call_fields.insert(intern("args"), arg.clone());
          es.xadd("tool/call", &agent, None, call_fields);

          let result = tm.call_tool(&method, arg, &es, &agent).await;

          match result {
            Ok(val) => {
              let mut result_fields = IndexMap::new();
              result_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
              result_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
              result_fields.insert(intern("method"), LxVal::str(&method));
              result_fields.insert(intern("result"), val.clone());
              es.xadd("tool/result", &agent, None, result_fields);
              Ok(val)
            },
            Err(e) => {
              let err_msg = e.to_string();
              let mut error_fields = IndexMap::new();
              error_fields.insert(intern("call_id"), LxVal::int(call_id as i64));
              error_fields.insert(intern("tool"), LxVal::str(module.as_ref()));
              error_fields.insert(intern("method"), LxVal::str(&method));
              error_fields.insert(intern("error"), LxVal::str(&err_msg));
              es.xadd("tool/error", &agent, None, error_fields);
              Err(LxError::runtime(format!("mcp tool '{module}' method '{method}': {err_msg}"), call_span))
            },
          }
        })
      }),
    );
    fields.insert(intern("call"), call_tool_fn);

    LxVal::record(fields)
  }
}
```

The MCP `ToolDecl::Mcp` branch:
- Spawns `ToolModule::new()` and pushes onto `self.tool_modules` for shutdown (field retained from Unit 4).
- Calls `build_mcp_tool_record` which creates a `LxVal::Record` with `command`, `alias` fields and a `call` builtin async function.
- The `call` function takes `(method_name: Str, args: Any)` and delegates to `tm.call_tool(...)`.
- This avoids `LxVal::ToolModule` (removed in Unit 2) by wrapping the MCP interface as a Record of closures, matching the pattern used by `build_lx_tool_module` in `lx_tool_module.rs`.

---

## File: `/home/entropybender/repos/lx/crates/lx-cli/src/run.rs`
### Current (line 44):
```rust
    interp.load_default_tools().await.map_err(|e| vec![e])?;
```
### Change:
Add `load_declared_tools()` call after:
```rust
    interp.load_default_tools().await.map_err(|e| vec![e])?;
    interp.load_declared_tools().await.map_err(|e| vec![e])?;
```

---

## File: `/home/entropybender/repos/lx/crates/lx/src/lib.rs`
### Current (line 15):
```rust
  pub use lx_eval::runtime::RuntimeCtx;
```
### Change:
Add `ToolDecl` re-export:
```rust
  pub use lx_eval::runtime::RuntimeCtx;
  pub use lx_eval::runtime::ToolDecl;
```

---

## Deletion checklist

| Item | Type | Path |
|------|------|------|
| WASM plugin CLI module | file | `crates/lx-cli/src/plugin.rs` |
| WASM plugin CLI tests | file | `crates/lx-cli/tests/plugin.rs` |
| WASM plugin test fixture | directory | `tests/fixtures/plugins/test_upper/` |
| WASM plugin test script | file | `tests/wasm_plugin.lx` |
| extism workspace dep | line | `Cargo.toml` line 41 |
| `mod plugin` | line | `crates/lx-cli/src/main.rs` line 10 |
| `Plugin` variant + `PluginAction` enum | lines | `crates/lx-cli/src/main.rs` lines 87-99 |
| `Command::Plugin` match arm | lines | `crates/lx-cli/src/main.rs` lines 135-140 |
| CLI test block in keywords.lx | lines | `tests/keywords.lx` lines 48-56 |
| MCP test block in keywords.lx | lines | `tests/keywords.lx` lines 68-76 |
| MCP block in brain/tools.lx | lines | `programs/brain/tools.lx` lines 14-24 |
| CLI blocks in pkg/git/git.lx | lines | `pkg/git/git.lx` lines 4-30 |
