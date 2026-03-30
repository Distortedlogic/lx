mod emit_expr;
mod emit_expr_helpers;
mod emit_pattern;
mod emit_stmt;
mod emit_type;

use lx_ast::ast::{AstArena, Program};

pub struct Formatter<'a> {
  arena: &'a AstArena,
  output: String,
  indent: usize,
}

impl<'a> Formatter<'a> {
  fn new(arena: &'a AstArena) -> Self {
    Self { arena, output: String::new(), indent: 0 }
  }

  fn write(&mut self, s: &str) {
    self.output.push_str(s);
  }
  fn space(&mut self) {
    self.output.push(' ');
  }
  fn newline(&mut self) {
    self.output.push('\n');
    for _ in 0..self.indent {
      self.output.push_str("  ");
    }
  }
  fn indent(&mut self) {
    self.indent += 1;
  }
  fn dedent(&mut self) {
    if self.indent > 0 {
      self.indent -= 1;
    }
  }

  fn format_program<P>(&mut self, program: &Program<P>) {
    for (i, &sid) in program.stmts.iter().enumerate() {
      if i > 0 {
        self.newline();
      }
      self.emit_stmt(sid);
    }
    if !program.stmts.is_empty() {
      self.output.push('\n');
    }
  }
}

pub fn format<P>(program: &Program<P>) -> String {
  let mut f = Formatter::new(&program.arena);
  f.format_program(program);
  f.output
}
