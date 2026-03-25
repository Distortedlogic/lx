use la_arena::{Arena, Idx};
use miette::SourceSpan;

use super::{Expr, Pattern, Stmt, TypeExpr};

#[derive(Debug, Clone)]
pub struct Spanned<T> {
  pub node: T,
  pub span: SourceSpan,
}

pub type ExprId = Idx<Spanned<Expr>>;
pub type StmtId = Idx<Spanned<Stmt>>;
pub type PatternId = Idx<Spanned<Pattern>>;
pub type TypeExprId = Idx<Spanned<TypeExpr>>;

#[derive(Debug, Clone, Default)]
pub struct AstArena {
  pub(crate) exprs: Arena<Spanned<Expr>>,
  pub(crate) stmts: Arena<Spanned<Stmt>>,
  pub(crate) patterns: Arena<Spanned<Pattern>>,
  pub(crate) type_exprs: Arena<Spanned<TypeExpr>>,
}

impl AstArena {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn alloc_expr(&mut self, expr: Expr, span: SourceSpan) -> ExprId {
    self.exprs.alloc(Spanned { node: expr, span })
  }

  pub fn alloc_stmt(&mut self, stmt: Stmt, span: SourceSpan) -> StmtId {
    self.stmts.alloc(Spanned { node: stmt, span })
  }

  pub fn alloc_pattern(&mut self, pattern: Pattern, span: SourceSpan) -> PatternId {
    self.patterns.alloc(Spanned { node: pattern, span })
  }

  pub fn alloc_type_expr(&mut self, type_expr: TypeExpr, span: SourceSpan) -> TypeExprId {
    self.type_exprs.alloc(Spanned { node: type_expr, span })
  }

  pub fn expr(&self, id: ExprId) -> &Expr {
    &self.exprs[id].node
  }

  pub fn expr_spanned(&self, id: ExprId) -> &Spanned<Expr> {
    &self.exprs[id]
  }

  pub fn expr_span(&self, id: ExprId) -> SourceSpan {
    self.exprs[id].span
  }

  pub fn stmt(&self, id: StmtId) -> &Stmt {
    &self.stmts[id].node
  }

  pub fn stmt_mut(&mut self, id: StmtId) -> &mut Stmt {
    &mut self.stmts[id].node
  }

  pub fn stmt_spanned(&self, id: StmtId) -> &Spanned<Stmt> {
    &self.stmts[id]
  }

  pub fn stmt_span(&self, id: StmtId) -> SourceSpan {
    self.stmts[id].span
  }

  pub fn pattern(&self, id: PatternId) -> &Pattern {
    &self.patterns[id].node
  }

  pub fn pattern_spanned(&self, id: PatternId) -> &Spanned<Pattern> {
    &self.patterns[id]
  }

  pub fn pattern_span(&self, id: PatternId) -> SourceSpan {
    self.patterns[id].span
  }

  pub fn type_expr(&self, id: TypeExprId) -> &TypeExpr {
    &self.type_exprs[id].node
  }

  pub fn type_expr_spanned(&self, id: TypeExprId) -> &Spanned<TypeExpr> {
    &self.type_exprs[id]
  }

  pub fn type_expr_span(&self, id: TypeExprId) -> SourceSpan {
    self.type_exprs[id].span
  }

  pub fn iter_exprs(&self) -> impl Iterator<Item = (ExprId, &Spanned<Expr>)> {
    self.exprs.iter()
  }

  pub fn iter_stmts(&self) -> impl Iterator<Item = (StmtId, &Spanned<Stmt>)> {
    self.stmts.iter()
  }

  pub fn iter_patterns(&self) -> impl Iterator<Item = (PatternId, &Spanned<Pattern>)> {
    self.patterns.iter()
  }

  pub fn iter_type_exprs(&self) -> impl Iterator<Item = (TypeExprId, &Spanned<TypeExpr>)> {
    self.type_exprs.iter()
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeId {
  Expr(ExprId),
  Stmt(StmtId),
  Pattern(PatternId),
  TypeExpr(TypeExprId),
}

impl NodeId {
  pub fn span(&self, arena: &AstArena) -> SourceSpan {
    match self {
      NodeId::Expr(id) => arena.expr_span(*id),
      NodeId::Stmt(id) => arena.stmt_span(*id),
      NodeId::Pattern(id) => arena.pattern_span(*id),
      NodeId::TypeExpr(id) => arena.type_expr_span(*id),
    }
  }
}

pub trait AstNode {
  fn span(&self, arena: &AstArena) -> SourceSpan;
  fn as_node_id(&self) -> NodeId;
}

impl AstNode for ExprId {
  fn span(&self, arena: &AstArena) -> SourceSpan {
    arena.expr_span(*self)
  }
  fn as_node_id(&self) -> NodeId {
    NodeId::Expr(*self)
  }
}

impl AstNode for StmtId {
  fn span(&self, arena: &AstArena) -> SourceSpan {
    arena.stmt_span(*self)
  }
  fn as_node_id(&self) -> NodeId {
    NodeId::Stmt(*self)
  }
}

impl AstNode for PatternId {
  fn span(&self, arena: &AstArena) -> SourceSpan {
    arena.pattern_span(*self)
  }
  fn as_node_id(&self) -> NodeId {
    NodeId::Pattern(*self)
  }
}

impl AstNode for TypeExprId {
  fn span(&self, arena: &AstArena) -> SourceSpan {
    arena.type_expr_span(*self)
  }
  fn as_node_id(&self) -> NodeId {
    NodeId::TypeExpr(*self)
  }
}
