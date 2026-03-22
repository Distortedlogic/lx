use crate::ast::{BindTarget, Binding, SStmt, Stmt, StmtFieldUpdate, StmtTypeDef, UseKind, UseStmt};
use crate::sym::Sym;
use miette::SourceSpan;

use super::types::{self, Type, TypeContext};
use super::{Checker, DiagLevel};

impl Checker {
  pub(crate) fn check_stmts(&mut self, stmts: &[SStmt]) -> Type {
    let mut result = Type::Unit;
    for stmt in stmts {
      result = self.check_stmt(stmt);
    }
    result
  }

  pub(super) fn check_stmt(&mut self, stmt: &SStmt) -> Type {
    match &stmt.node {
      Stmt::Binding(b) => {
        self.check_binding(b);
        Type::Unit
      },
      Stmt::TypeDef(StmtTypeDef { name, variants, .. }) => {
        let variant_names: Vec<Sym> = variants.iter().map(|(n, _)| *n).collect();
        let variant_types: Vec<types::Variant> = variants.iter().map(|(n, arity)| types::Variant { name: *n, fields: vec![Type::Unknown; *arity] }).collect();
        self.type_defs.insert(*name, variant_names);
        let union_type = Type::Union { name: *name, variants: variant_types };
        for (ctor_name, _) in variants {
          self.bind(*ctor_name, union_type.clone());
        }
        Type::Unit
      },
      Stmt::TraitUnion(_) => Type::Unit,
      Stmt::TraitDecl(data) => {
        let fields: Vec<(Sym, Type)> = data
          .entries
          .iter()
          .filter_map(|e| if let crate::ast::TraitEntry::Field(f) = e { Some((f.name, super::named_to_type(f.type_name.as_str()))) } else { None })
          .collect();
        if !fields.is_empty() {
          self.trait_fields.insert(data.name, fields);
        }
        self.bind(data.name, Type::Unknown);
        Type::Unit
      },
      Stmt::ClassDecl(data) => {
        for f in &data.fields {
          self.synth(&f.default);
        }
        for m in &data.methods {
          self.synth(&m.handler);
        }
        Type::Unit
      },
      Stmt::FieldUpdate(StmtFieldUpdate { value, .. }) => {
        self.synth(value);
        Type::Unit
      },
      Stmt::Use(u) => {
        self.resolve_use(u, stmt.span);
        Type::Unit
      },
      Stmt::Expr(e) => self.synth(e),
    }
  }

  fn resolve_use(&mut self, u: &UseStmt, span: SourceSpan) {
    match &u.kind {
      UseKind::Whole => {
        if let Some(name) = u.path.last() {
          self.check_import_conflict(*name, span);
          self.bind(*name, Type::Unknown);
        }
      },
      UseKind::Alias(alias) => {
        self.bind(*alias, Type::Unknown);
      },
      UseKind::Selective(names) => {
        for name in names {
          self.check_import_conflict(*name, span);
          self.bind(*name, Type::Unknown);
        }
      },
    }
  }

  fn check_import_conflict(&mut self, name: Sym, span: SourceSpan) {
    if let Some(existing) = self.import_sources.get(&name) {
      self.emit(DiagLevel::Warning, format!("'{name}' already imported at offset {}", existing.offset()), span);
    } else {
      self.import_sources.insert(name, span);
    }
  }

  fn check_binding(&mut self, b: &Binding) {
    let val_type = self.synth(&b.value);
    if let Some(ann) = &b.type_ann {
      let expected = self.resolve_type_ann(ann);
      let binding_name = match &b.target {
        BindTarget::Name(n) | BindTarget::Reassign(n) => n.to_string(),
        BindTarget::Pattern(_) => "<pattern>".into(),
      };
      let ctx = TypeContext::Binding { name: binding_name };
      if let Err(te) = self.table.unify_with_context(&expected, &val_type, ctx) {
        self.emit_type_error(&te, b.value.span);
      }
    }
    match &b.target {
      BindTarget::Name(name) => {
        if b.mutable {
          self.mutables.insert(*name);
        }
        self.bind(*name, val_type);
      },
      BindTarget::Reassign(name) => {
        if let Some(existing) = self.lookup(*name) {
          let resolved = self.table.resolve(&existing);
          let ctx = TypeContext::Binding { name: name.to_string() };
          if let Err(te) = self.table.unify_with_context(&resolved, &val_type, ctx) {
            self.emit_type_error(&te, b.value.span);
          }
        }
      },
      BindTarget::Pattern(_) => {},
    }
  }
}
