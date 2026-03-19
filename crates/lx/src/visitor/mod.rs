use crate::ast::{
    AgentMethod, BinOp, Binding, ClassField, Expr, FieldKind, FieldPattern, ListElem, Literal,
    MapEntry, MatchArm, McpToolDecl, Param, Pattern, Program, ProtocolEntry, ProtocolUnionDef,
    RecordField, SExpr, SPattern, SType, Section, SelArm, ShellMode, Stmt, StrPart,
    TraitMethodDecl, TypeExpr, TypeField, UnaryOp, UseStmt,
};
use crate::span::Span;

mod walk;
pub use walk::*;

pub struct TraitDeclCtx<'a> {
    pub name: &'a str,
    pub methods: &'a [TraitMethodDecl],
    pub requires: &'a [String],
    pub description: Option<&'a str>,
    pub tags: &'a [String],
}

pub struct AgentDeclCtx<'a> {
    pub name: &'a str,
    pub traits: &'a [String],
    pub uses: &'a [(String, String)],
    pub init: Option<&'a SExpr>,
    pub on: Option<&'a SExpr>,
    pub methods: &'a [AgentMethod],
}

pub struct RefineCtx<'a> {
    pub initial: &'a SExpr,
    pub grade: &'a SExpr,
    pub revise: &'a SExpr,
    pub threshold: &'a SExpr,
    pub max_rounds: &'a SExpr,
    pub on_round: Option<&'a SExpr>,
}

