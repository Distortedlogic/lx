use std::ops::ControlFlow;

use crate::ast::{
  BinOp, Expr, ExprId, Literal, MapEntry, MatchArm, Param, Pattern, PatternConstructor, PatternId, PatternList, PatternRecord, RecordField, TypeExprId,
};
use crate::sym::{self, Sym};
use crate::visitor::dispatch_expr;
use miette::SourceSpan;

use super::capture::free_vars;
use super::diagnostics::DiagnosticKind;
use super::exhaust::{check_exhaustiveness, check_exhaustiveness_no_variants};
use super::types::Type;
use super::unification::TypeContext;
use super::{Checker, DiagLevel};

impl Checker<'_> {
  pub(super) fn synth_expr(&mut self, eid: ExprId) -> Type {
    let arena = self.arena;
    let expr = arena.expr(eid);
    let espan = arena.expr_span(eid);
    match dispatch_expr(self, expr, espan, arena) {
      ControlFlow::Continue(()) | ControlFlow::Break(()) => {},
    }
    self.pop_type()
  }

  pub(super) fn synth_func_type(&mut self, params: &[Param], ret_type: &Option<TypeExprId>, body: ExprId) -> Type {
    self.push_scope();
    let mut param_types = Vec::new();
    for p in params {
      let ty = match p.type_ann {
        Some(ann) => self.resolve_type_ann(ann),
        None => self.fresh(),
      };
      self.bind(p.name, ty.clone());
      param_types.push(ty);
    }
    let body_span = self.arena.expr_span(body);
    let body_type = self.synth_expr(body);
    let ret_ann_span = ret_type.map(|ann| self.arena.type_expr_span(ann));
    if let Some(ret_ann) = ret_type {
      let expected = self.resolve_type_ann(*ret_ann);
      let ctx = TypeContext::FuncReturn { func_name: "anonymous".into() };
      match self.table.unify_with_context(&expected, &body_type, ctx) {
        Ok(_) => {},
        Err(mut te) => {
          te.expected_origin = ret_ann_span;
          self.emit_type_error(&te, body_span);
        },
      }
    }
    self.pop_scope();
    let ret = match ret_type {
      Some(ann) => self.resolve_type_ann(*ann),
      None => body_type,
    };
    Type::Func { params: param_types, ret: Box::new(ret) }
  }

  pub(super) fn bind_pattern_vars(&mut self, pid: PatternId) {
    match self.arena.pattern(pid).clone() {
      Pattern::Bind(name) => {
        self.bind(name, Type::Unknown);
      },
      Pattern::Constructor(PatternConstructor { args, .. }) => {
        for arg in &args {
          self.bind_pattern_vars(*arg);
        }
      },
      Pattern::Tuple(pats) => {
        for p in &pats {
          self.bind_pattern_vars(*p);
        }
      },
      Pattern::List(PatternList { elems, rest }) => {
        for p in &elems {
          self.bind_pattern_vars(*p);
        }
        if let Some(name) = rest {
          self.bind(name, Type::Unknown);
        }
      },
      Pattern::Record(PatternRecord { fields, rest }) => {
        for f in &fields {
          if let Some(p) = f.pattern {
            self.bind_pattern_vars(p);
          } else {
            self.bind(f.name, Type::Unknown);
          }
        }
        if let Some(name) = rest {
          self.bind(name, Type::Unknown);
        }
      },
      Pattern::Literal(_) | Pattern::Wildcard => {},
    }
  }

  pub(super) fn check_mutable_captures(&mut self, eid: ExprId, span: SourceSpan) {
    let fv = free_vars(eid, self.arena);
    for name in &fv {
      if self.is_mutable(*name) {
        self.emit(DiagLevel::Error, DiagnosticKind::MutableCaptureInConcurrent { name: *name }, span);
      }
    }
  }

  pub(super) fn synth_apply_type(&mut self, func: ExprId, arg: ExprId) -> Type {
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
            if let Err(te) = self.table.unify_with_context(trait_type, &val_t, ctx) {
              self.emit_type_error(&te, val_span);
            }
          }
        }
      }
      return Type::Record(fields);
    }
    let ft = self.synth_expr(func);
    let arg_span = self.arena.expr_span(arg);
    let arg_t = self.synth_expr(arg);
    match self.table.resolve(&ft) {
      Type::Func { params, ret } => {
        if params.is_empty() {
          return *ret;
        }
        let ctx = TypeContext::FuncArg { func_name: "apply".into(), param_name: "arg".into(), param_idx: 0 };
        if let Err(te) = self.table.unify_with_context(&params[0], &arg_t, ctx) {
          self.emit_type_error(&te, arg_span);
          return Type::Error;
        }
        if params.len() == 1 { *ret } else { Type::Func { params: params[1..].to_vec(), ret } }
      },
      Type::Error => Type::Error,
      _ => Type::Unknown,
    }
  }

  pub(super) fn synth_match_type(&mut self, scrutinee: ExprId, arms: &[MatchArm], span: SourceSpan) -> Type {
    let scrut_t = self.synth_expr(scrutinee);
    let resolved_scrut = self.table.resolve(&scrut_t);
    self.check_match_exhaustiveness(&resolved_scrut, arms, span);
    let result = self.fresh();
    for (idx, arm) in arms.iter().enumerate() {
      self.push_scope();
      self.bind_pattern_vars(arm.pattern);
      if let Some(guard) = arm.guard {
        self.synth_expr(guard);
      }
      let body_span = self.arena.expr_span(arm.body);
      let body_t = self.synth_expr(arm.body);
      self.pop_scope();
      let ctx = TypeContext::MatchArm { arm_idx: idx };
      if let Err(te) = self.table.unify_with_context(&result, &body_t, ctx) {
        self.emit_type_error(&te, body_span);
      }
    }
    self.table.resolve(&result)
  }

  fn check_match_exhaustiveness(&mut self, scrut_type: &Type, arms: &[MatchArm], span: SourceSpan) {
    let missing = match scrut_type {
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
    let type_name = match scrut_type {
      Type::Union { name, .. } => *name,
      _ => sym::intern(&format!("{scrut_type}")),
    };
    for pat in &missing {
      self.emit(DiagLevel::Warning, DiagnosticKind::NonExhaustiveMatch { type_name, missing_pattern: pat.clone() }, span);
    }
  }

  pub(super) fn synth_map_type(&mut self, entries: &[MapEntry]) -> Type {
    if entries.is_empty() {
      return Type::Map { key: Box::new(self.fresh()), value: Box::new(self.fresh()) };
    }
    let mut key_t = self.fresh();
    let mut val_t = self.fresh();
    for entry in entries {
      match entry {
        MapEntry::Keyed { key, value } => {
          let kt = self.synth_expr(*key);
          let key_span = self.arena.expr_span(*key);
          if let Err(te) = self.table.unify_with_context(&key_t, &kt, TypeContext::General) {
            self.emit_type_error(&te, key_span);
          }
          key_t = self.table.resolve(&key_t);
          let vt = self.synth_expr(*value);
          let val_span = self.arena.expr_span(*value);
          if let Err(te) = self.table.unify_with_context(&val_t, &vt, TypeContext::General) {
            self.emit_type_error(&te, val_span);
          }
          val_t = self.table.resolve(&val_t);
        },
        MapEntry::Spread(value) => {
          let vt = self.synth_expr(*value);
          let val_span = self.arena.expr_span(*value);
          if let Err(te) = self.table.unify_with_context(&val_t, &vt, TypeContext::General) {
            self.emit_type_error(&te, val_span);
          }
          val_t = self.table.resolve(&val_t);
        },
      }
    }
    Type::Map { key: Box::new(key_t), value: Box::new(val_t) }
  }

  pub(super) fn synth_binary_type(&mut self, op: &BinOp, lt: &Type, rt: &Type, span: SourceSpan) -> Type {
    let lt = self.table.resolve(lt);
    let rt = self.table.resolve(rt);
    match op {
      BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::IntDiv => {
        let ctx = TypeContext::BinaryOp { op: format!("{op:?}") };
        match self.table.unify_with_context(&lt, &rt, ctx) {
          Ok(t) => t,
          Err(te) => {
            self.emit_type_error(&te, span);
            Type::Error
          },
        }
      },
      BinOp::Concat => Type::Str,
      BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => Type::Bool,
      BinOp::And | BinOp::Or => {
        if lt != Type::Bool && lt != Type::Unknown && lt != Type::Error {
          self.emit(DiagLevel::Error, DiagnosticKind::LogicalOpRequiresBool, span);
        }
        Type::Bool
      },
      BinOp::Range | BinOp::RangeInclusive => Type::List(Box::new(Type::Int)),
    }
  }

  pub(super) fn synth_literal_type(lit: &Literal) -> Type {
    match lit {
      Literal::Int(_) => Type::Int,
      Literal::Float(_) => Type::Float,
      Literal::Str(_) | Literal::RawStr(_) => Type::Str,
      Literal::Bool(_) => Type::Bool,
      Literal::Unit => Type::Unit,
    }
  }
}
