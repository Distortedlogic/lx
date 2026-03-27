# Code Cleanup — Imports, Params, Serde, Literals

---

## Task 1: Inline imports in `store_dispatch.rs`

**File:** `crates/lx/src/stdlib/store/store_dispatch.rs`

Replace:

```
use crate::value::{BuiltinFunc, BuiltinKind, LxVal};
```

with:

```
use crate::sym::{Sym, intern};
use crate::value::{BuiltinFunc, BuiltinKind, LxVal, SyncBuiltinFn};
```

Replace:

```
  let method: Option<(&'static str, usize, crate::value::SyncBuiltinFn)> = match name {
```

with:

```
  let method: Option<(&'static str, usize, SyncBuiltinFn)> = match name {
```

Replace:

```
pub fn object_insert(fields: indexmap::IndexMap<crate::sym::Sym, crate::value::LxVal>) -> u64 {
```

with:

```
pub fn object_insert(fields: indexmap::IndexMap<Sym, LxVal>) -> u64 {
```

Replace:

```
pub fn object_get_field(id: u64, field: &str) -> Option<crate::value::LxVal> {
  STORES.get(&id).and_then(|s| s.data.get(&crate::sym::intern(field)).cloned())
```

with:

```
pub fn object_get_field(id: u64, field: &str) -> Option<LxVal> {
  STORES.get(&id).and_then(|s| s.data.get(&intern(field)).cloned())
```

Replace:

```
pub fn object_update_nested(id: u64, path: &[crate::sym::Sym], value: crate::value::LxVal) -> Result<(), String> {
```

with:

```
pub fn object_update_nested(id: u64, path: &[Sym], value: LxVal) -> Result<(), String> {
```

Replace:

```
fn update_nested_record(val: &crate::value::LxVal, path: &[crate::sym::Sym], new_val: crate::value::LxVal) -> Result<crate::value::LxVal, String> {
  let crate::value::LxVal::Record(rec) = val else {
```

with:

```
fn update_nested_record(val: &LxVal, path: &[Sym], new_val: LxVal) -> Result<LxVal, String> {
  let LxVal::Record(rec) = val else {
```

Replace (first occurrence in `update_nested_record`, inside `[field]` arm):

```
      Ok(crate::value::LxVal::record(new_rec))
    },
    [field, rest @ ..] => {
      let inner = rec.get(field).ok_or_else(|| format!("field '{field}' not found"))?;
      let updated = update_nested_record(inner, rest, new_val)?;
      let mut new_rec = rec.as_ref().clone();
      new_rec.insert(*field, updated);
      Ok(crate::value::LxVal::record(new_rec))
```

with:

```
      Ok(LxVal::record(new_rec))
    },
    [field, rest @ ..] => {
      let inner = rec.get(field).ok_or_else(|| format!("field '{field}' not found"))?;
      let updated = update_nested_record(inner, rest, new_val)?;
      let mut new_rec = rec.as_ref().clone();
      new_rec.insert(*field, updated);
      Ok(LxVal::record(new_rec))
```

Replace:

```
  Ok(LxVal::Bool(s.data.contains_key(&crate::sym::intern(key))))
```

with:

```
  Ok(LxVal::Bool(s.data.contains_key(&intern(key))))
```

Replace:

```
  let source_data: indexmap::IndexMap<crate::sym::Sym, LxVal> = match &args[1] {
```

with:

```
  let source_data: indexmap::IndexMap<Sym, LxVal> = match &args[1] {
```

---

## Task 2: Inline imports in `test_invoke.rs`

**File:** `crates/lx/src/stdlib/test_mod/test_invoke.rs`

Replace:

```
use std::sync::Arc;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;
```

with:

