pub mod types;
mod synth;

use std::collections::HashMap;

use crate::ast::{
  Binding, BindTarget, McpOutputType, Program, SStmt, SType, Stmt, TypeExpr,
};
use crate::span::Span;

use types::{Type, UnificationTable};

pub struct Diagnostic {
  pub msg: String,
  pub span: Span,
}

pub struct CheckResult {
  pub diagnostics: Vec<Diagnostic>,
}

pub(crate) struct Checker {
  pub(crate) table: UnificationTable,
  scope: Vec<HashMap<String, Type>>,
  pub(crate) diagnostics: Vec<Diagnostic>,
}

impl Checker {
  fn new() -> Self {
    Self {
      table: UnificationTable::new(),
      scope: vec![HashMap::new()],
      diagnostics: Vec::new(),
    }
  }

  pub(crate) fn bind(&mut self, name: String, ty: Type) {
    if let Some(scope) = self.scope.last_mut() {
      scope.insert(name, ty);
    }
  }

  pub(crate) fn lookup(&self, name: &str) -> Option<Type> {
    for scope in self.scope.iter().rev() {
      if let Some(ty) = scope.get(name) {
        return Some(ty.clone());
      }
    }
    None
  }

  pub(crate) fn push_scope(&mut self) {
    self.scope.push(HashMap::new());
  }

  pub(crate) fn pop_scope(&mut self) {
    self.scope.pop();
  }

  pub(crate) fn emit(&mut self, msg: String, span: Span) {
    self.diagnostics.push(Diagnostic { msg, span });
  }

  pub(crate) fn fresh(&mut self) -> Type {
    self.table.fresh_var()
  }

  pub(crate) fn resolve_type_ann(&mut self, ty: &SType) -> Type {
    match &ty.node {
      TypeExpr::Named(name) => named_to_type(name),
      TypeExpr::Var(_) => self.fresh(),
      TypeExpr::Applied(name, args) => {
        let resolved: Vec<Type> = args.iter().map(|a| self.resolve_type_ann(a)).collect();
        match name.as_str() {
          "Maybe" if resolved.len() == 1 => {
            Type::Maybe(Box::new(resolved.into_iter().next().unwrap_or(Type::Unknown)))
          },
          "Result" if resolved.len() == 2 => {
            let mut it = resolved.into_iter();
            Type::Result {
              ok: Box::new(it.next().unwrap_or(Type::Unknown)),
              err: Box::new(it.next().unwrap_or(Type::Unknown)),
            }
          },
          _ => Type::Unknown,
        }
      },
      TypeExpr::List(inner) => Type::List(Box::new(self.resolve_type_ann(inner))),
      TypeExpr::Map { key, value } => Type::Map {
        key: Box::new(self.resolve_type_ann(key)),
        value: Box::new(self.resolve_type_ann(value)),
      },
      TypeExpr::Record(fields) => {
        let fs = fields.iter().map(|f| (f.name.clone(), self.resolve_type_ann(&f.ty))).collect();
        Type::Record(fs)
      },
      TypeExpr::Tuple(elems) => {
        Type::Tuple(elems.iter().map(|e| self.resolve_type_ann(e)).collect())
      },
      TypeExpr::Func { param, ret } => Type::Func {
        param: Box::new(self.resolve_type_ann(param)),
        ret: Box::new(self.resolve_type_ann(ret)),
      },
      TypeExpr::Fallible { ok, err } => Type::Result {
        ok: Box::new(self.resolve_type_ann(ok)),
        err: Box::new(self.resolve_type_ann(err)),
      },
    }
  }

  pub(crate) fn check_stmts(&mut self, stmts: &[SStmt]) -> Type {
    let mut result = Type::Unit;
    for stmt in stmts {
      result = self.check_stmt(stmt);
    }
    result
  }

  fn check_stmt(&mut self, stmt: &SStmt) -> Type {
    match &stmt.node {
      Stmt::Binding(b) => { self.check_binding(b); Type::Unit },
      Stmt::TypeDef { variants, .. } => {
        for (ctor_name, _) in variants {
          self.bind(ctor_name.clone(), Type::Unknown);
        }
        Type::Unit
      },
      Stmt::Protocol { fields, .. } => {
        for f in fields { let _ = named_to_type(&f.type_name); }
        Type::Unit
      },
      Stmt::McpDecl { tools, .. } => {
        for tool in tools {
          for f in &tool.input { let _ = named_to_type(&f.type_name); }
          let _ = resolve_mcp_output(&tool.output);
        }
        Type::Unit
      },
      Stmt::FieldUpdate { value, .. } => { self.synth(value); Type::Unit },
      Stmt::Use(_) => Type::Unit,
      Stmt::Expr(e) => self.synth(e),
    }
  }

  fn check_binding(&mut self, b: &Binding) {
    let val_type = self.synth(&b.value);
    if let Some(ann) = &b.type_ann {
      let expected = self.resolve_type_ann(ann);
      if let Err(msg) = self.table.unify(&expected, &val_type) {
        self.emit(format!("binding type mismatch: {msg}"), b.value.span);
      }
    }
    match &b.target {
      BindTarget::Name(name) => self.bind(name.clone(), val_type),
      BindTarget::Reassign(name) => {
        if let Some(existing) = self.lookup(name)
          && let Err(msg) = self.table.unify(&existing, &val_type) {
            self.emit(format!("reassignment type mismatch: {msg}"), b.value.span);
          }
      },
      BindTarget::Pattern(_) => {},
    }
  }
}

pub fn check(program: &Program) -> CheckResult {
  let mut checker = Checker::new();
  for stmt in &program.stmts {
    checker.check_stmt(stmt);
  }
  CheckResult { diagnostics: checker.diagnostics }
}

fn named_to_type(name: &str) -> Type {
  match name {
    "Int" => Type::Int,
    "Float" => Type::Float,
    "Bool" => Type::Bool,
    "Str" => Type::Str,
    "Regex" => Type::Regex,
    "Unit" => Type::Unit,
    "Bytes" => Type::Bytes,
    _ => Type::Unknown,
  }
}

fn resolve_mcp_output(out: &McpOutputType) -> Type {
  match out {
    McpOutputType::Named(n) => named_to_type(n),
    McpOutputType::List(inner) => Type::List(Box::new(resolve_mcp_output(inner))),
    McpOutputType::Record(fields) => {
      Type::Record(fields.iter().map(|f| (f.name.clone(), named_to_type(&f.type_name))).collect())
    },
  }
}
