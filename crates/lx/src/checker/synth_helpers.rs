use crate::sym::Sym;
use miette::SourceSpan;

use crate::ast::{BinOp, Expr, Literal, MapEntry, MatchArm, Param, Pattern, SExpr, SPattern, SType};

use super::types::Type;
use super::{Checker, DiagLevel};

impl Checker {
  pub(super) fn synth_func(&mut self, params: &[Param], ret_type: &Option<SType>, body: &SExpr) -> Type {
    self.push_scope();
    let mut param_types = Vec::new();
    for p in params {
      let ty = match &p.type_ann {
        Some(ann) => self.resolve_type_ann(ann),
        None => self.fresh(),
      };
      self.bind(p.name, ty.clone());
      param_types.push(ty);
    }
    let body_type = self.synth(body);
    if let Some(ret_ann) = ret_type {
      let expected = self.resolve_type_ann(ret_ann);
      if let Err(msg) = self.table.unify(&expected, &body_type) {
        self.emit(DiagLevel::Error, format!("return type mismatch: {msg}"), body.span);
      }
    }
    self.pop_scope();
    let mut result = match ret_type {
      Some(ann) => self.resolve_type_ann(ann),
      None => body_type,
    };
    for pt in param_types.into_iter().rev() {
      result = Type::Func { param: Box::new(pt), ret: Box::new(result) };
    }
    result
  }

  pub(super) fn bind_pattern_vars(&mut self, pat: &SPattern) {
    match &pat.node {
      Pattern::Bind(name) => {
        self.bind(*name, Type::Unknown);
      },
      Pattern::Constructor { args, .. } => {
        for arg in args {
          self.bind_pattern_vars(arg);
        }
      },
      Pattern::Tuple(pats) => {
        for p in pats {
          self.bind_pattern_vars(p);
        }
      },
      Pattern::List { elems, rest } => {
        for p in elems {
          self.bind_pattern_vars(p);
        }
        if let Some(name) = rest {
          self.bind(*name, Type::Unknown);
        }
      },
      Pattern::Record { fields, rest } => {
        for f in fields {
          if let Some(p) = &f.pattern {
            self.bind_pattern_vars(p);
          } else {
            self.bind(f.name, Type::Unknown);
          }
        }
        if let Some(name) = rest {
          self.bind(*name, Type::Unknown);
        }
      },
      Pattern::Literal(_) | Pattern::Wildcard => {},
    }
  }

  pub(super) fn check_mutable_captures(&mut self, expr: &SExpr, span: SourceSpan) {
    let fv = super::capture::free_vars(expr);
    for name in &fv {
      if self.mutables.contains(name) {
        self.emit(DiagLevel::Error, format!("cannot capture mutable binding `{name}` in concurrent context"), span);
      }
    }
  }

  pub(super) fn synth_apply(&mut self, func: &SExpr, arg: &SExpr) -> Type {
    if let (Expr::Ident(name) | Expr::TypeConstructor(name), Expr::Record(rec_fields)) = (&func.node, &arg.node)
      && let Some(fields) = self.trait_fields.get(name).cloned()
    {
      for (trait_name, trait_type) in &fields {
        for rf in rec_fields {
          if rf.name == Some(*trait_name) {
            let val_t = self.synth(&rf.value);
            if let Err(msg) = self.table.unify(trait_type, &val_t) {
              self.emit(DiagLevel::Error, format!("Trait field `{trait_name}` type mismatch: {msg}"), rf.value.span);
            }
          }
        }
      }
      return Type::Record(fields);
    }
    let ft = self.synth(func);
    let _ = self.synth(arg);
    match self.table.resolve(&ft) {
      Type::Func { ret, .. } => *ret,
      _ => Type::Unknown,
    }
  }

  pub(super) fn synth_match(&mut self, scrutinee: &SExpr, arms: &[MatchArm], span: SourceSpan) -> Type {
    let scrut_t = self.synth(scrutinee);
    let resolved_scrut = self.table.resolve(&scrut_t);
    if let Type::Union { ref name, ref variants } = resolved_scrut {
      let variant_names: Vec<Sym> = variants.iter().map(|v| v.name).collect();
      let missing = super::exhaust::check_exhaustiveness(*name, &variant_names, arms);
      for v in &missing {
        self.emit(DiagLevel::Warning, format!("non-exhaustive match on {name}: missing {v}"), span);
      }
    }
    let result = self.fresh();
    for arm in arms {
      self.push_scope();
      self.bind_pattern_vars(&arm.pattern);
      if let Some(guard) = &arm.guard {
        self.synth(guard);
      }
      let body_t = self.synth(&arm.body);
      self.pop_scope();
      if let Err(msg) = self.table.unify(&result, &body_t) {
        self.emit(DiagLevel::Error, format!("match arm type mismatch: {msg}"), arm.body.span);
      }
    }
    self.table.resolve(&result)
  }

  pub(super) fn synth_map(&mut self, entries: &[MapEntry]) -> Type {
    if entries.is_empty() {
      return Type::Map { key: Box::new(self.fresh()), value: Box::new(self.fresh()) };
    }
    let mut key_t = self.fresh();
    let mut val_t = self.fresh();
    for e in entries {
      if let Some(k) = &e.key {
        let kt = self.synth(k);
        let _ = self.table.unify(&key_t, &kt);
        key_t = self.table.resolve(&key_t);
      }
      let vt = self.synth(&e.value);
      let _ = self.table.unify(&val_t, &vt);
      val_t = self.table.resolve(&val_t);
    }
    Type::Map { key: Box::new(key_t), value: Box::new(val_t) }
  }

  pub(super) fn synth_binary(&mut self, op: &BinOp, lt: &Type, rt: &Type, span: SourceSpan) -> Type {
    let lt = self.table.resolve(lt);
    let rt = self.table.resolve(rt);
    match op {
      BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::IntDiv => self.table.unify(&lt, &rt).unwrap_or(Type::Unknown),
      BinOp::Concat => Type::Str,
      BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => Type::Bool,
      BinOp::And | BinOp::Or => {
        if lt != Type::Bool && lt != Type::Unknown {
          self.emit(DiagLevel::Error, "logical operator requires Bool".into(), span);
        }
        Type::Bool
      },
      BinOp::Range | BinOp::RangeInclusive => Type::List(Box::new(Type::Int)),
    }
  }
}

pub(super) fn synth_literal(lit: &Literal) -> Type {
  match lit {
    Literal::Int(_) => Type::Int,
    Literal::Float(_) => Type::Float,
    Literal::Str(_) | Literal::RawStr(_) => Type::Str,
    Literal::Bool(_) => Type::Bool,
    Literal::Unit => Type::Unit,
  }
}
