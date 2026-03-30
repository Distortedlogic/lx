use lx_ast::ast::{BinOp, Expr, ExprBlock, ExprBreak, ExprId, ExprLoop, ExprPar, ExprPropagate, ExprTuple, FieldKind, ListElem, Literal, RecordField, UnaryOp};
use miette::SourceSpan;

use super::diagnostics::DiagnosticKind;
use super::type_arena::TypeId;
use super::type_error::TypeContext;
use super::types::Type;
use super::{Checker, DiagLevel};

impl Checker<'_> {
  pub(super) fn synth_expr(&mut self, eid: ExprId) -> TypeId {
    let ty = self.synth_expr_inner(eid);
    self.record_type(eid, ty);
    ty
  }

  fn synth_expr_inner(&mut self, eid: ExprId) -> TypeId {
    let arena = self.arena;
    let expr = arena.expr(eid);
    let span = arena.expr_span(eid);
    match expr {
      Expr::Literal(lit) => self.synth_literal(lit),
      Expr::Ident(name) => {
        if let Some(def_id) = self.sem.resolve_in_scope(*name) {
          self.sem.add_reference(eid, def_id);
        } else {
          let scope_names = self.sem.names_in_scope();
          let candidates: Vec<&str> = scope_names.iter().map(|s| s.as_str()).collect();
          let suggestions = super::suggest::closest_matches(name.as_str(), &candidates, 3);
          self.emit(DiagLevel::Error, DiagnosticKind::UnknownIdent { name: *name, suggestions }, span);
        }
        if let Some(narrowed) = self.narrowing.lookup(*name) { narrowed } else { self.sem.lookup_type(*name).unwrap_or(self.type_arena.unknown()) }
      },
      Expr::TypeConstructor(name) => {
        if let Some(def_id) = self.sem.resolve_in_scope(*name) {
          self.sem.add_reference(eid, def_id);
        }
        self.type_arena.unknown()
      },
      Expr::Binary(binary) => {
        let lt = self.synth_expr(binary.left);
        let rt = self.synth_expr(binary.right);
        self.synth_binary_type(&binary.op, lt, rt, span)
      },
      Expr::Unary(unary) => self.synth_unary(unary.op, unary.operand, span),
      Expr::Pipe(_) | Expr::Tell(_) | Expr::Ask(_) => unreachable!(),
      Expr::Apply(apply) => self.synth_apply_type(apply.func, apply.arg),
      Expr::Section(_) => unreachable!(),
      Expr::FieldAccess(fa) => {
        let fa = fa.clone();
        self.synth_expr(fa.expr);
        if let FieldKind::Computed(c) = fa.field {
          self.synth_expr(c);
        }
        self.type_arena.todo()
      },
      Expr::Block(ExprBlock { stmts }) => {
        let stmts = stmts.clone();
        self.check_stmts(&stmts)
      },
      Expr::Tuple(ExprTuple { elems }) => {
        let elems = elems.clone();
        let types: Vec<TypeId> = elems.iter().map(|e| self.synth_expr(*e)).collect();
        self.type_arena.alloc(Type::Tuple(types))
      },
      Expr::List(elems) => {
        let elems = elems.clone();
        self.synth_list(&elems, span)
      },
      Expr::Record(fields) => {
        let fields = fields.clone();
        self.synth_record(&fields)
      },
      Expr::Map(entries) => {
        let entries = entries.clone();
        self.synth_map_type(&entries)
      },
      Expr::Func(func) => {
        let func = func.clone();
        self.synth_func_type(&func.type_params, &func.params, &func.ret_type, func.body)
      },
      Expr::Match(m) => {
        let m = m.clone();
        self.synth_match_type(m.scrutinee, &m.arms, span)
      },
      Expr::Ternary(_) => unreachable!(),
      Expr::Propagate(ExprPropagate { inner }) => self.synth_propagate(*inner, span),
      Expr::Coalesce(_) => unreachable!(),
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
      Expr::Loop(ExprLoop { stmts }) => {
        let stmts = stmts.clone();
        self.check_stmts(&stmts);
        self.type_arena.unit()
      },
      Expr::Break(ExprBreak { value }) => {
        if let Some(v) = *value {
          self.synth_expr(v);
        }
        self.type_arena.unit()
      },
      Expr::Assert(assert) => {
        self.synth_expr(assert.expr);
        if let Some(m) = assert.msg {
          self.synth_expr(m);
        }
        self.type_arena.unit()
      },
      Expr::Par(ExprPar { stmts }) => {
        let stmts = stmts.clone();
        self.synth_par_type(&stmts, span)
      },
      Expr::Sel(arms) => {
        let arms = arms.clone();
        self.synth_sel_type(&arms, span)
      },
      Expr::Timeout(timeout) => self.synth_timeout_type(timeout.ms, timeout.body),
      Expr::Spawn(inner) => {
        self.synth_expr(*inner);
        self.type_arena.unknown()
      },
      Expr::Stop => self.type_arena.unit(),
      Expr::Emit(emit) => {
        self.synth_expr(emit.value);
        self.type_arena.unit()
      },
      Expr::Yield(yld) => {
        self.synth_expr(yld.value);
        self.type_arena.todo()
      },
      Expr::With(with) => {
        let with = with.clone();
        self.synth_with_type(&with.kind, &with.body)
      },
      Expr::Grouped(inner) => self.synth_expr(*inner),
    }
  }

  fn synth_unary(&mut self, op: UnaryOp, operand: ExprId, span: SourceSpan) -> TypeId {
    let t = self.synth_expr(operand);
    match op {
      UnaryOp::Neg => {
        let resolved = self.table.resolve(t, &self.type_arena);
        match self.type_arena.get(resolved) {
          Type::Int | Type::Float => t,
          Type::Error => self.type_arena.error(),
          Type::Unknown | Type::Todo => t,
          _ => {
            self.emit(DiagLevel::Error, DiagnosticKind::NegationRequiresNumeric, span);
            self.type_arena.error()
          },
        }
      },
      UnaryOp::Not => self.type_arena.bool(),
    }
  }

  fn synth_propagate(&mut self, inner: ExprId, span: SourceSpan) -> TypeId {
    let t = self.synth_expr(inner);
    let resolved = self.table.resolve(t, &self.type_arena);
    match self.type_arena.get(resolved).clone() {
      Type::Result { ok, .. } => ok,
      Type::Maybe(inner) => inner,
      Type::Unknown | Type::Todo => self.type_arena.unknown(),
      Type::Error => self.type_arena.error(),
      _ => {
        self.emit(DiagLevel::Error, DiagnosticKind::PropagateRequiresResultOrMaybe, span);
        self.type_arena.error()
      },
    }
  }

  fn synth_list(&mut self, elems: &[ListElem], _span: SourceSpan) -> TypeId {
    if elems.is_empty() {
      let elem_ty = self.fresh();
      return self.type_arena.alloc(Type::List(elem_ty));
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
    self.type_arena.alloc(Type::List(first))
  }

  fn synth_record(&mut self, fields: &[RecordField]) -> TypeId {
    let fs: Vec<_> = fields
      .iter()
      .filter_map(|f| match f {
        RecordField::Named { name, value } => Some((*name, self.synth_expr(*value))),
        RecordField::Spread(_) => None,
      })
      .collect();
    self.type_arena.alloc(Type::Record(fs))
  }

  pub(super) fn synth_binary_type(&mut self, op: &BinOp, lt: TypeId, rt: TypeId, span: SourceSpan) -> TypeId {
    let lt = self.table.resolve(lt, &self.type_arena);
    let rt = self.table.resolve(rt, &self.type_arena);
    match op {
      BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::IntDiv => {
        let ctx = TypeContext::BinaryOp { op: format!("{op:?}") };
        match self.table.unify_with_context(lt, rt, ctx, &mut self.type_arena) {
          Ok(t) => t,
          Err(te) => {
            self.emit_type_error(&te, span);
            self.type_arena.error()
          },
        }
      },
      BinOp::Concat => self.type_arena.str(),
      BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => self.type_arena.bool(),
      BinOp::And | BinOp::Or => {
        let bool_id = self.type_arena.bool();
        let unknown_id = self.type_arena.unknown();
        let todo_id = self.type_arena.todo();
        let error_id = self.type_arena.error();
        if lt != bool_id && lt != unknown_id && lt != todo_id && lt != error_id {
          self.emit(DiagLevel::Error, DiagnosticKind::LogicalOpRequiresBool, span);
        }
        bool_id
      },
      BinOp::Range | BinOp::RangeInclusive => {
        let int_id = self.type_arena.int();
        self.type_arena.alloc(Type::List(int_id))
      },
    }
  }

  pub(super) fn synth_literal_type(&self, lit: &Literal) -> TypeId {
    match lit {
      Literal::Int(_) => self.type_arena.int(),
      Literal::Float(_) => self.type_arena.float(),
      Literal::Str(_) | Literal::RawStr(_) => self.type_arena.str(),
      Literal::Bool(_) => self.type_arena.bool(),
      Literal::Unit => self.type_arena.unit(),
    }
  }
}
