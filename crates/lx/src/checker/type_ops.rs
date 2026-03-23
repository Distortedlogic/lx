use crate::ast::{BinOp, Expr, ExprId, FieldKind, ListElem, Literal, RecordField, UnaryOp};
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::types::Type;
use super::unification::TypeContext;
use super::{Checker, DiagLevel};

impl Checker<'_> {
  pub(super) fn synth_expr(&mut self, eid: ExprId) -> Type {
    let arena = self.arena;
    let expr = arena.expr(eid).clone();
    let span = arena.expr_span(eid);
    match expr {
      Expr::Literal(lit) => self.synth_literal(&lit),
      Expr::Ident(name) => self.lookup(name).unwrap_or(Type::Unknown),
      Expr::TypeConstructor(_) => Type::Unknown,
      Expr::Binary(binary) => {
        let lt = self.synth_expr(binary.left);
        let rt = self.synth_expr(binary.right);
        self.synth_binary_type(&binary.op, &lt, &rt, span)
      },
      Expr::Unary(unary) => self.synth_unary(unary.op, unary.operand, span),
      Expr::Pipe(_) => Type::Todo,
      Expr::Apply(apply) => self.synth_apply_type(apply.func, apply.arg),
      Expr::Section(_) => Type::Todo,
      Expr::FieldAccess(fa) => {
        self.synth_expr(fa.expr);
        if let FieldKind::Computed(c) = fa.field {
          self.synth_expr(c);
        }
        Type::Todo
      },
      Expr::Block(stmts) => self.check_stmts(&stmts),
      Expr::Tuple(elems) => {
        let types: Vec<Type> = elems.iter().map(|e| self.synth_expr(*e)).collect();
        Type::Tuple(types)
      },
      Expr::List(elems) => self.synth_list(&elems, span),
      Expr::Record(fields) => self.synth_record(fields),
      Expr::Map(entries) => self.synth_map_type(&entries),
      Expr::Func(func) => self.synth_func_type(&func.params, &func.ret_type, func.body),
      Expr::Match(m) => self.synth_match_type(m.scrutinee, &m.arms, span),
      Expr::Ternary(ternary) => self.synth_ternary_type(ternary.cond, ternary.then_, ternary.else_),
      Expr::Propagate(inner) => self.synth_propagate(inner, span),
      Expr::Coalesce(_) => Type::Todo,
      Expr::Slice(slice) => {
        if let Some(s) = slice.start {
          self.synth_expr(s);
        }
        if let Some(e) = slice.end {
          self.synth_expr(e);
        }
        self.synth_expr(slice.expr)
      },
      Expr::NamedArg(na) => self.synth_expr(na.value),
      Expr::Loop(stmts) => {
        self.check_stmts(&stmts);
        Type::Unit
      },
      Expr::Break(value) => {
        if let Some(v) = value {
          self.synth_expr(v);
        }
        Type::Unit
      },
      Expr::Assert(assert) => {
        self.synth_expr(assert.expr);
        if let Some(m) = assert.msg {
          self.synth_expr(m);
        }
        Type::Unit
      },
      Expr::Par(stmts) => self.synth_par_type(&stmts, span),
      Expr::Sel(arms) => self.synth_sel_type(&arms, span),
      Expr::Timeout(timeout) => self.synth_timeout_type(timeout.ms, timeout.body),
      Expr::Emit(emit) => {
        self.synth_expr(emit.value);
        Type::Unit
      },
      Expr::Yield(yld) => {
        self.synth_expr(yld.value);
        Type::Todo
      },
      Expr::With(with) => self.synth_with_type(&with.kind, &with.body),
    }
  }

  fn synth_unary(&mut self, op: UnaryOp, operand: ExprId, span: SourceSpan) -> Type {
    let t = self.synth_expr(operand);
    match op {
      UnaryOp::Neg => match self.table.resolve(&t) {
        Type::Int | Type::Float => t,
        Type::Error => Type::Error,
        Type::Unknown | Type::Todo => t,
        _ => {
          self.emit(DiagLevel::Error, DiagnosticKind::NegationRequiresNumeric, span);
          Type::Error
        },
      },
      UnaryOp::Not => Type::Bool,
    }
  }

  fn synth_propagate(&mut self, inner: ExprId, span: SourceSpan) -> Type {
    let t = self.synth_expr(inner);
    match self.table.resolve(&t) {
      Type::Result { ok, .. } => *ok,
      Type::Maybe(inner) => *inner,
      Type::Unknown | Type::Todo => Type::Unknown,
      Type::Error => Type::Error,
      _ => {
        self.emit(DiagLevel::Error, DiagnosticKind::PropagateRequiresResultOrMaybe, span);
        Type::Error
      },
    }
  }

  fn synth_list(&mut self, elems: &[ListElem], _span: SourceSpan) -> Type {
    if elems.is_empty() {
      return Type::List(Box::new(self.fresh()));
    }
    let first = match &elems[0] {
      ListElem::Single(e) | ListElem::Spread(e) => self.synth_expr(*e),
    };
    for elem in &elems[1..] {
      match elem {
        ListElem::Single(e) | ListElem::Spread(e) => {
          self.synth_expr(*e);
        },
      }
    }
    Type::List(Box::new(first))
  }

  fn synth_record(&mut self, fields: Vec<RecordField>) -> Type {
    let fs: Vec<_> = fields
      .iter()
      .filter_map(|f| match f {
        RecordField::Named { name, value } => Some((*name, self.synth_expr(*value))),
        RecordField::Spread(_) => None,
      })
      .collect();
    Type::Record(fs)
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
        if lt != Type::Bool && lt != Type::Unknown && lt != Type::Todo && lt != Type::Error {
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
