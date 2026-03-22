mod fold_expr;
mod fold_expr_compound;
mod fold_pattern;
mod fold_stmt;
mod fold_type;

pub use fold_expr::*;
pub use fold_expr_compound::*;
pub use fold_pattern::*;
pub use fold_stmt::*;
pub use fold_type::*;

use crate::ast::{
    Binding, ExprApply, ExprAssert, ExprBinary, ExprCoalesce, ExprEmit,
    ExprFieldAccess, ExprFunc, ExprMatch, ExprNamedArg, ExprPipe, ExprSlice,
    ExprTernary, ExprTimeout, ExprUnary, ExprWith, ExprYield, Expr, ListElem,
    Literal, MapEntry, Pattern, PatternConstructor, PatternList, PatternRecord,
    Program, RecordField, SExpr, SPattern, SStmt, SType, Section, SelArm,
    Stmt, TypeExpr,
};
use crate::sym::Sym;
use miette::SourceSpan;

pub trait AstFolder {
    fn fold_program(&mut self, program: Program) -> Program {
        fold_program(self, program)
    }

    fn fold_stmt(&mut self, stmt: Stmt, span: SourceSpan) -> SStmt {
        fold_stmt(self, stmt, span)
    }

    fn fold_binding(&mut self, binding: Binding, span: SourceSpan) -> SStmt {
        fold_binding(self, binding, span)
    }

    fn fold_expr(&mut self, expr: Expr, span: SourceSpan) -> SExpr {
        fold_expr(self, expr, span)
    }

    fn fold_literal(&mut self, lit: Literal, span: SourceSpan) -> Literal {
        fold_literal(self, lit, span)
    }

    fn fold_ident(&mut self, name: Sym, _span: SourceSpan) -> Sym {
        name
    }

    fn fold_type_constructor(&mut self, name: Sym, _span: SourceSpan) -> Sym {
        name
    }

    fn fold_binary(&mut self, b: ExprBinary, span: SourceSpan) -> SExpr {
        fold_binary(self, b, span)
    }

    fn fold_unary(&mut self, u: ExprUnary, span: SourceSpan) -> SExpr {
        fold_unary(self, u, span)
    }

    fn fold_pipe(&mut self, p: ExprPipe, span: SourceSpan) -> SExpr {
        fold_pipe(self, p, span)
    }

    fn fold_apply(&mut self, a: ExprApply, span: SourceSpan) -> SExpr {
        fold_apply(self, a, span)
    }

    fn fold_section(&mut self, s: Section, span: SourceSpan) -> SExpr {
        fold_section(self, s, span)
    }

    fn fold_field_access(&mut self, fa: ExprFieldAccess, span: SourceSpan) -> SExpr {
        fold_field_access(self, fa, span)
    }

    fn fold_block(&mut self, stmts: Vec<SStmt>, span: SourceSpan) -> SExpr {
        fold_block(self, stmts, span)
    }

    fn fold_tuple(&mut self, elems: Vec<SExpr>, span: SourceSpan) -> SExpr {
        fold_tuple(self, elems, span)
    }

    fn fold_list(&mut self, elems: Vec<ListElem>, span: SourceSpan) -> SExpr {
        fold_list(self, elems, span)
    }

    fn fold_record(&mut self, fields: Vec<RecordField>, span: SourceSpan) -> SExpr {
        fold_record(self, fields, span)
    }

    fn fold_map(&mut self, entries: Vec<MapEntry>, span: SourceSpan) -> SExpr {
        fold_map(self, entries, span)
    }

    fn fold_func(&mut self, func: ExprFunc, span: SourceSpan) -> SExpr {
        fold_func(self, func, span)
    }

    fn fold_match(&mut self, m: ExprMatch, span: SourceSpan) -> SExpr {
        fold_match(self, m, span)
    }

    fn fold_ternary(&mut self, t: ExprTernary, span: SourceSpan) -> SExpr {
        fold_ternary(self, t, span)
    }

    fn fold_propagate(&mut self, inner: Box<SExpr>, span: SourceSpan) -> SExpr {
        fold_propagate(self, inner, span)
    }

    fn fold_coalesce(&mut self, c: ExprCoalesce, span: SourceSpan) -> SExpr {
        fold_coalesce(self, c, span)
    }

    fn fold_slice(&mut self, s: ExprSlice, span: SourceSpan) -> SExpr {
        fold_slice(self, s, span)
    }

    fn fold_named_arg(&mut self, na: ExprNamedArg, span: SourceSpan) -> SExpr {
        fold_named_arg(self, na, span)
    }

    fn fold_loop(&mut self, stmts: Vec<SStmt>, span: SourceSpan) -> SExpr {
        fold_loop(self, stmts, span)
    }

    fn fold_break(&mut self, val: Option<Box<SExpr>>, span: SourceSpan) -> SExpr {
        fold_break(self, val, span)
    }

    fn fold_assert(&mut self, a: ExprAssert, span: SourceSpan) -> SExpr {
        fold_assert(self, a, span)
    }

    fn fold_par(&mut self, stmts: Vec<SStmt>, span: SourceSpan) -> SExpr {
        fold_par(self, stmts, span)
    }

    fn fold_sel(&mut self, arms: Vec<SelArm>, span: SourceSpan) -> SExpr {
        fold_sel(self, arms, span)
    }

    fn fold_timeout(&mut self, t: ExprTimeout, span: SourceSpan) -> SExpr {
        fold_timeout(self, t, span)
    }

    fn fold_emit(&mut self, e: ExprEmit, span: SourceSpan) -> SExpr {
        fold_emit(self, e, span)
    }

    fn fold_yield(&mut self, y: ExprYield, span: SourceSpan) -> SExpr {
        fold_yield(self, y, span)
    }

    fn fold_with(&mut self, w: ExprWith, span: SourceSpan) -> SExpr {
        fold_with(self, w, span)
    }

    fn fold_pattern(&mut self, pattern: Pattern, span: SourceSpan) -> SPattern {
        fold_pattern(self, pattern, span)
    }

    fn fold_pattern_list(&mut self, pl: PatternList, span: SourceSpan) -> SPattern {
        fold_pattern_list(self, pl, span)
    }

    fn fold_pattern_record(&mut self, pr: PatternRecord, span: SourceSpan) -> SPattern {
        fold_pattern_record(self, pr, span)
    }

    fn fold_pattern_constructor(&mut self, pc: PatternConstructor, span: SourceSpan) -> SPattern {
        fold_pattern_constructor(self, pc, span)
    }

    fn fold_type_expr(&mut self, type_expr: TypeExpr, span: SourceSpan) -> SType {
        fold_type_expr(self, type_expr, span)
    }
}
