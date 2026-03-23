use crate::ast::{AstArena, Core, Expr, ExprId, NodeId, Program, StmtId, WithKind};

pub fn validate_core(program: &Program<Core>) {
  for &sid in &program.stmts {
    validate_stmt(sid, &program.arena);
  }
}

fn validate_stmt(id: StmtId, arena: &AstArena) {
  let stmt = arena.stmt(id);
  for child in stmt.children() {
    match child {
      NodeId::Expr(eid) => validate_expr(eid, arena),
      NodeId::Stmt(sid) => validate_stmt(sid, arena),
      _ => {},
    }
  }
}

fn validate_expr(id: ExprId, arena: &AstArena) {
  let span = arena.expr_span(id);
  let expr = arena.expr(id);

  match expr {
    Expr::Pipe(_) => panic!("Core AST contains Expr::Pipe at offset {}. The desugarer should have converted this to Expr::Apply.", span.offset()),
    Expr::Section(_) => panic!("Core AST contains Expr::Section at offset {}. The desugarer should have converted this to a lambda.", span.offset()),
    Expr::Ternary(_) => panic!("Core AST contains Expr::Ternary at offset {}. The desugarer should have converted this to Expr::Match.", span.offset()),
    Expr::Coalesce(_) => panic!("Core AST contains Expr::Coalesce at offset {}. The desugarer should have converted this to Expr::Match.", span.offset()),
    Expr::With(w) if matches!(w.kind, WithKind::Binding { .. }) => {
      panic!("Core AST contains Expr::With(Binding) at offset {}. The desugarer should have converted this to Expr::Block.", span.offset())
    },
    _ => {},
  }

  for child in expr.children() {
    match child {
      NodeId::Expr(eid) => validate_expr(eid, arena),
      NodeId::Stmt(sid) => validate_stmt(sid, arena),
      _ => {},
    }
  }
}
