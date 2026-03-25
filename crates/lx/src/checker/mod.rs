use crate::sym::Sym;
mod capture;
mod check_expr;
pub mod diagnostics;
mod exhaust;
mod exhaust_core;
mod exhaust_types;
mod generics;
mod infer_pattern;
pub mod module_graph;
mod narrowing;
pub mod semantic;
mod stdlib_sigs;
pub(crate) mod suggest;
mod synth_compound;
mod synth_control;
pub mod type_arena;
pub mod type_error;
mod type_ops;
pub mod types;
pub mod unification;
mod visit_stmt;

use std::collections::HashMap;

use std::sync::Arc;

use la_arena::ArenaMap;

use crate::ast::{AstArena, Core, ExprId, Program, Stmt, StmtId, TypeExpr, TypeExprId};
use crate::visitor::{AstVisitor, VisitAction};
use diagnostics::{DiagnosticKind, Fix};
use miette::SourceSpan;
use module_graph::ModuleSignature;
use narrowing::NarrowingEnv;
use semantic::{SemanticModel, SemanticModelBuilder};
use type_arena::{TypeArena, TypeId};

use type_error::TypeError;
use types::{Type, Variant};
use unification::UnificationTable;

use crate::linter::{RuleRegistry, lint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagLevel {
  Error,
  Warning,
}

pub struct Diagnostic {
  pub level: DiagLevel,
  pub kind: DiagnosticKind,
  pub code: &'static str,
  pub span: SourceSpan,
  pub secondary: Vec<(SourceSpan, String)>,
  pub fix: Option<Fix>,
}

pub struct CheckResult {
  pub diagnostics: Vec<Diagnostic>,
  pub source: Arc<str>,
  pub semantic: SemanticModel,
}

pub(crate) struct Checker<'a> {
  pub(crate) table: UnificationTable,
  pub(crate) type_arena: TypeArena,
  pub(crate) diagnostics: Vec<Diagnostic>,
  pub(crate) type_defs: HashMap<Sym, Vec<Sym>>,
  import_sources: HashMap<Sym, SourceSpan>,
  pub(crate) import_signatures: HashMap<Sym, ModuleSignature>,
  pub(crate) translated_imports: HashMap<Sym, HashMap<Sym, TypeId>>,
  pub(crate) trait_fields: HashMap<Sym, Vec<(Sym, TypeId)>>,
  pub(crate) arena: &'a AstArena,
  pub(crate) sem: SemanticModelBuilder,
  pub(crate) expr_types: ArenaMap<ExprId, TypeId>,
  generic_scope: Vec<HashMap<Sym, TypeId>>,
  pub(crate) narrowing: NarrowingEnv,
  stdlib_sigs: HashMap<String, ModuleSignature>,
}

impl<'a> Checker<'a> {
  fn new(arena: &'a AstArena) -> Self {
    Self {
      table: UnificationTable::new(),
      type_arena: TypeArena::new(),
      diagnostics: Vec::new(),
      type_defs: HashMap::new(),
      import_sources: HashMap::new(),
      import_signatures: HashMap::new(),
      translated_imports: HashMap::new(),
      trait_fields: HashMap::new(),
      arena,
      sem: SemanticModelBuilder::new(),
      expr_types: ArenaMap::default(),
      generic_scope: Vec::new(),
      narrowing: NarrowingEnv::new(),
      stdlib_sigs: stdlib_sigs::build_stdlib_signatures(),
    }
  }

  pub(crate) fn record_type(&mut self, id: ExprId, ty: TypeId) {
    self.expr_types.insert(id, ty);
  }

  pub(crate) fn is_mutable(&self, name: Sym) -> bool {
    self.sem.resolve_in_scope(name).map(|id| self.sem.definitions[id.index()].mutable).unwrap_or(false)
  }

  pub(crate) fn emit(&mut self, level: DiagLevel, kind: DiagnosticKind, span: SourceSpan) {
    let fix = kind.suggest_fix(span, &self.type_arena);
    let code = kind.code();
    self.diagnostics.push(Diagnostic { level, kind, code, span, secondary: Vec::new(), fix });
  }

  pub(crate) fn make_type_error_diagnostic(&self, te: &TypeError, span: SourceSpan) -> Diagnostic {
    let kind = DiagnosticKind::TypeMismatch { error: te.clone() };
    let fix = kind.suggest_fix(span, &self.type_arena);
    let mut secondary = Vec::new();
    if let Some(origin) = te.expected_origin {
      secondary.push((origin, "expected type declared here".into()));
    }
    let code = kind.code();
    Diagnostic { level: DiagLevel::Error, kind, code, span, secondary, fix }
  }

  pub(crate) fn emit_type_error(&mut self, te: &TypeError, span: SourceSpan) {
    let diag = self.make_type_error_diagnostic(te, span);
    self.diagnostics.push(diag);
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
      TypeExpr::Var(name) => {
        if let Some(ty) = self.lookup_type_param(name) {
          return ty;
        }
        self.fresh()
      },
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
    let _ = crate::visitor::walk_program(self, program);
  }
}

impl AstVisitor for Checker<'_> {
  fn visit_stmt(&mut self, _id: StmtId, _stmt: &Stmt, _span: SourceSpan) -> VisitAction {
    VisitAction::Skip
  }

  fn leave_stmt(&mut self, id: StmtId, _stmt: &Stmt, _span: SourceSpan) {
    let arena = self.arena;
    self.check_stmt(id, arena);
  }
}

pub fn check(program: &Program<Core>, source: Arc<str>) -> CheckResult {
  let mut checker = Checker::new(&program.arena);
  checker.check_program(program);
  let semantic = checker.sem.build(checker.expr_types, checker.type_defs, checker.trait_fields, checker.type_arena);
  let mut diagnostics = checker.diagnostics;
  let mut registry = RuleRegistry::default_rules();
  let lint_diags = lint(program, &semantic, &mut registry);
  diagnostics.extend(lint_diags);
  CheckResult { diagnostics, source, semantic }
}

pub fn check_with_imports(program: &Program<Core>, source: Arc<str>, import_signatures: HashMap<Sym, ModuleSignature>) -> CheckResult {
  let mut checker = Checker::new(&program.arena);
  for (module_name, sig) in &import_signatures {
    let mut translated = HashMap::new();
    for (binding_name, &foreign_type_id) in &sig.bindings {
      let local_id = checker.type_arena.copy_type(foreign_type_id, &sig.type_arena);
      translated.insert(*binding_name, local_id);
    }
    checker.translated_imports.insert(*module_name, translated);
    for (type_name, variants) in &sig.types {
      checker.type_defs.insert(*type_name, variants.clone());
    }
    for (trait_name, fields) in &sig.traits {
      let local_fields: Vec<(Sym, TypeId)> = fields.iter().map(|(n, t)| (*n, checker.type_arena.copy_type(*t, &sig.type_arena))).collect();
      checker.trait_fields.insert(*trait_name, local_fields);
    }
  }
  checker.import_signatures = import_signatures;
  checker.check_program(program);
  let semantic = checker.sem.build(checker.expr_types, checker.type_defs, checker.trait_fields, checker.type_arena);
  let mut diagnostics = checker.diagnostics;
  let mut registry = RuleRegistry::default_rules();
  let lint_diags = lint(program, &semantic, &mut registry);
  diagnostics.extend(lint_diags);
  CheckResult { diagnostics, source, semantic }
}
