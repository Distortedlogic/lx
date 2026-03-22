use crate::ast::{BindTarget, Binding, SStmt, Stmt, UseKind, UseStmt};
use miette::SourceSpan;

use super::Checker;
use super::types::{self, Type};

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
      Stmt::TypeDef { name, variants, .. } => {
        let variant_names: Vec<String> = variants.iter().map(|(n, _)| n.clone()).collect();
        let variant_types: Vec<types::Variant> =
          variants.iter().map(|(n, arity)| types::Variant { name: n.clone(), fields: vec![Type::Unknown; *arity] }).collect();
        self.type_defs.insert(name.clone(), variant_names);
        let union_type = Type::Union { name: name.clone(), variants: variant_types };
        for (ctor_name, _) in variants {
          self.bind(ctor_name.clone(), union_type.clone());
        }
        Type::Unit
      },
      Stmt::TraitUnion(_) => Type::Unit,
      Stmt::TraitDecl(data) => {
        let fields: Vec<(String, Type)> = data
          .entries
          .iter()
          .filter_map(|e| if let crate::ast::TraitEntry::Field(f) = e { Some((f.name.clone(), super::named_to_type(&f.type_name))) } else { None })
          .collect();
        if !fields.is_empty() {
          self.trait_fields.insert(data.name.clone(), fields);
        }
        self.bind(data.name.clone(), Type::Unknown);
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
      Stmt::FieldUpdate { value, .. } => {
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
          self.check_import_conflict(name, span);
          self.bind(name.clone(), Type::Unknown);
        }
      },
      UseKind::Alias(alias) => {
        self.bind(alias.clone(), Type::Unknown);
      },
      UseKind::Selective(names) => {
        for name in names {
          self.check_import_conflict(name, span);
          self.bind(name.clone(), Type::Unknown);
        }
      },
    }
  }

  fn check_import_conflict(&mut self, name: &str, span: SourceSpan) {
    if let Some(existing) = self.import_sources.get(name) {
      self.emit_warning(format!("'{name}' already imported at offset {}", existing.offset()), span);
    } else {
      self.import_sources.insert(name.to_string(), span);
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
      BindTarget::Name(name) => {
        if b.mutable {
          self.mutables.insert(name.clone());
        }
        self.bind(name.clone(), val_type);
      },
      BindTarget::Reassign(name) => {
        if let Some(existing) = self.lookup(name) {
          let resolved = self.table.resolve_deep(&existing);
          if let Err(msg) = self.table.unify(&resolved, &val_type) {
            self.emit(format!("reassignment type mismatch: {msg}"), b.value.span);
          }
        }
      },
      BindTarget::Pattern(_) => {},
    }
  }
}
