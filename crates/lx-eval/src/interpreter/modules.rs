use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use indexmap::IndexMap;

use lx_ast::ast::{BindTarget, Core, Program, Stmt, StmtTypeDef, UseKind, UseStmt};
use lx_desugar::folder::desugar;
use lx_parser::parser::parse;
use lx_span::source::FileId;
use lx_value::LxError;
use lx_value::LxVal;
use miette::SourceSpan;

use lx_value::ModuleExports;

use super::Interpreter;

impl Interpreter {
  pub(super) async fn eval_use(&mut self, use_stmt: &UseStmt, span: SourceSpan) -> Result<(), LxError> {
    let str_path: Vec<&str> = use_stmt.path.iter().map(|s| s.as_str()).collect();
    let exports = if crate::stdlib::std_module_exists(&str_path) {
      if let Some(rust_exports) = crate::stdlib::get_std_module(&str_path) {
        rust_exports
      } else if let Some(lx_source) = crate::stdlib::lx_std_module_source(str_path[1]) {
        self.load_module_from_source(str_path[1], lx_source, span).await?
      } else {
        return Err(LxError::runtime(format!("unknown stdlib module: {}", str_path.join("/")), span));
      }
    } else if let Some(file_path) = self.resolve_workspace_module(&str_path) {
      self.load_module(&file_path, span).await?
    } else if let Some(file_path) = self.resolve_dep_module(&str_path) {
      self.load_module(&file_path, span).await?
    } else {
      let source_dir = self.source_dir.as_ref().ok_or_else(|| LxError::runtime("cannot resolve module path: no source directory", span))?.clone();
      let file_path = resolve_module_path(&source_dir, &str_path, span)?;
      self.load_module(&file_path, span).await?
    };
    let env = self.env.child();
    for name in &exports.variant_ctors {
      if let Some(val) = exports.bindings.get(name) {
        env.bind(*name, val.clone());
      }
    }
    match &use_stmt.kind {
      UseKind::Whole => {
        let module_name = use_stmt.path.last().ok_or_else(|| LxError::runtime("empty module path", span))?;
        let record = LxVal::record(exports.bindings.clone());
        env.bind(*module_name, record);
      },
      UseKind::Alias(alias) => {
        let record = LxVal::record(exports.bindings.clone());
        env.bind(*alias, record);
      },
      UseKind::Selective(names) => {
        for name in names {
          let val = exports.bindings.get(name).ok_or_else(|| LxError::runtime(format!("'{name}' not exported by module"), span))?;
          env.bind(*name, val.clone());
        }
      },
    }
    self.env = Arc::new(env);
    Ok(())
  }



  fn resolve_workspace_module(&self, path: &[&str]) -> Option<PathBuf> {
    if path.len() < 2 {
      return None;
    }
    let member_dir = self.ctx.workspace_members.get(path[0])?;
    let mut result = member_dir.clone();
    result.extend(&path[1..]);
    result.set_extension("lx");
    Some(result)
  }

  fn resolve_dep_module(&self, path: &[&str]) -> Option<PathBuf> {
    if path.is_empty() {
      return None;
    }
    let dep_dir = self.ctx.dep_dirs.get(path[0])?;
    if path.len() == 1 {
      let entry = dep_dir.join("main.lx");
      if entry.exists() {
        return Some(entry);
      }
      let src_entry = dep_dir.join("src").join("main.lx");
      if src_entry.exists() {
        return Some(src_entry);
      }
      return None;
    }
    let mut result = dep_dir.clone();
    result.extend(&path[1..]);
    result.set_extension("lx");
    if result.exists() { Some(result) } else { None }
  }

  async fn load_module_from_source(&mut self, name: &str, source: &str, span: SourceSpan) -> Result<ModuleExports, LxError> {
    let cache_key = PathBuf::from(format!("__std_lx_{name}"));
    {
      let cache = self.module_cache.lock();
      if let Some(exports) = cache.get(&cache_key) {
        return Ok(exports.clone());
      }
    }
    let (tokens, comments) = lx_parser::lexer::lex(source).map_err(|e| LxError::runtime(format!("std/{name}: {e}"), span))?;
    let result = parse(tokens, FileId::new(0), comments, source);
    let surface = result.program.ok_or_else(|| LxError::runtime(format!("std/{name}: parse error"), span))?;
    let program = desugar(surface);
    let saved_source_dir = self.ctx.source_dir.lock().clone();
    let mut mod_interp = Interpreter::new(source, None, Arc::clone(&self.ctx));
    mod_interp.module_cache = Arc::clone(&self.module_cache);
    mod_interp.loading = Arc::clone(&self.loading);
    mod_interp.exec(&program).await.map_err(|e| LxError::runtime(format!("std/{name}: {e}"), span))?;
    let exports = collect_exports(&program, &mod_interp);
    *self.ctx.source_dir.lock() = saved_source_dir;
    self.module_cache.lock().insert(cache_key, exports.clone());
    Ok(exports)
  }

