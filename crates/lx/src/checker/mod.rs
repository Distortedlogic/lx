mod capture;
mod exhaust;
mod stmts;
mod synth;
mod synth_helpers;
pub mod types;

use std::collections::{HashMap, HashSet};

use crate::ast::{Program, SType, TypeExpr};
use miette::SourceSpan;

use types::{Type, UnificationTable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
  Error,
  Warning,
}

pub struct Diagnostic {
  pub level: DiagLevel,
  pub msg: String,
  pub span: SourceSpan,
}

pub struct CheckResult {
  pub diagnostics: Vec<Diagnostic>,
}

pub(crate) struct Checker {
  pub(crate) table: UnificationTable,
  scope: Vec<HashMap<String, Type>>,
  pub(crate) diagnostics: Vec<Diagnostic>,
  pub(crate) type_defs: HashMap<String, Vec<String>>,
  pub(crate) mutables: HashSet<String>,
  import_sources: HashMap<String, SourceSpan>,
  pub(crate) trait_fields: HashMap<String, Vec<(String, Type)>>,
}

impl Checker {
  fn new() -> Self {
    Self {
      table: UnificationTable::new(),
      scope: vec![HashMap::new()],
      diagnostics: Vec::new(),
      type_defs: HashMap::new(),
      mutables: HashSet::new(),
      import_sources: HashMap::new(),
      trait_fields: HashMap::new(),
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

  pub(crate) fn emit(&mut self, msg: String, span: SourceSpan) {
    self.diagnostics.push(Diagnostic { level: DiagLevel::Error, msg, span });
  }

  pub(crate) fn emit_warning(&mut self, msg: String, span: SourceSpan) {
    self.diagnostics.push(Diagnostic { level: DiagLevel::Warning, msg, span });
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
          "Maybe" if resolved.len() == 1 => Type::Maybe(Box::new(resolved.into_iter().next().unwrap_or(Type::Unknown))),
          "Result" if resolved.len() == 2 => {
            let mut it = resolved.into_iter();
            Type::Result { ok: Box::new(it.next().unwrap_or(Type::Unknown)), err: Box::new(it.next().unwrap_or(Type::Unknown)) }
          },
          _ => Type::Unknown,
        }
      },
      TypeExpr::List(inner) => Type::List(Box::new(self.resolve_type_ann(inner))),
      TypeExpr::Map { key, value } => Type::Map { key: Box::new(self.resolve_type_ann(key)), value: Box::new(self.resolve_type_ann(value)) },
      TypeExpr::Record(fields) => {
        let fs = fields.iter().map(|f| (f.name.clone(), self.resolve_type_ann(&f.ty))).collect();
        Type::Record(fs)
      },
      TypeExpr::Tuple(elems) => Type::Tuple(elems.iter().map(|e| self.resolve_type_ann(e)).collect()),
      TypeExpr::Func { param, ret } => Type::Func { param: Box::new(self.resolve_type_ann(param)), ret: Box::new(self.resolve_type_ann(ret)) },
      TypeExpr::Fallible { ok, err } => Type::Result { ok: Box::new(self.resolve_type_ann(ok)), err: Box::new(self.resolve_type_ann(err)) },
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
    "Unit" => Type::Unit,
    "Bytes" => Type::Bytes,
    _ => Type::Unknown,
  }
}
