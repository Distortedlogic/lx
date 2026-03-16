use std::path::PathBuf;
use std::sync::Arc;

use indexmap::IndexMap;

use crate::ast::{BindTarget, Program, Stmt, UseKind, UseStmt};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use super::{Interpreter, ModuleExports};

impl Interpreter {
    pub(super) fn eval_use(&mut self, use_stmt: &UseStmt, span: Span) -> Result<(), LxError> {
        let exports = if crate::stdlib::std_module_exists(&use_stmt.path) {
            crate::stdlib::get_std_module(&use_stmt.path).ok_or_else(|| {
                LxError::runtime(
                    format!("unknown stdlib module: {}", use_stmt.path.join("/")),
                    span,
                )
            })?
        } else {
            let source_dir = self
                .source_dir
                .as_ref()
                .ok_or_else(|| {
                    LxError::runtime("cannot resolve module path: no source directory", span)
                })?
                .clone();
            let file_path = resolve_module_path(&source_dir, &use_stmt.path, span)?;
            self.load_module(&file_path, span)?
        };
        let mut env = self.env.child();
        for name in &exports.variant_ctors {
            if let Some(val) = exports.bindings.get(name) {
                env.bind(name.clone(), val.clone());
            }
        }
        match &use_stmt.kind {
            UseKind::Whole => {
                let module_name = use_stmt
                    .path
                    .last()
                    .ok_or_else(|| LxError::runtime("empty module path", span))?;
                let record = Value::Record(Arc::new(exports.bindings.clone()));
                env.bind(module_name.clone(), record);
            }
            UseKind::Alias(alias) => {
                let record = Value::Record(Arc::new(exports.bindings.clone()));
                env.bind(alias.clone(), record);
            }
            UseKind::Selective(names) => {
                for name in names {
                    let val = exports.bindings.get(name).ok_or_else(|| {
                        LxError::runtime(format!("'{name}' not exported by module"), span)
                    })?;
                    env.bind(name.clone(), val.clone());
                }
            }
        }
        self.env = env.into_arc();
        Ok(())
    }

    fn load_module(&mut self, file_path: &PathBuf, span: Span) -> Result<ModuleExports, LxError> {
        let canonical = std::fs::canonicalize(file_path).map_err(|e| {
            LxError::runtime(
                format!("cannot resolve module '{}': {e}", file_path.display()),
                span,
            )
        })?;
        {
            let cache = self.module_cache.lock();
            if let Some(exports) = cache.get(&canonical) {
                return Ok(exports.clone());
            }
        }
        {
            let mut loading = self.loading.lock();
            if !loading.insert(canonical.clone()) {
                return Err(LxError::runtime(
                    format!("circular import: {}", canonical.display()),
                    span,
                ));
            }
        }
        let source = std::fs::read_to_string(file_path).map_err(|e| {
            LxError::runtime(
                format!("cannot read module '{}': {e}", file_path.display()),
                span,
            )
        })?;
        let tokens = crate::lexer::lex(&source).map_err(|e| {
            LxError::runtime(format!("module '{}': {e}", file_path.display()), span)
        })?;
        let program = crate::parser::parse(tokens).map_err(|e| {
            LxError::runtime(format!("module '{}': {e}", file_path.display()), span)
        })?;
        let module_dir = file_path.parent().map(|p| p.to_path_buf());
        let mut mod_interp = Interpreter::new(&source, module_dir, Arc::clone(&self.ctx));
        mod_interp.module_cache = Arc::clone(&self.module_cache);
        mod_interp.loading = Arc::clone(&self.loading);
        mod_interp.exec(&program).map_err(|e| {
            LxError::runtime(format!("module '{}': {e}", file_path.display()), span)
        })?;
        let exports = collect_exports(&program, &mod_interp);
        self.module_cache
            .lock()
            .insert(canonical.clone(), exports.clone());
        self.loading.lock().remove(&canonical);
        Ok(exports)
    }
}

fn resolve_module_path(
    source_dir: &std::path::Path,
    path: &[String],
    span: Span,
) -> Result<PathBuf, LxError> {
    if path.is_empty() {
        return Err(LxError::runtime("empty module path", span));
    }
    let mut result = if path[0] == "." || path[0] == ".." {
        let mut base = source_dir.to_path_buf();
        if path[0] == ".." {
            base = base
                .parent()
                .ok_or_else(|| LxError::runtime("cannot go above root directory", span))?
                .to_path_buf();
        }
        for segment in &path[1..] {
            base.push(segment);
        }
        base
    } else {
        return Err(LxError::runtime(
            format!(
                "unknown module path: {} (use ./relative or std/module)",
                path.join("/")
            ),
            span,
        ));
    };
    result.set_extension("lx");
    Ok(result)
}

fn collect_exports(program: &Program, interp: &Interpreter) -> ModuleExports {
    let mut bindings = IndexMap::new();
    let mut variant_ctors = Vec::new();
    for stmt in &program.stmts {
        match &stmt.node {
            Stmt::Binding(b) if b.exported => {
                if let BindTarget::Name(name) = &b.target
                    && let Some(val) = interp.env.get(name)
                {
                    bindings.insert(name.clone(), val);
                }
            }
            Stmt::TypeDef {
                exported: true,
                variants,
                ..
            } => {
                for (ctor_name, _) in variants {
                    if let Some(val) = interp.env.get(ctor_name) {
                        variant_ctors.push(ctor_name.clone());
                        bindings.insert(ctor_name.clone(), val);
                    }
                }
            }
            Stmt::Protocol {
                exported: true,
                name,
                ..
            } => {
                if let Some(val) = interp.env.get(name) {
                    bindings.insert(name.clone(), val);
                }
            }
            Stmt::ProtocolUnion(def) if def.exported => {
                if let Some(val) = interp.env.get(&def.name) {
                    bindings.insert(def.name.clone(), val);
                }
            }
            Stmt::McpDecl {
                exported: true,
                name,
                ..
            } => {
                if let Some(val) = interp.env.get(name) {
                    bindings.insert(name.clone(), val);
                }
            }
            Stmt::TraitDecl {
                exported: true,
                name,
                ..
            } => {
                if let Some(val) = interp.env.get(name) {
                    bindings.insert(name.clone(), val);
                }
            }
            _ => {}
        }
    }
    ModuleExports {
        bindings,
        variant_ctors,
    }
}