```
use std::sync::Arc;

use crate::ast::{BindTarget, Program, Stmt};
use crate::error::{EvalSignal, LxError};
use crate::folder::desugar;
use crate::interpreter::Interpreter;
use crate::lexer::lex;
use crate::parser::parse;
use crate::runtime::RuntimeCtx;
use crate::source::FileId;
use crate::sym::intern;
use crate::value::LxVal;
use miette::SourceSpan;
```

Replace:

```
  let (tokens, comments) = crate::lexer::lex(&source).map_err(|e| LxError::runtime(format!("test.run: lex error in '{flow_path}': {e}"), span))?;
  let result = crate::parser::parse(tokens, crate::source::FileId::new(0), comments, &source);
```

with:

```
  let (tokens, comments) = lex(&source).map_err(|e| LxError::runtime(format!("test.run: lex error in '{flow_path}': {e}"), span))?;
  let result = parse(tokens, FileId::new(0), comments, &source);
```

Replace:

```
  let program = crate::folder::desugar(surface);
  let module_dir = path.parent().map(|p| p.to_path_buf());
  let mut interp = crate::interpreter::Interpreter::new(&source, module_dir, Arc::clone(ctx));
```

with:

```
  let program = desugar(surface);
  let module_dir = path.parent().map(|p| p.to_path_buf());
  let mut interp = Interpreter::new(&source, module_dir, Arc::clone(ctx));
```

Replace:

```
        .get(crate::sym::intern(&entry_name))
```

with:

```
        .get(intern(&entry_name))
```

Replace:

```
      interp.apply_func(entry, input.clone(), span).await.map_err(|e| match e {
        crate::error::EvalSignal::Error(e) => e,
        crate::error::EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
      })
```

with:

```
      interp.apply_func(entry, input.clone(), span).await.map_err(|e| match e {
        EvalSignal::Error(e) => e,
        EvalSignal::Break(_) => LxError::runtime("break outside loop", span),
      })
```

Replace:

```
fn find_flow_entry_name<P>(program: &crate::ast::Program<P>) -> Option<String> {
  use crate::ast::{BindTarget, Stmt};
```

with:

```
fn find_flow_entry_name<P>(program: &Program<P>) -> Option<String> {
```

---

## Task 3: Inline imports in `source.rs`

**File:** `crates/lx/src/source.rs`

Replace:

```
use crate::ast::{ExprId, PatternId, StmtId, TypeExprId};
```

with:

```
use crate::ast::{ExprId, NodeId, PatternId, StmtId, TypeExprId};
```

Replace:

```
pub type CommentMap = std::collections::HashMap<crate::ast::NodeId, Vec<AttachedComment>>;
```

with:

```
pub type CommentMap = std::collections::HashMap<NodeId, Vec<AttachedComment>>;
```

Replace:

```
impl crate::ast::NodeId {
```

with:

```
impl NodeId {
```

Replace:

```
      crate::ast::NodeId::Expr(id) => GlobalNodeId::Expr(GlobalExprId::new(file, id)),
      crate::ast::NodeId::Stmt(id) => GlobalNodeId::Stmt(GlobalStmtId::new(file, id)),
      crate::ast::NodeId::Pattern(id) => GlobalNodeId::Pattern(GlobalPatternId::new(file, id)),
      crate::ast::NodeId::TypeExpr(id) => GlobalNodeId::TypeExpr(GlobalTypeExprId::new(file, id)),
```

with:

```
      NodeId::Expr(id) => GlobalNodeId::Expr(GlobalExprId::new(file, id)),
      NodeId::Stmt(id) => GlobalNodeId::Stmt(GlobalStmtId::new(file, id)),
      NodeId::Pattern(id) => GlobalNodeId::Pattern(GlobalPatternId::new(file, id)),
      NodeId::TypeExpr(id) => GlobalNodeId::TypeExpr(GlobalTypeExprId::new(file, id)),
```

---

## Task 4: Inline imports in `ast/mod.rs`

**File:** `crates/lx/src/ast/mod.rs`

Replace:

```
use crate::sym::Sym;
```

with:

