use crate::sym::Sym;
mod capture;
mod check_expr;
pub mod diagnostics;
mod exhaust;
mod exhaust_core;
mod exhaust_types;
mod resolve;
mod symbol_table;
mod synth_compound;
mod synth_control;
mod type_ops;
pub mod types;
pub mod unification;
mod visit_stmt;

use std::collections::{HashMap, HashSet};

use std::sync::Arc;

use crate::ast::{AstArena, Core, Program, TypeExpr, TypeExprId};
use diagnostics::{DiagnosticKind, Fix};
use miette::SourceSpan;
use symbol_table::SymbolTable;

use types::{Type, Variant};
use unification::{TypeError, UnificationTable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
  Error,
  Warning,
}

pub struct Diagnostic {
  pub level: DiagLevel,
  pub kind: DiagnosticKind,
  pub span: SourceSpan,
  pub secondary: Vec<(SourceSpan, String)>,
  pub fix: Option<Fix>,
}

pub struct CheckResult {
  pub diagnostics: Vec<Diagnostic>,
  pub source: Arc<str>,
}

pub(crate) struct Checker<'a> {
  pub(crate) table: UnificationTable,
  scope: Vec<HashMap<Sym, Type>>,
  pub(crate) diagnostics: Vec<Diagnostic>,
  pub(crate) type_defs: HashMap<Sym, Vec<Sym>>,
  import_sources: HashMap<Sym, SourceSpan>,
  pub(crate) trait_fields: HashMap<Sym, Vec<(Sym, Type)>>,
  pub(crate) arena: &'a AstArena,
  mutables: HashSet<Sym>,
  symbols: SymbolTable,
}

impl<'a> Checker<'a> {
  fn new(arena: &'a AstArena, symbols: SymbolTable) -> Self {
    Self {
      table: UnificationTable::new(),
      scope: vec![HashMap::new()],
      diagnostics: Vec::new(),
      type_defs: HashMap::new(),
      import_sources: HashMap::new(),
      trait_fields: HashMap::new(),
      arena,
      mutables: HashSet::new(),
      symbols,
    }
  }

  pub(crate) fn bind(&mut self, name: Sym, ty: Type) {
    if let Some(scope) = self.scope.last_mut() {
      scope.insert(name, ty);
    }
  }

  pub(crate) fn lookup(&self, name: Sym) -> Option<Type> {
    for scope in self.scope.iter().rev() {
      if let Some(ty) = scope.get(&name) {
        return Some(ty.clone());
      }
    }
    None
  }

  pub(crate) fn is_mutable(&self, name: Sym) -> bool {
    self.mutables.contains(&name)
  }

  pub(crate) fn push_scope(&mut self) {
    self.scope.push(HashMap::new());
  }

  pub(crate) fn pop_scope(&mut self) {
    self.scope.pop();
  }

  pub(crate) fn emit(&mut self, level: DiagLevel, kind: DiagnosticKind, span: SourceSpan) {
    let fix = kind.suggest_fix(span);
    self.diagnostics.push(Diagnostic { level, kind, span, secondary: Vec::new(), fix });
  }

  pub(crate) fn emit_type_error(&mut self, te: &TypeError, span: SourceSpan) {
    let kind = DiagnosticKind::TypeMismatch { error: te.clone() };
    let fix = kind.suggest_fix(span);
    let mut secondary = Vec::new();
    if let Some(origin) = te.expected_origin {
      secondary.push((origin, "expected type declared here".into()));
    }
    self.diagnostics.push(Diagnostic { level: DiagLevel::Error, kind, span, secondary, fix });
  }

  pub(crate) fn fresh(&mut self) -> Type {
    self.table.fresh_var()
  }

  pub(crate) fn resolve_type_ann(&mut self, ty_id: TypeExprId) -> Type {
    match self.arena.type_expr(ty_id).clone() {
      TypeExpr::Named(name) => {
        let t = named_to_type(name.as_str());
        if t != Type::Unknown {
          return t;
        }
        let sym = crate::sym::intern(name.as_str());
        if let Some(variant_names) = self.type_defs.get(&sym).cloned() {
          let variants = variant_names.iter().map(|vn| Variant { name: *vn, fields: vec![] }).collect();
          return Type::Union { name: sym, variants };
        }
        if let Some(fields) = self.trait_fields.get(&sym).cloned() {
          return Type::Record(fields);
        }
        Type::Unknown
      },
      TypeExpr::Var(_) => self.fresh(),
      TypeExpr::Applied(name, args) => {
        let resolved: Vec<Type> = args.iter().map(|a| self.resolve_type_ann(*a)).collect();
        match name.as_str() {
          "Maybe" if resolved.len() == 1 => Type::Maybe(Box::new(resolved.into_iter().next().unwrap_or(Type::Unknown))),
          "Result" if resolved.len() == 2 => {
            let mut it = resolved.into_iter();
            Type::Result { ok: Box::new(it.next().unwrap_or(Type::Unknown)), err: Box::new(it.next().unwrap_or(Type::Unknown)) }
          },
          _ => {
            let sym = crate::sym::intern(name.as_str());
            if let Some(variant_names) = self.type_defs.get(&sym).cloned() {
              let variants = variant_names.iter().map(|vn| Variant { name: *vn, fields: vec![] }).collect();
              Type::Union { name: sym, variants }
            } else {
              Type::Unknown
            }
          },
        }
      },
      TypeExpr::List(inner) => Type::List(Box::new(self.resolve_type_ann(inner))),
      TypeExpr::Map { key, value } => {
        Type::Map { key: Box::new(self.resolve_type_ann(key)), value: Box::new(self.resolve_type_ann(value)) }
      },
      TypeExpr::Record(fields) => {
        let fs = fields.iter().map(|f| (f.name, self.resolve_type_ann(f.ty))).collect();
        Type::Record(fs)
      },
      TypeExpr::Tuple(elems) => {
        Type::Tuple(elems.iter().map(|e| self.resolve_type_ann(*e)).collect())
      },
      TypeExpr::Func { param, ret } => {
        Type::Func { params: vec![self.resolve_type_ann(param)], ret: Box::new(self.resolve_type_ann(ret)) }
      },
      TypeExpr::Fallible { ok, err } => {
        Type::Result { ok: Box::new(self.resolve_type_ann(ok)), err: Box::new(self.resolve_type_ann(err)) }
      },
    }
  }

  fn check_program(&mut self, program: &Program<Core>) {
    for &sid in &program.stmts {
      self.check_stmt(sid, &program.arena);
    }
  }
}

pub fn check(program: &Program<Core>, source: Arc<str>) -> CheckResult {
  let symbols = resolve::resolve(program);
  let mut checker = Checker::new(&program.arena, symbols);
  checker.check_program(program);
  CheckResult { diagnostics: checker.diagnostics, source }
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
