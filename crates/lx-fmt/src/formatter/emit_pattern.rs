use lx_ast::ast::{Pattern, PatternId};

use super::Formatter;

impl Formatter<'_> {
  pub(super) fn emit_pattern(&mut self, id: PatternId) {
    let pattern = self.arena.pattern(id);
    match pattern {
      Pattern::Literal(lit) => self.emit_literal(lit),
      Pattern::Bind(name) => self.write(name.as_str()),
      Pattern::Wildcard => self.write("_"),
      Pattern::Tuple(pats) => {
        self.write("(");
        for (i, &p) in pats.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.emit_pattern(p);
        }
        self.write(")");
      },
      Pattern::List(pl) => {
        self.write("[");
        for (i, &p) in pl.elems.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.emit_pattern(p);
        }
        if let Some(rest) = pl.rest {
          if !pl.elems.is_empty() {
            self.write("; ");
          }
          self.write("..");
          self.write(rest.as_str());
        }
        self.write("]");
      },
      Pattern::Record(pr) => {
        self.write("{ ");
        for (i, f) in pr.fields.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.write(f.name.as_str());
          if let Some(p) = f.pattern {
            self.write(": ");
            self.emit_pattern(p);
          }
        }
        if let Some(rest) = pr.rest {
          if !pr.fields.is_empty() {
            self.write("; ");
          }
          self.write("..");
          self.write(rest.as_str());
        }
        self.write(" }");
      },
      Pattern::Constructor(pc) => {
        self.write(pc.name.as_str());
        for &arg in &pc.args {
          self.space();
          self.emit_pattern(arg);
        }
      },
    }
  }
}
