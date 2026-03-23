use crate::sym::Sym;
mod capture;
mod check_expr;
pub mod diagnostics;
mod exhaust;
mod exhaust_core;
mod exhaust_types;
pub(crate) mod symbol_table;
mod synth_compound;
mod synth_control;
pub mod type_arena;
mod type_ops;
pub mod types;
pub mod unification;
mod visit_stmt;

use std::collections::{HashMap, HashSet};

use std::sync::Arc;

use crate::ast::{AstArena, Core, ExprId, Program, TypeExpr, TypeExprId};
use diagnostics::{DiagnosticKind, Fix};
use miette::SourceSpan;
use symbol_table::SymbolTable;
use type_arena::{TypeArena, TypeId};

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
  pub expr_types: HashMap<ExprId, TypeId>,
  pub type_arena: TypeArena,
}

pub(crate) struct Checker<'a> {
  pub(crate) table: UnificationTable,
  pub(crate) type_arena: TypeArena,
  pub(crate) diagnostics: Vec<Diagnostic>,
  pub(crate) type_defs: HashMap<Sym, Vec<Sym>>,
  import_sources: HashMap<Sym, SourceSpan>,
  pub(crate) trait_fields: HashMap<Sym, Vec<(Sym, TypeId)>>,
  pub(crate) arena: &'a AstArena,
  mutables: HashSet<Sym>,
  pub(crate) symbols: SymbolTable,
  pub(crate) expr_types: HashMap<ExprId, TypeId>,
}

impl<'a> Checker<'a> {
  fn new(arena: &'a AstArena) -> Self {
    Self {
      table: UnificationTable::new(),
      type_arena: TypeArena::new(),
      diagnostics: Vec::new(),
      type_defs: HashMap::new(),
      import_sources: HashMap::new(),
      trait_fields: HashMap::new(),
      arena,
      mutables: HashSet::new(),
      symbols: SymbolTable::new(),
      expr_types: HashMap::new(),
    }
  }

  pub(crate) fn record_type(&mut self, id: ExprId, ty: TypeId) {
    self.expr_types.insert(id, ty);
  }

  pub(crate) fn is_mutable(&self, name: Sym) -> bool {
    self.mutables.contains(&name)
  }

  pub(crate) fn emit(&mut self, level: DiagLevel, kind: DiagnosticKind, span: SourceSpan) {
    let fix = kind.suggest_fix(span, &self.type_arena);
    self.diagnostics.push(Diagnostic { level, kind, span, secondary: Vec::new(), fix });
  }

  pub(crate) fn emit_type_error(&mut self, te: &TypeError, span: SourceSpan) {
    let kind = DiagnosticKind::TypeMismatch { error: te.clone() };
    let fix = kind.suggest_fix(span, &self.type_arena);
    let mut secondary = Vec::new();
    if let Some(origin) = te.expected_origin {
      secondary.push((origin, "expected type declared here".into()));
    }
    self.diagnostics.push(Diagnostic { level: DiagLevel::Error, kind, span, secondary, fix });
  }

  pub(crate) fn fresh(&mut self) -> TypeId {
    self.table.fresh_var(&mut self.type_arena)
  }

  pub(crate) fn resolve_type_ann(&mut self, ty_id: TypeExprId) -> TypeId {
    match self.arena.type_expr(ty_id).clone() {
      TypeExpr::Named(name) => {
        let t = self.named_to_type(name.as_str());
        if t != self.type_arena.unknown() {
          return t;
        }
        let sym = crate::sym::intern(name.as_str());
        if let Some(variant_names) = self.type_defs.get(&sym).cloned() {
          let variants = variant_names.iter().map(|vn| Variant { name: *vn, fields: vec![] }).collect();
          return self.type_arena.alloc(Type::Union { name: sym, variants });
        }
        if let Some(fields) = self.trait_fields.get(&sym).cloned() {
          return self.type_arena.alloc(Type::Record(fields));
        }
        self.type_arena.unknown()
      },
      TypeExpr::Var(_) => self.fresh(),
      TypeExpr::Applied(name, args) => {
        let resolved: Vec<TypeId> = args.iter().map(|a| self.resolve_type_ann(*a)).collect();
        match name.as_str() {
          "Maybe" if resolved.len() == 1 => {
            let inner = resolved.into_iter().next().unwrap_or(self.type_arena.unknown());
            self.type_arena.alloc(Type::Maybe(inner))
          },
          "Result" if resolved.len() == 2 => {
            let mut it = resolved.into_iter();
            let ok = it.next().unwrap_or(self.type_arena.unknown());
            let err = it.next().unwrap_or(self.type_arena.unknown());
            self.type_arena.alloc(Type::Result { ok, err })
          },
          _ => {
            let sym = crate::sym::intern(name.as_str());
            if let Some(variant_names) = self.type_defs.get(&sym).cloned() {
              let variants = variant_names.iter().map(|vn| Variant { name: *vn, fields: vec![] }).collect();
              self.type_arena.alloc(Type::Union { name: sym, variants })
            } else {
              self.type_arena.unknown()
            }
          },
        }
      },
      TypeExpr::List(inner) => {
        let inner = self.resolve_type_ann(inner);
        self.type_arena.alloc(Type::List(inner))
      },
      TypeExpr::Map { key, value } => {
        let key = self.resolve_type_ann(key);
        let value = self.resolve_type_ann(value);
        self.type_arena.alloc(Type::Map { key, value })
      },
      TypeExpr::Record(fields) => {
        let fs = fields.iter().map(|f| (f.name, self.resolve_type_ann(f.ty))).collect();
        self.type_arena.alloc(Type::Record(fs))
      },
      TypeExpr::Tuple(elems) => {
        let elems: Vec<_> = elems.iter().map(|e| self.resolve_type_ann(*e)).collect();
        self.type_arena.alloc(Type::Tuple(elems))
      },
      TypeExpr::Func { param, ret } => {
        let param = self.resolve_type_ann(param);
        let ret = self.resolve_type_ann(ret);
        self.type_arena.alloc(Type::Func { param, ret })
      },
      TypeExpr::Fallible { ok, err } => {
        let ok = self.resolve_type_ann(ok);
        let err = self.resolve_type_ann(err);
        self.type_arena.alloc(Type::Result { ok, err })
      },
    }
  }

  fn named_to_type(&self, name: &str) -> TypeId {
    match name {
      "Int" => self.type_arena.int(),
      "Float" => self.type_arena.float(),
      "Bool" => self.type_arena.bool(),
      "Str" => self.type_arena.str(),
      "Unit" => self.type_arena.unit(),
      "Bytes" => self.type_arena.bytes(),
      _ => self.type_arena.unknown(),
    }
  }

  fn check_program(&mut self, program: &Program<Core>) {
    for &sid in &program.stmts {
      self.check_stmt(sid, &program.arena);
    }
  }
}

pub fn check(program: &Program<Core>, source: Arc<str>) -> CheckResult {
  let mut checker = Checker::new(&program.arena);
  checker.check_program(program);
  CheckResult { diagnostics: checker.diagnostics, source, expr_types: checker.expr_types, type_arena: checker.type_arena }
}