```
use crate::source::{Comment, CommentMap, CommentPlacement, CommentStore, FileId};
use crate::sym::Sym;
```

Replace:

```
  pub comments: crate::source::CommentStore,
  pub comment_map: crate::source::CommentMap,
  pub file: crate::source::FileId,
```

with:

```
  pub comments: CommentStore,
  pub comment_map: CommentMap,
  pub file: FileId,
```

Replace:

```
  pub fn leading_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
    self.attached_comments(node, crate::source::CommentPlacement::Leading)
```

with:

```
  pub fn leading_comments(&self, node: NodeId) -> Vec<&Comment> {
    self.attached_comments(node, CommentPlacement::Leading)
```

Replace:

```
  pub fn trailing_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
    self.attached_comments(node, crate::source::CommentPlacement::Trailing)
```

with:

```
  pub fn trailing_comments(&self, node: NodeId) -> Vec<&Comment> {
    self.attached_comments(node, CommentPlacement::Trailing)
```

Replace:

```
  pub fn dangling_comments(&self, node: NodeId) -> Vec<&crate::source::Comment> {
    self.attached_comments(node, crate::source::CommentPlacement::Dangling)
```

with:

```
  pub fn dangling_comments(&self, node: NodeId) -> Vec<&Comment> {
    self.attached_comments(node, CommentPlacement::Dangling)
```

Replace:

```
  fn attached_comments(&self, node: NodeId, placement: crate::source::CommentPlacement) -> Vec<&crate::source::Comment> {
```

with:

```
  fn attached_comments(&self, node: NodeId, placement: CommentPlacement) -> Vec<&Comment> {
```

---

## Task 5: Inline imports in `lx-desktop` `terminal/view.rs`

**File:** `crates/lx-desktop/src/terminal/view.rs`

Replace:

```
use crate::panes::DesktopPane;
```

with:

```
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::status_bar::StatusBarState;
use crate::panes::DesktopPane;
```

Replace:

```
  let activity_log = use_context::<crate::contexts::activity_log::ActivityLog>();
```

with:

```
  let activity_log = use_context::<ActivityLog>();
```

Replace:

```
            let ctx = use_context::<crate::contexts::status_bar::StatusBarState>();
```

with:

```
            let ctx = use_context::<StatusBarState>();
```

---

## Task 6: `&Arc<Vec<T>>` parameters in `trait_apply.rs`

**File:** `crates/lx/src/interpreter/trait_apply.rs`

Replace:

```
  pub(super) async fn apply_trait_fields(&mut self, name: &str, fields: &Arc<Vec<FieldDef>>, arg: &LxVal, _span: SourceSpan) -> EvalResult<LxVal> {
```

with:

```
  pub(super) async fn apply_trait_fields(&mut self, name: &str, fields: &[FieldDef], arg: &LxVal, _span: SourceSpan) -> EvalResult<LxVal> {
```

Replace:

```
  pub(super) async fn apply_trait_union(&mut self, name: &str, variants: &Arc<Vec<Sym>>, arg: &LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
```

with:

```
  pub(super) async fn apply_trait_union(&mut self, name: &str, variants: &[Sym], arg: &LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
```

Replace:

```
      if self.try_match_variant(&proto_trait.fields, rec, span).is_ok() {
```

with:

```
      if self.try_match_variant(&*proto_trait.fields, rec, span).is_ok() {
```

Replace:

```
  fn try_match_variant(&mut self, fields: &Arc<Vec<FieldDef>>, rec: &Arc<indexmap::IndexMap<Sym, LxVal>>, span: SourceSpan) -> Result<(), LxError> {
```

with:

```
  fn try_match_variant(&mut self, fields: &[FieldDef], rec: &Arc<indexmap::IndexMap<Sym, LxVal>>, span: SourceSpan) -> Result<(), LxError> {
```

**File:** `crates/lx/src/interpreter/apply.rs`

Replace:

