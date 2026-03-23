use crate::ast::{AstArena, BindTarget, Binding, Pattern, PatternId, Stmt, StmtId, TraitEntry, UseKind, UseStmt};
use crate::sym::Sym;
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::symbol_table::DefKind;
use super::type_arena::TypeId;
use super::types::{Type, Variant};
use super::unification::TypeContext;
use super::{Checker, DiagLevel, Diagnostic};

impl Checker<'_> {
  pub(crate) fn bind_pattern_names_to_symbols(&mut self, pid: PatternId) {
    let span = self.arena.pattern_span(pid);
    match self.arena.pattern(pid).clone() {
      Pattern::Bind(name) => {
        self.symbols.define(name, DefKind::PatternBind, span);
      },
      Pattern::Constructor(c) => {
        for arg in &c.args {
          self.bind_pattern_names_to_symbols(*arg);
        }
      },
      Pattern::Tuple(pats) => {
        for p in &pats {
          self.bind_pattern_names_to_symbols(*p);
        }
      },
      Pattern::List(pl) => {
        for p in &pl.elems {
          self.bind_pattern_names_to_symbols(*p);
        }
        if let Some(rest) = pl.rest {
          self.symbols.define(rest, DefKind::PatternBind, span);
        }
      },
      Pattern::Record(pr) => {
        for f in &pr.fields {
          if let Some(p) = f.pattern {
            self.bind_pattern_names_to_symbols(p);
          } else {
            self.symbols.define(f.name, DefKind::PatternBind, span);
          }
        }
        if let Some(rest) = pr.rest {
          self.symbols.define(rest, DefKind::PatternBind, span);
        }
      },
      Pattern::Literal(_) | Pattern::Wildcard => {},
    }
  }
}

impl Checker<'_> {
  pub(crate) fn check_stmts(&mut self, stmts: &[StmtId]) -> TypeId {
    let mut result = self.type_arena.unit();
    let arena = self.arena;
    for &sid in stmts {
      result = self.check_stmt(sid, arena);
    }
    result
  }

  pub(super) fn check_stmt(&mut self, sid: StmtId, arena: &AstArena) -> TypeId {
    let span = arena.stmt_span(sid);
    let stmt = arena.stmt(sid).clone();
    match stmt {
      Stmt::Binding(b) => {
        self.check_binding(&b, span);
        self.type_arena.unit()
      },
      Stmt::TypeDef(td) => {
        self.symbols.define(td.name, DefKind::TypeDef, span);
        let variant_names: Vec<Sym> = td.variants.iter().map(|(n, _)| *n).collect();
        let unknown = self.type_arena.unknown();
        let variant_types: Vec<Variant> = td.variants.iter().map(|(n, arity)| Variant { name: *n, fields: vec![unknown; *arity] }).collect();
        self.type_defs.insert(td.name, variant_names);
        let union_type = self.type_arena.alloc(Type::Union { name: td.name, variants: variant_types });
        for (ctor_name, _) in &td.variants {
          self.symbols.define(*ctor_name, DefKind::TypeDef, span);
          self.symbols.set_type(*ctor_name, union_type);
        }
        self.type_arena.unit()
      },
      Stmt::TraitUnion(_) => self.type_arena.unit(),
      Stmt::TraitDecl(data) => {
        self.symbols.define(data.name, DefKind::TraitDef, span);
        let fields: Vec<(Sym, TypeId)> = data
          .entries
          .iter()
          .filter_map(|e| if let TraitEntry::Field(f) = e { Some((f.name, self.named_to_type(f.type_name.as_str()))) } else { None })
          .collect();
        if !fields.is_empty() {
          self.trait_fields.insert(data.name, fields);
        }
        self.symbols.set_type(data.name, self.type_arena.unknown());
        self.type_arena.unit()
      },
      Stmt::ClassDecl(data) => {
        self.symbols.define(data.name, DefKind::ClassDef, span);
        for f in &data.fields {
          self.synth_expr(f.default);
        }
        for m in &data.methods {
          self.synth_expr(m.handler);
        }
        self.type_arena.unit()
      },
      Stmt::FieldUpdate(fu) => {
        self.synth_expr(fu.value);
        self.type_arena.unit()
      },
      Stmt::Use(u) => {
        self.resolve_use(&u, span);
        self.type_arena.unit()
      },
      Stmt::Expr(e) => self.synth_expr(e),
    }
  }

  fn resolve_use(&mut self, u: &UseStmt, span: SourceSpan) {
    let unknown = self.type_arena.unknown();
    match &u.kind {
      UseKind::Whole => {
        if let Some(name) = u.path.last() {
          self.check_import_conflict(*name, span);
          self.symbols.define(*name, DefKind::Import, span);
          self.symbols.set_type(*name, unknown);
        }
      },
      UseKind::Alias(alias) => {
        self.symbols.define(*alias, DefKind::Import, span);
        self.symbols.set_type(*alias, unknown);
      },
      UseKind::Selective(names) => {
        for name in names {
          self.check_import_conflict(*name, span);
          self.symbols.define(*name, DefKind::Import, span);
          self.symbols.set_type(*name, unknown);
        }
      },
    }
  }

  fn check_import_conflict(&mut self, name: Sym, span: SourceSpan) {
    if let Some(&existing) = self.import_sources.get(&name) {
      let original_span = self.symbols.resolve(name).filter(|def| matches!(def.kind, DefKind::Import)).map(|def| def.span).unwrap_or(existing);
      let kind = DiagnosticKind::DuplicateImport { name, original_span };
      let secondary = vec![(original_span, "previously imported here".into())];
      let fix = kind.suggest_fix(span, &self.type_arena);
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
      let checked = self.check_expr(b.value, expected);
      let binding_name = match &b.target {
        BindTarget::Name(n) | BindTarget::Reassign(n) => n.to_string(),
        BindTarget::Pattern(_) => "<pattern>".into(),
      };
      let ctx = TypeContext::Binding { name: binding_name };
      match self.table.unify_with_context(expected, checked, ctx, &mut self.type_arena) {
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
        self.symbols.define(*name, DefKind::Binding, _span);
        if b.mutable {
          self.mutables.insert(*name);
        }
        self.symbols.set_type(*name, val_type);
      },
      BindTarget::Reassign(name) => {
        if let Some(existing) = self.symbols.lookup_type(*name) {
          let resolved = self.table.resolve(existing, &self.type_arena);
          let ctx = TypeContext::Binding { name: name.to_string() };
          if let Err(te) = self.table.unify_with_context(resolved, val_type, ctx, &mut self.type_arena) {
            self.emit_type_error(&te, val_span);
          }
        }
      },
      BindTarget::Pattern(pid) => {
        self.bind_pattern_names_to_symbols(*pid);
      },
    }
  }
}