  pub(crate) async fn load_module(&mut self, file_path: &PathBuf, span: SourceSpan) -> Result<ModuleExports, LxError> {
    let canonical = fs::canonicalize(file_path).map_err(|e| LxError::runtime(format!("cannot resolve module '{}': {e}", file_path.display()), span))?;
    {
      let cache = self.module_cache.lock();
      if let Some(exports) = cache.get(&canonical) {
        return Ok(exports.clone());
      }
    }
    {
      let mut loading = self.loading.lock();
      if !loading.insert(canonical.clone()) {
        return Err(LxError::runtime(format!("circular import: {}", canonical.display()), span));
      }
    }
    let source = fs::read_to_string(file_path).map_err(|e| LxError::runtime(format!("cannot read module '{}': {e}", file_path.display()), span))?;
    let (tokens, comments) = lx_parser::lexer::lex(&source).map_err(|e| LxError::runtime(format!("module '{}': {e}", file_path.display()), span))?;
    let result = parse(tokens, FileId::new(0), comments, &source);
    let surface = result.program.ok_or_else(|| {
      let msgs: Vec<String> = result.errors.iter().map(|e| format!("{e}")).collect();
      LxError::runtime(format!("module '{}': {}", file_path.display(), msgs.join("; ")), span)
    })?;
    if !result.errors.is_empty() {
      let msgs: Vec<String> = result.errors.iter().map(|e| format!("{e}")).collect();
      eprintln!("parse warning in module '{}': {}", file_path.display(), msgs.join("; "));
    }
    let program = desugar(surface);
    let module_dir = file_path.parent().map(|p| p.to_path_buf());
    let saved_source_dir = self.ctx.source_dir.lock().clone();
    let mut mod_interp = Interpreter::new(&source, module_dir, Arc::clone(&self.ctx));
    mod_interp.module_cache = Arc::clone(&self.module_cache);
    mod_interp.loading = Arc::clone(&self.loading);
    mod_interp.exec(&program).await.map_err(|e| LxError::runtime(format!("module '{}': {e}", file_path.display()), span))?;
    let exports = collect_exports(&program, &mod_interp);
    *self.ctx.source_dir.lock() = saved_source_dir;
    self.module_cache.lock().insert(canonical.clone(), exports.clone());
    self.loading.lock().remove(&canonical);
    Ok(exports)
  }
}

fn resolve_module_path(source_dir: &Path, path: &[&str], span: SourceSpan) -> Result<PathBuf, LxError> {
  if path.is_empty() {
    return Err(LxError::runtime("empty module path", span));
  }
  let mut result = if path[0] == "." || path[0] == ".." {
    let mut base = source_dir.to_path_buf();
    let mut idx = 0;
    while idx < path.len() && path[idx] == ".." {
      base = base.parent().ok_or_else(|| LxError::runtime("cannot go above root directory", span))?.to_path_buf();
      idx += 1;
    }
    if idx == 0 {
      idx = 1;
    }
    base.extend(&path[idx..]);
    base
  } else {
    return Err(LxError::runtime(format!("unknown module path: {} (use ./relative, std/module, or dep-name/module)", path.join("/")), span));
  };
  result.set_extension("lx");
  Ok(result)
}

fn collect_exports(program: &Program<Core>, interp: &Interpreter) -> ModuleExports {
  let mut bindings = IndexMap::new();
  let mut variant_ctors = Vec::new();
  for &sid in &program.stmts {
    let stmt = program.arena.stmt(sid);
    match stmt {
      Stmt::Binding(b) if b.exported => {
        if let BindTarget::Name(name) = &b.target
          && let Some(val) = interp.env.get(*name)
        {
          bindings.insert(*name, val);
        }
      },
      Stmt::TypeDef(StmtTypeDef { exported: true, variants, .. }) => {
        for (ctor_name, _) in variants {
          if let Some(val) = interp.env.get(*ctor_name) {
            variant_ctors.push(*ctor_name);
            bindings.insert(*ctor_name, val);
          }
        }
      },
      Stmt::TraitDecl(data) if data.exported => {
        if let Some(val) = interp.env.get(data.name) {
          bindings.insert(data.name, val);
        }
      },
      Stmt::ClassDecl(data) if data.exported => {
        if let Some(val) = interp.env.get(data.name) {
          bindings.insert(data.name, val);
        }
      },
      Stmt::TraitUnion(def) if def.exported => {
        if let Some(val) = interp.env.get(def.name) {
          bindings.insert(def.name, val);
        }
      },
      _ => {},
    }
  }
  ModuleExports { bindings, variant_ctors }
}