```
      LxVal::Trait(ref t) if !t.fields.is_empty() => self.apply_trait_fields(t.name.as_str(), &t.fields, &arg, span).await,
```

with:

```
      LxVal::Trait(ref t) if !t.fields.is_empty() => self.apply_trait_fields(t.name.as_str(), &*t.fields, &arg, span).await,
```

Replace:

```
      LxVal::TraitUnion { name, variants } => Ok(self.apply_trait_union(name.as_str(), &variants, &arg, span).await?),
```

with:

```
      LxVal::TraitUnion { name, variants } => Ok(self.apply_trait_union(name.as_str(), &*variants, &arg, span).await?),
```

---

## Task 7: Remove redundant `From<&String>` impl in `sym.rs`

**File:** `crates/lx/src/sym.rs`

Delete this block:

```rust
impl From<&String> for Sym {
  fn from(s: &String) -> Self {
    intern(s)
  }
}
```

---

## Task 8: Serde attributes in `wasm.rs`

**File:** `crates/lx/src/stdlib/wasm.rs`

Replace:

```rust
#[derive(serde::Deserialize)]
struct SandboxConfig {
  #[serde(default)]
  wasi: Option<bool>,
  #[serde(default)]
  fuel: Option<u64>,
}
```

with:

```rust
#[derive(serde::Deserialize)]
#[serde(default)]
struct SandboxConfig {
  wasi: Option<bool>,
  fuel: Option<u64>,
}
```

---

## Task 9: Extract `"plugin.toml"` constant

### 9a. Add constant to `crates/lx/src/lib.rs`

Add before the first `pub mod` declaration:

```rust
pub const PLUGIN_MANIFEST: &str = "plugin.toml";
```

### 9b. Update `crates/lx/src/interpreter/modules.rs`

Replace:

```
      if local.join("plugin.toml").exists() {
```

with:

```
      if local.join(crate::PLUGIN_MANIFEST).exists() {
```

Replace:

```
      if global.join("plugin.toml").exists() {
```

with:

```
      if global.join(crate::PLUGIN_MANIFEST).exists() {
```

### 9c. Update `crates/lx/src/stdlib/wasm.rs`

Replace:

```
  let toml_path = plugin_dir.join("plugin.toml");
```

with:

```
  let toml_path = plugin_dir.join(crate::PLUGIN_MANIFEST);
```

### 9d. Update `crates/lx-cli/src/plugin.rs`

Replace:

```
  let manifest_path = dir.join("plugin.toml");
```

with:

```
  let manifest_path = dir.join(lx::PLUGIN_MANIFEST);
```

Replace:

```
    (dir.join("plugin.toml"), &plugin_toml),
```

with:

```
    (dir.join(lx::PLUGIN_MANIFEST), &plugin_toml),
```

---

## Task 10: Extract `"lx.toml"` constant

### 10a. Add constant to `crates/lx/src/lib.rs`

Add alongside the `PLUGIN_MANIFEST` constant:

```rust
pub const LX_MANIFEST: &str = "lx.toml";
```

### 10b. Update `crates/lx-cli/src/manifest.rs`

In `find_manifest_root`, replace:

```
    let candidate = dir.join("lx.toml");
    if candidate.exists() {
      return Some(dir);
    }
    if !dir.pop() {
      return None;
    }
```

with:

```
    let candidate = dir.join(lx::LX_MANIFEST);
    if candidate.exists() {
      return Some(dir);
    }
    if !dir.pop() {
      return None;
    }
```

In `load_manifest`, replace:

```
  let manifest_path = root.join("lx.toml");
```

with:

```
  let manifest_path = root.join(lx::LX_MANIFEST);
```

In `find_workspace_root`, replace:

```
    let candidate = dir.join("lx.toml");
    if candidate.exists() {
```

with:

```
    let candidate = dir.join(lx::LX_MANIFEST);
    if candidate.exists() {
```

In `load_workspace`, replace:

