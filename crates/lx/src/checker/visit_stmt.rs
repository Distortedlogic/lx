use std::collections::HashMap;

use crate::ast::{AstArena, BindTarget, Binding, Stmt, StmtId, TraitEntry, UseKind, UseStmt};
use crate::sym::Sym;
use miette::SourceSpan;

use crate::stdlib::STDLIB_ROOT;

use super::diagnostics::DiagnosticKind;
use super::semantic::DefKind;
use super::type_arena::TypeId;
use super::type_error::TypeContext;
use super::types::{Type, Variant};
use super::{Checker, DiagLevel, Diagnostic};

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
        if !td.type_params.is_empty() {
          let bounds: Vec<(Sym, Option<TypeId>)> = td.type_params.iter().map(|s| (*s, None)).collect();
          self.push_generic_scope(&bounds);
        }
        self.sem.add_definition(td.name, DefKind::TypeDef, span, false);
        let variant_names: Vec<Sym> = td.variants.iter().map(|(n, _)| *n).collect();
        let unknown = self.type_arena.unknown();
        let variant_types: Vec<Variant> = td.variants.iter().map(|(n, arity)| Variant { name: *n, fields: vec![unknown; *arity] }).collect();
        self.type_defs.insert(td.name, variant_names);
        let union_type = self.type_arena.alloc(Type::Union { name: td.name, variants: variant_types });
        for (ctor_name, _) in &td.variants {
          let def_id = self.sem.add_definition(*ctor_name, DefKind::TypeDef, span, false);
          self.sem.set_definition_type(def_id, union_type);
        }
        if !td.type_params.is_empty() {
          self.pop_generic_scope();
        }
        self.type_arena.unit()
      },
      Stmt::TraitUnion(_) => self.type_arena.unit(),
      Stmt::TraitDecl(data) => {
        let unknown = self.type_arena.unknown();
        let def_id = self.sem.add_definition(data.name, DefKind::TraitDef, span, false);
        let fields: Vec<(Sym, TypeId)> = data
          .entries
          .iter()
          .filter_map(|e| if let TraitEntry::Field(f) = e { Some((f.name, self.named_to_type(f.type_name.as_str()))) } else { None })
          .collect();
        if !fields.is_empty() {
          self.trait_fields.insert(data.name, fields);
        }
        self.sem.set_definition_type(def_id, unknown);
        self.type_arena.unit()
      },
      Stmt::ClassDecl(data) => {
        self.sem.add_definition(data.name, DefKind::ClassDef, span, false);
        for f in &data.fields {
          self.synth_expr(f.default);
        }
        for m in &data.methods {
          self.synth_expr(m.handler);
        }
        self.type_arena.unit()
      },
      Stmt::KeywordDecl(_) => self.type_arena.unit(),
      Stmt::ChannelDecl(name) => {
        self.sem.add_definition(name, DefKind::Binding, span, false);
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
    let module_name = u.path.last().copied();
    let is_std = u.path.first().is_some_and(|s| s.as_str() == STDLIB_ROOT);

    let std_data: Option<(Vec<(Sym, TypeId)>, super::type_arena::TypeArena)> = if is_std && u.path.len() >= 2 {
      let module_key = u.path[1].as_str();
      self.stdlib_sigs.get(module_key).map(|sig| {
        let pairs: Vec<_> = sig.bindings.iter().map(|(n, &t)| (*n, t)).collect();
        (pairs, sig.type_arena.clone())
      })
    } else {
      None
    };

    let std_translated: Option<HashMap<Sym, TypeId>> =
      std_data.map(|(pairs, src_arena)| pairs.into_iter().map(|(n, t)| (n, self.type_arena.copy_type(t, &src_arena))).collect());

    if is_std && u.path.len() >= 2 && std_translated.is_none() {
      let module_key = u.path[1].as_str();
      let available: Vec<&str> = self.stdlib_sigs.keys().map(|k| k.as_str()).collect();
      let suggestions = super::suggest::closest_matches(module_key, &available, 3);
      self.emit(DiagLevel::Error, DiagnosticKind::UnknownModule { name: format!("std.{module_key}"), suggestions }, span);
    }

    match &u.kind {
      UseKind::Whole => {
        if let Some(name) = module_name {
          self.check_import_conflict(name, span);
          let ty = if let Some(ref translated) = std_translated {
            let fields: Vec<_> = translated.iter().map(|(n, t)| (*n, *t)).collect();
            self.type_arena.alloc(Type::Record(fields))
          } else {
            self.record_type_for_module(name, unknown)
          };
          let def_id = self.sem.add_definition(name, DefKind::Import, span, false);
          self.sem.set_definition_type(def_id, ty);
        }
      },
      UseKind::Alias(alias) => {
        let ty = if let Some(ref translated) = std_translated {
          let fields: Vec<_> = translated.iter().map(|(n, t)| (*n, *t)).collect();
          self.type_arena.alloc(Type::Record(fields))
        } else {
          module_name.map(|m| self.record_type_for_module(m, unknown)).unwrap_or(unknown)
        };
        let def_id = self.sem.add_definition(*alias, DefKind::Import, span, false);
        self.sem.set_definition_type(def_id, ty);
      },
      UseKind::Selective(names) => {
        for name in names {
          self.check_import_conflict(*name, span);
          let ty = std_translated
            .as_ref()
            .and_then(|t| t.get(name).copied())
            .or_else(|| module_name.and_then(|m| self.translated_imports.get(&m).and_then(|t| t.get(name).copied())))
            .unwrap_or(unknown);
          let def_id = self.sem.add_definition(*name, DefKind::Import, span, false);
          self.sem.set_definition_type(def_id, ty);
          if std_translated.as_ref().is_some_and(|t| !t.contains_key(name)) {
            let export_names: Vec<&str> = std_translated.as_ref().map(|t| t.keys().map(|k| k.as_str()).collect()).unwrap_or_default();
            let suggestions = super::suggest::closest_matches(name.as_str(), &export_names, 5);
            self.emit(DiagLevel::Error, DiagnosticKind::UnknownImport { name: *name, module: module_name.unwrap_or(*name), suggestions }, span);
          } else if std_translated.is_none()
            && let Some(mod_sym) = module_name
            && let Some(sig) = self.import_signatures.get(&mod_sym)
            && !sig.bindings.contains_key(name)
            && !sig.types.contains_key(name)
            && !sig.traits.contains_key(name)
          {
            let mut export_names: Vec<&str> = sig.bindings.keys().map(|k| k.as_str()).collect();
            export_names.extend(sig.types.keys().map(|k| k.as_str()));
            export_names.extend(sig.traits.keys().map(|k| k.as_str()));
            let suggestions = super::suggest::closest_matches(name.as_str(), &export_names, 5);
            self.emit(DiagLevel::Error, DiagnosticKind::UnknownImport { name: *name, module: mod_sym, suggestions }, span);
          }
        }
      },
      UseKind::Tool { alias, .. } => {
        let def_id = self.sem.add_definition(*alias, DefKind::Import, span, false);
        self.sem.set_definition_type(def_id, unknown);
      },
    }
  }

  fn record_type_for_module(&mut self, module: Sym, fallback: TypeId) -> TypeId {
    let fields: Option<Vec<(Sym, TypeId)>> = self.translated_imports.get(&module).map(|t| t.iter().map(|(n, id)| (*n, *id)).collect());
    match fields {
      Some(f) if !f.is_empty() => self.type_arena.alloc(Type::Record(f)),
      _ => fallback,
    }
  }

  fn check_import_conflict(&mut self, name: Sym, span: SourceSpan) {
    if let Some(&existing) = self.import_sources.get(&name) {
      let original_span = self
        .sem
        .resolve_in_scope(name)
        .filter(|&id| matches!(self.sem.definitions[id.index()].kind, DefKind::Import))
        .map(|id| self.sem.definitions[id.index()].span)
        .unwrap_or(existing);
      let kind = DiagnosticKind::DuplicateImport { name, original_span };
      let secondary = vec![(original_span, "previously imported here".into())];
      let fix = kind.suggest_fix(span, &self.type_arena);
      let code = kind.code();
      self.diagnostics.push(Diagnostic { level: DiagLevel::Warning, kind, code, span, secondary, fix });
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
        let def_id = self.sem.add_definition(*name, DefKind::Binding, _span, b.mutable);
        self.sem.set_definition_type(def_id, val_type);
      },
      BindTarget::Reassign(name) => {
        if let Some(existing) = self.sem.lookup_type(*name) {
          let resolved = self.table.resolve(existing, &self.type_arena);
          let ctx = TypeContext::Binding { name: name.to_string() };
          if let Err(te) = self.table.unify_with_context(resolved, val_type, ctx, &mut self.type_arena) {
            self.emit_type_error(&te, val_span);
          }
        }
      },
      BindTarget::Pattern(pid) => {
        self.infer_pattern_bindings(*pid, val_type);
      },
    }
  }
}
