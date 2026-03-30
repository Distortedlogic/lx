use std::collections::HashMap;

use lx_ast::ast::{Expr, ExprId, FieldKind, Literal, MapEntry, MatchArm, Param, RecordField, StrPart, TypeExprId};
use lx_span::sym::{self, Sym};
use miette::SourceSpan;

use super::capture::free_vars;
use super::diagnostics::DiagnosticKind;
use super::exhaust::{check_exhaustiveness, check_exhaustiveness_no_variants};
use super::narrowing;
use super::semantic::{DefKind, ScopeKind};
use super::type_arena::TypeId;
use super::type_error::{TypeContext, TypeError};
use super::types::Type;
use super::{Checker, DiagLevel};

impl Checker<'_> {
  pub(super) fn synth_func_type(&mut self, type_params: &[Sym], params: &[Param], ret_type: &Option<TypeExprId>, body: ExprId) -> TypeId {
    let func_span = self.arena.expr_span(body);
    if !type_params.is_empty() {
      let bounds: Vec<(Sym, Option<TypeId>)> = type_params.iter().map(|s| (*s, None)).collect();
      self.push_generic_scope(&bounds);
    }
    self.sem.push_scope(ScopeKind::Function, func_span);
    let mut param_types = Vec::new();
    for p in params {
      let ty = match p.type_ann {
        Some(ann) => self.resolve_type_ann(ann),
        None => self.fresh(),
      };
      let def_id = self.sem.add_definition(p.name, DefKind::FuncParam, func_span, false);
      self.sem.set_definition_type(def_id, ty);
      param_types.push(ty);
    }
    let body_span = self.arena.expr_span(body);
    let body_type = self.synth_expr(body);
    let ret_ann_span = ret_type.map(|ann| self.arena.type_expr_span(ann));
    if let Some(ret_ann) = ret_type {
      let expected = self.resolve_type_ann(*ret_ann);
      let ctx = TypeContext::FuncReturn { func_name: "anonymous".into() };
      match self.table.unify_with_context(expected, body_type, ctx, &mut self.type_arena) {
        Ok(_) => {},
        Err(mut te) => {
          te.expected_origin = ret_ann_span;
          self.emit_type_error(&te, body_span);
        },
      }
    }
    self.sem.pop_scope();
    if !type_params.is_empty() {
      self.pop_generic_scope();
    }
    let ret = match ret_type {
      Some(ann) => self.resolve_type_ann(*ann),
      None => body_type,
    };
    let mut func_type = ret;
    for &p in param_types.iter().rev() {
      func_type = self.type_arena.alloc(Type::Func { param: p, ret: func_type });
    }
    func_type
  }

  pub(super) fn check_mutable_captures(&mut self, eid: ExprId, span: SourceSpan) {
    let fv = free_vars(eid, self.arena);
    for name in &fv {
      if self.is_mutable(*name) {
        self.emit(DiagLevel::Error, DiagnosticKind::MutableCaptureInConcurrent { name: *name }, span);
      }
    }
  }

  pub(super) fn synth_apply_type(&mut self, func: ExprId, arg: ExprId) -> TypeId {
    let func_expr = self.arena.expr(func).clone();
    let arg_expr = self.arena.expr(arg).clone();
    if let (Expr::Ident(name) | Expr::TypeConstructor(name), Expr::Record(rec_fields)) = (&func_expr, &arg_expr)
      && let Some(fields) = self.trait_fields.get(name).cloned()
    {
      for (trait_name, trait_type) in &fields {
        for rf in rec_fields {
          if let RecordField::Named { name, value } = rf
            && *name == *trait_name
          {
            let val_span = self.arena.expr_span(*value);
            let val_t = self.synth_expr(*value);
            let ctx = TypeContext::RecordField { field_name: trait_name.to_string() };
            if let Err(te) = self.table.unify_with_context(*trait_type, val_t, ctx, &mut self.type_arena) {
              self.emit_type_error(&te, val_span);
            }
          }
        }
      }
      return self.type_arena.alloc(Type::Record(fields));
    }
    let func_name = match &func_expr {
      Expr::Ident(name) => name.to_string(),
      Expr::FieldAccess(fa) => match &fa.field {
        FieldKind::Named(name) => name.to_string(),
        _ => "<fn>".into(),
      },
      _ => "<fn>".into(),
    };
    let ft = self.synth_expr(func);
    let arg_span = self.arena.expr_span(arg);
    let resolved = self.table.resolve(ft, &self.type_arena);
    let sig_display = self.type_arena.display(resolved);
    match self.type_arena.get(resolved).clone() {
      Type::Func { param, ret } => {
        let type_params = self.collect_params(resolved);
        let (inst_param, inst_ret) = if type_params.is_empty() {
          (param, ret)
        } else {
          let subst: HashMap<Sym, TypeId> = type_params.into_iter().map(|name| (name, self.fresh())).collect();
          (self.substitute(param, &subst), self.substitute(ret, &subst))
        };
        let arg_t = self.check_expr(arg, inst_param);
        let ctx = TypeContext::FuncArg { func_name, param_name: "arg".into(), param_idx: 0 };
        if let Err(te) = self.table.unify_with_context(inst_param, arg_t, ctx, &mut self.type_arena) {
          let func_span = self.arena.expr_span(func);
          let mut diag = self.make_type_error_diagnostic(&te, arg_span);
          diag.secondary.push((func_span, format!("signature: {sig_display}")));
          self.diagnostics.push(diag);
          return self.type_arena.error();
        }
        inst_ret
      },
      Type::Error => {
        self.synth_expr(arg);
        self.type_arena.error()
      },
      _ => {
        self.synth_expr(arg);
        let param = self.type_arena.unknown();
        let ret = self.type_arena.unknown();
        let expected = self.type_arena.alloc(Type::Func { param, ret });
        let func_span = self.arena.expr_span(func);
        let kind = DiagnosticKind::TypeMismatch { error: TypeError { expected, found: resolved, context: TypeContext::General, expected_origin: None } };
        self.emit(DiagLevel::Error, kind, func_span);
        self.type_arena.unknown()
      },
    }
  }

  pub(super) fn synth_match_type(&mut self, scrutinee: ExprId, arms: &[MatchArm], span: SourceSpan) -> TypeId {
    let scrut_t = self.synth_expr(scrutinee);
    let resolved_scrut = self.table.resolve(scrut_t, &self.type_arena);
    self.check_match_exhaustiveness(resolved_scrut, arms, span);
    let result = self.fresh();
    for (idx, arm) in arms.iter().enumerate() {
      let arm_span = self.arena.pattern_span(arm.pattern);
      self.sem.push_scope(ScopeKind::MatchArm, arm_span);
      self.infer_pattern_bindings(arm.pattern, resolved_scrut);
      self.narrowing.push();
      if let Expr::Ident(scrut_name) = self.arena.expr(scrutinee) {
        let pattern = self.arena.pattern(arm.pattern).clone();
        let resolved_type = self.type_arena.get(resolved_scrut).clone();
        let narrowed = narrowing::compute_narrowed_type(&pattern, resolved_scrut, &mut self.type_arena, &resolved_type);
        self.narrowing.narrow(*scrut_name, narrowed);
      }
      if let Some(guard) = arm.guard {
        self.synth_expr(guard);
      }
      let body_span = self.arena.expr_span(arm.body);
      let body_t = self.check_expr(arm.body, result);
      self.narrowing.pop();
      self.sem.pop_scope();
      let ctx = TypeContext::MatchArm { arm_idx: idx };
      if let Err(te) = self.table.unify_with_context(result, body_t, ctx, &mut self.type_arena) {
        self.emit_type_error(&te, body_span);
      }
    }
    self.table.resolve(result, &self.type_arena)
  }

  pub(super) fn check_match_exhaustiveness(&mut self, scrut_type: TypeId, arms: &[MatchArm], span: SourceSpan) {
    let scrut = self.type_arena.get(scrut_type).clone();
    let missing = match &scrut {
      Type::Union { name, variants } => {
        let variant_info: Vec<(Sym, usize)> = variants.iter().map(|v| (v.name, v.fields.len())).collect();
        check_exhaustiveness(*name, &variant_info, arms, self.arena)
      },
      Type::Bool => {
        let type_name = sym::intern("Bool");
        check_exhaustiveness_no_variants(type_name, arms, self.arena)
      },
      Type::Tuple(_) => {
        let type_name = sym::intern("Tuple");
        check_exhaustiveness_no_variants(type_name, arms, self.arena)
      },
      Type::Unit => {
        let type_name = sym::intern("()");
        check_exhaustiveness_no_variants(type_name, arms, self.arena)
      },
      Type::List(_) => {
        let type_name = sym::intern("List");
        let nil = sym::intern("Nil");
        let cons = sym::intern("Cons");
        let variants = vec![(nil, 0), (cons, 2)];
        check_exhaustiveness(type_name, &variants, arms, self.arena)
      },
      _ => return,
    };
    let type_name = match &scrut {
      Type::Union { name, .. } => *name,
      _ => sym::intern(&self.type_arena.display(scrut_type)),
    };
    for pat in &missing {
      self.emit(DiagLevel::Warning, DiagnosticKind::NonExhaustiveMatch { type_name, missing_pattern: pat.clone() }, span);
    }
  }

  pub(super) fn synth_map_type(&mut self, entries: &[MapEntry]) -> TypeId {
    if entries.is_empty() {
      let key = self.fresh();
      let value = self.fresh();
      return self.type_arena.alloc(Type::Map { key, value });
    }
    let mut key_t = self.fresh();
    let mut val_t = self.fresh();
    for entry in entries {
      match entry {
        MapEntry::Keyed { key, value } => {
          let kt = self.synth_expr(*key);
          let key_span = self.arena.expr_span(*key);
          if let Err(te) = self.table.unify_with_context(key_t, kt, TypeContext::General, &mut self.type_arena) {
            self.emit_type_error(&te, key_span);
          }
          key_t = self.table.resolve(key_t, &self.type_arena);
          let vt = self.synth_expr(*value);
          let val_span = self.arena.expr_span(*value);
          if let Err(te) = self.table.unify_with_context(val_t, vt, TypeContext::General, &mut self.type_arena) {
            self.emit_type_error(&te, val_span);
          }
          val_t = self.table.resolve(val_t, &self.type_arena);
        },
        MapEntry::Spread(value) => {
          let vt = self.synth_expr(*value);
          let val_span = self.arena.expr_span(*value);
          if let Err(te) = self.table.unify_with_context(val_t, vt, TypeContext::General, &mut self.type_arena) {
            self.emit_type_error(&te, val_span);
          }
          val_t = self.table.resolve(val_t, &self.type_arena);
        },
      }
    }
    self.type_arena.alloc(Type::Map { key: key_t, value: val_t })
  }

  pub(super) fn synth_literal(&mut self, lit: &Literal) -> TypeId {
    if let Literal::Str(parts) = lit {
      for part in parts {
        if let StrPart::Interp(eid) = part {
          self.synth_expr(*eid);
        }
      }
    }
    self.synth_literal_type(lit)
  }
}
