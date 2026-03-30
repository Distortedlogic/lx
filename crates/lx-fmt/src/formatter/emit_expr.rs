use lx_ast::ast::{
  BinOp, Expr, ExprAssert, ExprBlock, ExprBreak, ExprFieldAccess, ExprFunc, ExprId, ExprLoop, ExprMatch, ExprPar, ExprPropagate, ExprTimeout, ExprTuple,
  FieldKind, ListElem, MapEntry, RecordField, SelArm, StmtId,
};

use super::Formatter;

pub(super) const PREC_TERNARY: u8 = 3;
pub(super) const PREC_COALESCE: u8 = 11;
const PREC_OR: u8 = 13;
const PREC_AND: u8 = 15;
const PREC_CMP: u8 = 17;
pub(super) const PREC_PIPE: u8 = 19;
const PREC_CONCAT: u8 = 21;
const PREC_RANGE: u8 = 23;
const PREC_ADD: u8 = 25;
const PREC_MUL: u8 = 27;
pub(super) const PREC_APPLY: u8 = 31;

pub(super) fn binop_prec(op: &BinOp) -> u8 {
  match op {
    BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::IntDiv => PREC_MUL,
    BinOp::Add | BinOp::Sub => PREC_ADD,
    BinOp::Range | BinOp::RangeInclusive => PREC_RANGE,
    BinOp::Concat => PREC_CONCAT,
    BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => PREC_CMP,
    BinOp::And => PREC_AND,
    BinOp::Or => PREC_OR,
  }
}