pub trait AstVisitor {
    fn visit_program(&mut self, program: &Program) {
        walk_program(self, program);
    }
    fn visit_stmt(&mut self, stmt: &Stmt, span: Span) {
        walk_stmt(self, stmt, span);
    }
    fn visit_binding(&mut self, binding: &Binding, span: Span) {
        walk_binding(self, binding, span);
    }
    fn visit_type_def(
        &mut self,
        _name: &str,
        _variants: &[(String, usize)],
        _exported: bool,
        _span: Span,
    ) {
    }
    fn visit_protocol(
        &mut self,
        _name: &str,
        entries: &[ProtocolEntry],
        _exported: bool,
        span: Span,
    ) {
        walk_protocol(self, entries, span);
    }
    fn visit_protocol_union(&mut self, _def: &ProtocolUnionDef, _span: Span) {}
    fn visit_mcp_decl(
        &mut self,
        _name: &str,
        _tools: &[McpToolDecl],
        _exported: bool,
        _span: Span,
    ) {
    }
    fn visit_trait_decl(&mut self, _ctx: &TraitDeclCtx<'_>, _span: Span) {}
    fn visit_agent_decl(&mut self, ctx: &AgentDeclCtx<'_>, span: Span) {
        walk_agent_decl(self, ctx, span);
    }
    fn visit_class_decl(
        &mut self,
        _name: &str,
        _traits: &[String],
        fields: &[ClassField],
        methods: &[AgentMethod],
        _exported: bool,
        span: Span,
    ) {
        walk_class_decl(self, fields, methods, span);
    }
    fn visit_field_update(&mut self, _name: &str, _fields: &[String], value: &SExpr, span: Span) {
        walk_field_update(self, value, span);
    }
    fn visit_use(&mut self, _stmt: &UseStmt, _span: Span) {}
    fn visit_expr(&mut self, expr: &Expr, span: Span) {
        walk_expr(self, expr, span);
    }
    fn visit_literal(&mut self, lit: &Literal, span: Span) {
        walk_literal(self, lit, span);
    }
    fn visit_ident(&mut self, _name: &str, _span: Span) {}
    fn visit_type_constructor(&mut self, _name: &str, _span: Span) {}
    fn visit_binary(&mut self, _op: BinOp, left: &SExpr, right: &SExpr, span: Span) {
        walk_binary(self, left, right, span);
    }
    fn visit_unary(&mut self, _op: UnaryOp, operand: &SExpr, span: Span) {
        walk_unary(self, operand, span);
    }
    fn visit_pipe(&mut self, left: &SExpr, right: &SExpr, span: Span) {
        walk_pipe(self, left, right, span);
    }
    fn visit_apply(&mut self, func: &SExpr, arg: &SExpr, span: Span) {
        walk_apply(self, func, arg, span);
    }
    fn visit_section(&mut self, section: &Section, span: Span) {
        walk_section(self, section, span);
    }
    fn visit_field_access(&mut self, expr: &SExpr, field: &FieldKind, span: Span) {
        walk_field_access(self, expr, field, span);
    }
    fn visit_block(&mut self, stmts: &[crate::ast::SStmt], span: Span) {
        walk_block(self, stmts, span);
    }
    fn visit_tuple(&mut self, elems: &[SExpr], span: Span) {
        walk_tuple(self, elems, span);
    }
    fn visit_list(&mut self, elems: &[ListElem], span: Span) {
        walk_list(self, elems, span);
    }
    fn visit_record(&mut self, fields: &[RecordField], span: Span) {
        walk_record(self, fields, span);
    }
    fn visit_map(&mut self, entries: &[MapEntry], span: Span) {
        walk_map(self, entries, span);
    }
    fn visit_func(&mut self, params: &[Param], ret_type: Option<&SType>, body: &SExpr, span: Span) {
        walk_func(self, params, ret_type, body, span);
    }
    fn visit_match(&mut self, scrutinee: &SExpr, arms: &[MatchArm], span: Span) {
        walk_match(self, scrutinee, arms, span);
    }
    fn visit_ternary(&mut self, cond: &SExpr, then_: &SExpr, else_: Option<&SExpr>, span: Span) {
        walk_ternary(self, cond, then_, else_, span);
    }
    fn visit_propagate(&mut self, inner: &SExpr, span: Span) {
        walk_propagate(self, inner, span);
    }
    fn visit_coalesce(&mut self, expr: &SExpr, default: &SExpr, span: Span) {
        walk_coalesce(self, expr, default, span);
    }
    fn visit_slice(
        &mut self,
        expr: &SExpr,
        start: Option<&SExpr>,
        end: Option<&SExpr>,
        span: Span,
    ) {
        walk_slice(self, expr, start, end, span);
    }
    fn visit_named_arg(&mut self, _name: &str, value: &SExpr, span: Span) {
        walk_named_arg(self, value, span);
    }
    fn visit_loop(&mut self, stmts: &[crate::ast::SStmt], span: Span) {
        walk_loop(self, stmts, span);
    }
    fn visit_break(&mut self, value: Option<&SExpr>, span: Span) {
        walk_break(self, value, span);
    }
    fn visit_assert(&mut self, expr: &SExpr, msg: Option<&SExpr>, span: Span) {
        walk_assert(self, expr, msg, span);
    }
    fn visit_par(&mut self, stmts: &[crate::ast::SStmt], span: Span) {
        walk_par(self, stmts, span);
    }
    fn visit_sel(&mut self, arms: &[SelArm], span: Span) {
        walk_sel(self, arms, span);
    }
    fn visit_agent_send(&mut self, target: &SExpr, msg: &SExpr, span: Span) {
        walk_agent_send(self, target, msg, span);
    }
    fn visit_agent_ask(&mut self, target: &SExpr, msg: &SExpr, span: Span) {
        walk_agent_ask(self, target, msg, span);
    }
    fn visit_emit(&mut self, value: &SExpr, span: Span) {
        walk_emit(self, value, span);
    }
    fn visit_yield(&mut self, value: &SExpr, span: Span) {
        walk_yield(self, value, span);
    }
    fn visit_with(
        &mut self,
        _name: &str,
        value: &SExpr,
        body: &[crate::ast::SStmt],
        _mutable: bool,
        span: Span,
    ) {
        walk_with(self, value, body, span);
    }
    fn visit_with_resource(
        &mut self,
        resources: &[(SExpr, String)],
        body: &[crate::ast::SStmt],
        span: Span,
    ) {
        walk_with_resource(self, resources, body, span);
    }
    fn visit_refine(&mut self, ctx: &RefineCtx<'_>, span: Span) {
        walk_refine(self, ctx, span);
    }
    fn visit_shell(&mut self, _mode: ShellMode, parts: &[StrPart], span: Span) {
        walk_shell(self, parts, span);
    }
    fn visit_pattern(&mut self, pattern: &Pattern, span: Span) {
        walk_pattern(self, pattern, span);
    }
    fn visit_pattern_literal(&mut self, _lit: &Literal, _span: Span) {}
    fn visit_pattern_bind(&mut self, _name: &str, _span: Span) {}
    fn visit_pattern_wildcard(&mut self, _span: Span) {}
    fn visit_pattern_tuple(&mut self, elems: &[SPattern], span: Span) {
        walk_pattern_tuple(self, elems, span);
    }
    fn visit_pattern_list(&mut self, elems: &[SPattern], _rest: Option<&str>, span: Span) {
        walk_pattern_list(self, elems, span);
    }
    fn visit_pattern_record(&mut self, fields: &[FieldPattern], _rest: Option<&str>, span: Span) {
        walk_pattern_record(self, fields, span);
    }
    fn visit_pattern_constructor(&mut self, _name: &str, args: &[SPattern], span: Span) {
        walk_pattern_constructor(self, args, span);
    }
    fn visit_type_expr(&mut self, type_expr: &TypeExpr, span: Span) {
        walk_type_expr(self, type_expr, span);
    }
    fn visit_type_named(&mut self, _name: &str, _span: Span) {}
    fn visit_type_var(&mut self, _name: &str, _span: Span) {}
    fn visit_type_applied(&mut self, _name: &str, args: &[SType], span: Span) {
        walk_type_applied(self, args, span);
    }
    fn visit_type_list(&mut self, inner: &SType, span: Span) {
        walk_type_list(self, inner, span);
    }
    fn visit_type_map(&mut self, key: &SType, value: &SType, span: Span) {
        walk_type_map(self, key, value, span);
    }
    fn visit_type_record(&mut self, fields: &[TypeField], span: Span) {
        walk_type_record(self, fields, span);
    }
    fn visit_type_tuple(&mut self, elems: &[SType], span: Span) {
        walk_type_tuple(self, elems, span);
    }
    fn visit_type_func(&mut self, param: &SType, ret: &SType, span: Span) {
        walk_type_func(self, param, ret, span);
    }
    fn visit_type_fallible(&mut self, ok: &SType, err: &SType, span: Span) {
        walk_type_fallible(self, ok, err, span);
    }
}
