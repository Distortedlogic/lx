mod visitor;

use crate::ast::{AstArena, Pattern, PatternId, Program};
use crate::visitor::dispatch_stmt;

use super::symbol_table::{DefKind, SymbolTable};

pub fn resolve<P>(program: &Program<P>) -> SymbolTable {
  let mut resolver = Resolver::new(&program.arena);
  for &sid in &program.stmts {
    if dispatch_stmt(&mut resolver, sid, &program.arena).is_break() {
      break;
    }
  }
  resolver.table
}

pub(super) struct Resolver<'a> {
  pub(super) table: SymbolTable,
  pub(super) arena: &'a AstArena,
}

impl<'a> Resolver<'a> {
  fn new(arena: &'a AstArena) -> Self {
    Self { table: SymbolTable::new(), arena }
  }

  pub(super) fn bind_pattern_names(&mut self, pid: PatternId) {
    let span = self.arena.pattern_span(pid);
    match self.arena.pattern(pid).clone() {
      Pattern::Bind(name) => {
        self.table.define(name, DefKind::PatternBind, span);
      },
      Pattern::Constructor(c) => {
        for arg in &c.args {
          self.bind_pattern_names(*arg);
        }
      },
      Pattern::Tuple(pats) => {
        for p in &pats {
          self.bind_pattern_names(*p);
        }
      },
      Pattern::List(pl) => {
        for p in &pl.elems {
          self.bind_pattern_names(*p);
        }
        if let Some(rest) = pl.rest {
          self.table.define(rest, DefKind::PatternBind, span);
        }
      },
      Pattern::Record(pr) => {
        for f in &pr.fields {
          if let Some(p) = f.pattern {
            self.bind_pattern_names(p);
          } else {
            self.table.define(f.name, DefKind::PatternBind, span);
          }
        }
        if let Some(rest) = pr.rest {
          self.table.define(rest, DefKind::PatternBind, span);
        }
      },
      Pattern::Literal(_) | Pattern::Wildcard => {},
    }
  }
}