impl Formatter<'_> {
  pub(super) fn emit_expr(&mut self, id: ExprId) {
    self.emit_expr_prec(id, 0);
  }

  pub(super) fn emit_expr_prec(&mut self, id: ExprId, parent_prec: u8) {
    let expr = self.arena.expr(id);
    match expr {
      Expr::Literal(lit) => self.emit_literal(lit),
      Expr::Ident(name) => self.write(name.as_str()),
      Expr::TypeConstructor(name) => self.write(name.as_str()),
      Expr::Binary(b) => self.emit_binary(b, parent_prec),
      Expr::Unary(u) => self.emit_unary(u, parent_prec),
      Expr::Pipe(p) => self.emit_pipe(p, parent_prec),
      Expr::Apply(a) => self.emit_apply(a, parent_prec),
      Expr::Section(s) => self.emit_section(s),
      Expr::FieldAccess(fa) => self.emit_field_access(fa),
      Expr::Block(ExprBlock { stmts }) => self.emit_block(stmts),
      Expr::Tuple(ExprTuple { elems }) => self.emit_tuple(elems),
      Expr::List(elems) => self.emit_list(elems),
      Expr::Record(fields) => self.emit_record(fields),
      Expr::Map(entries) => self.emit_map(entries),
      Expr::Func(func) => self.emit_func(func),
      Expr::Match(m) => self.emit_match(m),
      Expr::Ternary(t) => self.emit_ternary(t, parent_prec),
      Expr::Propagate(ExprPropagate { inner }) => self.emit_propagate(*inner),
      Expr::Coalesce(c) => self.emit_coalesce(c, parent_prec),
      Expr::Slice(s) => self.emit_slice(s),
      Expr::NamedArg(na) => self.emit_named_arg(na),
      Expr::Loop(ExprLoop { stmts }) => self.emit_block_keyword("loop", stmts),
      Expr::Break(ExprBreak { value: val }) => self.emit_break(val),
      Expr::Assert(a) => self.emit_assert(a),
      Expr::Par(ExprPar { stmts }) => self.emit_block_keyword("par", stmts),
      Expr::Sel(arms) => self.emit_sel(arms),
      Expr::Timeout(t) => self.emit_timeout(t),
      Expr::Emit(e) => {
        self.write("emit ");
        self.emit_expr(e.value);
      },
      Expr::Yield(y) => {
        self.write("yield ");
        self.emit_expr(y.value);
      },
      Expr::With(w) => self.emit_with(w),
      Expr::Tell(t) => {
        self.emit_expr(t.target);
        self.write(" ~> ");
        self.emit_expr(t.msg);
      },
      Expr::Ask(a) => {
        self.emit_expr(a.target);
        self.write(" ~>? ");
        self.emit_expr(a.msg);
      },
      Expr::Spawn(inner) => {
        self.write("spawn ");
        self.emit_expr(*inner);
      },
      Expr::Stop => {
        self.write("stop");
      },
      Expr::Grouped(inner) => {
        self.write("(");
        self.emit_expr(*inner);
        self.write(")");
      },
    }
  }

  fn emit_field_access(&mut self, fa: &ExprFieldAccess) {
    self.emit_expr_prec(fa.expr, PREC_APPLY + 1);
    self.write(".");
    match &fa.field {
      FieldKind::Named(n) => self.write(n.as_str()),
      FieldKind::Index(i) => self.write(&i.to_string()),
      FieldKind::Computed(eid) => {
        self.write("[");
        self.emit_expr(*eid);
        self.write("]");
      },
    }
  }

  fn emit_block(&mut self, stmts: &[StmtId]) {
    self.write("{");
    self.indent();
    for &sid in stmts {
      self.newline();
      self.emit_stmt(sid);
    }
    self.dedent();
    self.newline();
    self.write("}");
  }

  fn emit_tuple(&mut self, elems: &[ExprId]) {
    self.write("(");
    for (i, &e) in elems.iter().enumerate() {
      if i > 0 {
        self.write("; ");
      }
      self.emit_expr(e);
    }
    self.write(")");
  }

  fn emit_list(&mut self, elems: &[ListElem]) {
    self.write("[");
    for (i, elem) in elems.iter().enumerate() {
      if i > 0 {
        self.write("; ");
      }
      match elem {
        ListElem::Single(eid) => self.emit_expr(*eid),
        ListElem::Spread(eid) => {
          self.write("..");
          self.emit_expr(*eid);
        },
      }
    }
    self.write("]");
  }

  fn emit_record(&mut self, fields: &[RecordField]) {
    self.write("{ ");
    for (i, f) in fields.iter().enumerate() {
      if i > 0 {
        self.write("; ");
      }
      match f {
        RecordField::Named { name, value } => {
          let is_shorthand = matches!(self.arena.expr(*value), Expr::Ident(sym) if *sym == *name);
          self.write(name.as_str());
          if !is_shorthand {
            self.write(": ");
            self.emit_expr(*value);
          }
        },
        RecordField::Spread(eid) => {
          self.write("..");
          self.emit_expr(*eid);
        },
      }
    }
    self.write(" }");
  }

  fn emit_map(&mut self, entries: &[MapEntry]) {
    self.write("%{");
    for (i, e) in entries.iter().enumerate() {
      if i > 0 {
        self.write("; ");
      }
      match e {
        MapEntry::Keyed { key, value } => {
          self.emit_expr(*key);
          self.write(": ");
          self.emit_expr(*value);
        },
        MapEntry::Spread(eid) => {
          self.write("..");
          self.emit_expr(*eid);
        },
      }
    }
    self.write("}");
  }

  fn emit_func(&mut self, func: &ExprFunc) {
    self.write("(");
    for (i, p) in func.params.iter().enumerate() {
      if i > 0 {
        self.space();
      }
      self.write(p.name.as_str());
      if let Some(ty) = p.type_ann {
        self.write(": ");
        self.emit_type_expr(ty);
      }
      if let Some(default) = p.default {
        self.write(" = ");
        self.emit_expr(default);
      }
    }
    self.write(")");
    self.emit_type_params(&func.type_params);
    if let Some(ret) = func.ret_type {
      self.write(" -> ");
      self.emit_type_expr(ret);
    }
    if let Some(guard) = func.guard {
      self.write(" & ");
      self.emit_expr(guard);
    }
    self.space();
    self.emit_expr(func.body);
  }

  fn emit_match(&mut self, m: &ExprMatch) {
    self.emit_expr_prec(m.scrutinee, PREC_TERNARY + 1);
    self.write(" ? {");
    self.indent();
    for arm in &m.arms {
      self.newline();
      self.emit_pattern(arm.pattern);
      if let Some(guard) = arm.guard {
        self.write(" & ");
        self.emit_expr(guard);
      }
      self.write(" -> ");
      self.emit_expr(arm.body);
    }
    self.dedent();
    self.newline();
    self.write("}");
  }

  fn emit_break(&mut self, val: &Option<ExprId>) {
    self.write("break");
    if let Some(v) = val {
      self.space();
      self.emit_expr(*v);
    }
  }

  fn emit_assert(&mut self, a: &ExprAssert) {
    self.write("assert ");
    self.emit_expr(a.expr);
    if let Some(msg) = a.msg {
      self.space();
      self.emit_expr(msg);
    }
  }

  fn emit_sel(&mut self, arms: &[SelArm]) {
    self.write("sel {");
    self.indent();
    for arm in arms {
      self.newline();
      self.emit_expr(arm.expr);
      self.write(" -> ");
      self.emit_expr(arm.handler);
    }
    self.dedent();
    self.newline();
    self.write("}");
  }

  fn emit_timeout(&mut self, t: &ExprTimeout) {
    self.write("timeout ");
    self.emit_expr(t.ms);
    self.space();
    self.emit_expr(t.body);
  }
}
