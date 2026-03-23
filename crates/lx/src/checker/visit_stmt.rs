use crate::ast::{AstArena, BindTarget, Binding, Stmt, StmtId, TraitEntry, UseKind, UseStmt};
use crate::sym::Sym;
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::symbol_table::DefKind;
use super::types::{self, Type};
use super::unification::TypeContext;
use super::{Checker, DiagLevel, Diagnostic, named_to_type};

impl Checker<'_> {
  pub(crate) fn check_stmts(&mut self, stmts: &[StmtId]) -> Type {
    let mut result = Type::Unit;
    let arena = self.arena;
    for &sid in stmts {
      result = self.check_stmt(sid, arena);
    }
    result
  }

  pub(super) fn check_stmt(&mut self, sid: StmtId, arena: &AstArena) -> Type {
    let span = arena.stmt_span(sid);
    let stmt = arena.stmt(sid).clone();
    match stmt {
      Stmt::Binding(b) => {
        self.check_binding(&b, span);
        Type::Unit
      },
      Stmt::TypeDef(td) => {
        let variant_names: Vec<Sym> = td.variants.iter().map(|(n, _)| *n).collect();
        let variant_types: Vec<types::Variant> =
          td.variants.iter().map(|(n, arity)| types::Variant { name: *n, fields: vec![Type::Unknown; *arity] }).collect();
        self.type_defs.insert(td.name, variant_names);
        let union_type = Type::Union { name: td.name, variants: variant_types };
        for (ctor_name, _) in &td.variants {
          self.bind(*ctor_name, union_type.clone());
        }
        Type::Unit
      },
      Stmt::TraitUnion(_) => Type::Unit,
      Stmt::TraitDecl(data) => {
        let fields: Vec<(Sym, Type)> =
          data.entries.iter().filter_map(|e| if let TraitEntry::Field(f) = e { Some((f.name, named_to_type(f.type_name.as_str()))) } else { None }).collect();
        if !fields.is_empty() {
          self.trait_fields.insert(data.name, fields);
        }
        self.bind(data.name, Type::Unknown);
        Type::Unit
      },
      Stmt::ClassDecl(data) => {
        for f in &data.fields {
          self.synth_expr(f.default);
        }
        for m in &data.methods {
          self.synth_expr(m.handler);
        }
        Type::Unit
      },
      Stmt::FieldUpdate(fu) => {
        self.synth_expr(fu.value);
        Type::Unit
      },
      Stmt::Use(u) => {
        self.resolve_use(&u, span);
        Type::Unit
      },
      Stmt::Expr(e) => self.synth_expr(e),
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
    if let Some(&existing) = self.import_sources.get(&name) {
      let original_span = self.symbols.resolve(name).filter(|def| matches!(def.kind, DefKind::Import)).map(|def| def.span).unwrap_or(existing);
      let kind = DiagnosticKind::DuplicateImport { name, original_span };
      let secondary = vec![(original_span, "previously imported here".into())];
      let fix = kind.suggest_fix(span);
      self.diagnostics.push(Diagnostic { level: DiagLevel::Warning, kind, span, secondary, fix });
    } else {
      self.import_sources.insert(name, span);
    }
  }

  fn check_binding(&mut self, b: &Binding, _span: SourceSpan) {
    let val_span = self.arena.expr_span(b.value);
    let val_type = if let Some(ann) = b.type_ann {
      let expected = self.resolve_type_ann(ann);
      let ann_span = self.arena.type_expr_span(ann);
      let checked = self.check_expr(b.value, &expected);
      let binding_name = match &b.target {
        BindTarget::Name(n) | BindTarget::Reassign(n) => n.to_string(),
        BindTarget::Pattern(_) => "<pattern>".into(),
      };
      let ctx = TypeContext::Binding { name: binding_name };
      match self.table.unify_with_context(&expected, &checked, ctx) {
        Ok(t) => t,
        Err(mut te) => {
          te.expected_origin = Some(ann_span);
          self.emit_type_error(&te, val_span);
          checked
        },
      }
    } else {
      self.synth_expr(b.value)
    };
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
            self.emit_type_error(&te, val_span);
          }
        }
      },
      BindTarget::Pattern(_) => {},
    }
  }
}