```
  let manifest_path = root.join("lx.toml");
  let content = std::fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
  let manifest: RootManifest = toml::from_str(&content).map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
  let ws = manifest.workspace.ok_or_else(|| format!("{} has no [workspace] section", manifest_path.display()))?;

  let mut members = Vec::new();
  for member_path in &ws.members {
    let member_dir = root.join(member_path);
    let member_manifest_path = member_dir.join("lx.toml");
```

with:

```
  let manifest_path = root.join(lx::LX_MANIFEST);
  let content = std::fs::read_to_string(&manifest_path).map_err(|e| format!("cannot read {}: {e}", manifest_path.display()))?;
  let manifest: RootManifest = toml::from_str(&content).map_err(|e| format!("invalid {}: {e}", manifest_path.display()))?;
  let ws = manifest.workspace.ok_or_else(|| format!("{} has no [workspace] section", manifest_path.display()))?;

  let mut members = Vec::new();
  for member_path in &ws.members {
    let member_dir = root.join(member_path);
    let member_manifest_path = member_dir.join(lx::LX_MANIFEST);
```

### 10c. Update `crates/lx-cli/src/init.rs`

Replace:

```
  let manifest_path = project_dir.join("lx.toml");
```

with:

```
  let manifest_path = project_dir.join(lx::LX_MANIFEST);
```

### 10d. Update `crates/lx-cli/src/install_ops.rs`

Replace:

```
  let manifest_path = root.join("lx.toml");
```

with:

```
  let manifest_path = root.join(lx::LX_MANIFEST);
```

---

## Task 11: Extract sandbox policy lookup helper

### 11a. Add helper function in `crates/lx/src/stdlib/sandbox/mod.rs`

After the `make_handle` function (after the closing `}` of `fn make_handle`), add:

```rust
pub(super) fn get_policy(id: u64, span: SourceSpan) -> Result<dashmap::mapref::one::Ref<'static, u64, Policy>, LxError> {
  POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))
}
```

### 11b. Update `mod.rs` call sites

Replace (in `bi_describe`):

```
  let p = POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
  Ok(policy_to_describe(&p))
```

with:

```
  let p = get_policy(id, span)?;
  Ok(policy_to_describe(&p))
```

Replace (in `bi_permits`):

```
  let p = POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
  Ok(LxVal::Bool(permits_check(&p, &capability, &target)))
```

with:

```
  let p = get_policy(id, span)?;
  Ok(LxVal::Bool(permits_check(&p, &capability, &target)))
```

Replace (in `bi_merge`):

```
    let p = POLICIES.get(&id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
```

with:

```
    let p = get_policy(id, span)?;
```

Replace (in `bi_attenuate`):

```
  let parent = POLICIES.get(&parent_id).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?.clone();
```

with:

```
  let parent = get_policy(parent_id, span)?.clone();
```

### 11c. Update `sandbox_scope.rs`

**File:** `crates/lx/src/stdlib/sandbox/sandbox_scope.rs`

Replace:

```
use super::sandbox::{POLICIES, Policy, policy_id};
```

with:

```
use super::sandbox::{Policy, get_policy, policy_id};
```

Replace:

```
  let policy = POLICIES.get(&pid).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?.clone();
```

with:

```
  let policy = get_policy(pid, span)?.clone();
```

### 11d. Update `sandbox_exec.rs`

**File:** `crates/lx/src/stdlib/sandbox/sandbox_exec.rs`

Replace:

```
use super::sandbox::{POLICIES, policy_id};
```

with:

```
use super::sandbox::{get_policy, policy_id};
```

Replace:

```
  let policy = POLICIES.get(&pid).ok_or_else(|| LxError::runtime("sandbox: policy not found", span))?;
```

with:

```
  let policy = get_policy(pid, span)?;
```

---

## Verification

After all changes, run `just diagnose` to confirm no compilation errors or warnings.
