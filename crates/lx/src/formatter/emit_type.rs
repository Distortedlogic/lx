use crate::ast::{TypeExpr, TypeExprId};

use super::Formatter;

impl Formatter<'_> {
  pub(super) fn emit_type_expr(&mut self, id: TypeExprId) {
    let te = self.arena.type_expr(id);
    match te {
      TypeExpr::Named(name) => self.write(name.as_str()),
      TypeExpr::Var(name) => self.write(name.as_str()),
      TypeExpr::Applied(name, args) => {
        self.write(name.as_str());
        for &a in args {
          self.space();
          self.emit_type_expr(a);
        }
      },
      TypeExpr::List(inner) => {
        self.write("[");
        self.emit_type_expr(*inner);
        self.write("]");
      },
      TypeExpr::Map { key, value } => {
        self.write("%{");
        self.emit_type_expr(*key);
        self.write(": ");
        self.emit_type_expr(*value);
        self.write("}");
      },
      TypeExpr::Record(fields) => {
        self.write("{ ");
        for (i, f) in fields.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.write(f.name.as_str());
          self.write(": ");
          self.emit_type_expr(f.ty);
        }
        self.write(" }");
      },
      TypeExpr::Tuple(elems) => {
        self.write("(");
        for (i, &e) in elems.iter().enumerate() {
          if i > 0 {
            self.space();
          }
          self.emit_type_expr(e);
        }
        self.write(")");
      },
      TypeExpr::Func { param, ret } => {
        self.emit_type_expr(*param);
        self.write(" -> ");
        self.emit_type_expr(*ret);
      },
      TypeExpr::Fallible { ok, err } => {
        self.emit_type_expr(*ok);
        self.write(" ^ ");
        self.emit_type_expr(*err);
      },
    }
  }
}
