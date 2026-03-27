use crate::ast::{
  ExprApply, ExprBinary, ExprCoalesce, ExprId, ExprNamedArg, ExprPipe, ExprSlice, ExprTernary, ExprUnary, ExprWith, Literal, Section, StrPart, WithKind,
};

use super::Formatter;
use super::emit_expr::{PREC_APPLY, PREC_COALESCE, PREC_PIPE, PREC_TERNARY, binop_prec};

const PREC_UNARY: u8 = 29;

impl Formatter<'_> {
  pub(super) fn emit_binary(&mut self, b: &ExprBinary, parent_prec: u8) {
    let prec = binop_prec(&b.op);
    let needs_parens = prec < parent_prec;
    if needs_parens {
      self.write("(");
    }
    self.emit_expr_prec(b.left, prec);
    self.write(" ");
    self.write(&b.op.to_string());
    self.write(" ");
    self.emit_expr_prec(b.right, prec + 1);
    if needs_parens {
      self.write(")");
    }
  }

  pub(super) fn emit_unary(&mut self, u: &ExprUnary, parent_prec: u8) {
    let needs_parens = PREC_UNARY < parent_prec;
    if needs_parens {
      self.write("(");
    }
    self.write(&u.op.to_string());
    self.emit_expr_prec(u.operand, PREC_UNARY);
    if needs_parens {
      self.write(")");
    }
  }

  pub(super) fn emit_pipe(&mut self, p: &ExprPipe, parent_prec: u8) {
    let needs_parens = PREC_PIPE < parent_prec;
    if needs_parens {
      self.write("(");
    }
    self.emit_expr_prec(p.left, PREC_PIPE);
    self.write(" | ");
    self.emit_expr_prec(p.right, PREC_PIPE + 1);
    if needs_parens {
      self.write(")");
    }
  }

  pub(super) fn emit_apply(&mut self, a: &ExprApply, parent_prec: u8) {
    let needs_parens = PREC_APPLY < parent_prec;
    if needs_parens {
      self.write("(");
    }
    self.emit_expr_prec(a.func, PREC_APPLY);
    self.space();
    self.emit_expr_prec(a.arg, PREC_APPLY + 1);
    if needs_parens {
      self.write(")");
    }
  }

  pub(super) fn emit_ternary(&mut self, t: &ExprTernary, parent_prec: u8) {
    let needs_parens = PREC_TERNARY < parent_prec;
    if needs_parens {
      self.write("(");
    }
    self.emit_expr_prec(t.cond, PREC_TERNARY + 1);
    self.write(" ? ");
    self.emit_expr(t.then_);
    if let Some(e) = t.else_ {
      self.write(" : ");
      self.emit_expr(e);
    }
    if needs_parens {
      self.write(")");
    }
  }

  pub(super) fn emit_propagate(&mut self, inner: ExprId) {
    self.emit_expr_prec(inner, PREC_APPLY + 1);
    self.write("^");
  }

  pub(super) fn emit_coalesce(&mut self, c: &ExprCoalesce, parent_prec: u8) {
    let needs_parens = PREC_COALESCE < parent_prec;
    if needs_parens {
      self.write("(");
    }
    self.emit_expr_prec(c.expr, PREC_COALESCE);
    self.write(" ?? ");
    self.emit_expr_prec(c.default, PREC_COALESCE + 1);
    if needs_parens {
      self.write(")");
    }
  }

  pub(super) fn emit_slice(&mut self, s: &ExprSlice) {
    self.emit_expr(s.expr);
    self.write("[");
    if let Some(start) = s.start {
      self.emit_expr(start);
    }
    self.write("..");
    if let Some(end) = s.end {
      self.emit_expr(end);
    }
    self.write("]");
  }

  pub(super) fn emit_named_arg(&mut self, na: &ExprNamedArg) {
    self.write("&");
    self.write(na.name.as_str());
    self.write(": ");
    self.emit_expr(na.value);
  }

  pub(super) fn emit_literal(&mut self, lit: &Literal) {
    match lit {
      Literal::Int(n) => self.write(&n.to_string()),
      Literal::Float(f) => self.write(&format!("{f}")),
      Literal::Bool(b) => self.write(if *b { "true" } else { "false" }),
      Literal::Unit => self.write("()"),
      Literal::Str(parts) => {
        self.write("\"");
        for part in parts {
          match part {
            StrPart::Text(s) => self.write(s),
            StrPart::Interp(eid) => {
              self.write("${");
              self.emit_expr(*eid);
              self.write("}");
            },
          }
        }
        self.write("\"");
      },
      Literal::RawStr(s) => {
        self.write("r\"");
        self.write(s);
        self.write("\"");
      },
    }
  }

  pub(super) fn emit_section(&mut self, s: &Section) {
    self.write("(");
    match s {
      Section::Right { op, operand } => {
        self.write(&op.to_string());
        self.space();
        self.emit_expr(*operand);
      },
      Section::Left { operand, op } => {
        self.emit_expr(*operand);
        self.space();
        self.write(&op.to_string());
      },
      Section::BinOp(op) => self.write(&op.to_string()),
      Section::Field(name) => {
        self.write(".");
        self.write(name.as_str());
      },
      Section::Index(i) => {
        self.write(".");
        self.write(&i.to_string());
      },
      Section::FieldCompare { field, op, value } => {
        self.write(".");
        self.write(field.as_str());
        self.space();
        self.write(&op.to_string());
        self.space();
        self.emit_expr(*value);
      },
    }
    self.write(")");
  }

  pub(super) fn emit_with(&mut self, w: &ExprWith) {
    self.write("with ");
    match &w.kind {
      WithKind::Binding { name, value, mutable } => {
        if *mutable {
          self.write("mut ");
        }
        self.write(name.as_str());
        self.write(" = ");
        self.emit_expr(*value);
      },
      WithKind::Resources { resources } => {
        for (i, (eid, name)) in resources.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.emit_expr(*eid);
          self.write(" as ");
          self.write(name.as_str());
        }
      },
      WithKind::Context { fields } => {
        self.write("context ");
        for (i, (name, eid)) in fields.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.write(name.as_str());
          self.write(": ");
          self.emit_expr(*eid);
        }
      },
    }
    self.write(" {");
    self.indent();
    for &sid in &w.body {
      self.newline();
      self.emit_stmt(sid);
    }
    self.dedent();
    self.newline();
    self.write("}");
  }

  pub(super) fn emit_block_keyword(&mut self, keyword: &str, stmts: &[crate::ast::StmtId]) {
    self.write(keyword);
    self.write(" {");
    self.indent();
    for &sid in stmts {
      self.newline();
      self.emit_stmt(sid);
    }
    self.dedent();
    self.newline();
    self.write("}");
  }
}
