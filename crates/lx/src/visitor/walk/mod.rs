mod walk_pattern;

pub use walk_pattern::*;

use crate::ast::{
    AgentMethod, Binding, Expr, FieldKind, ListElem, Literal, MapEntry, Program, ProtocolEntry,
    RecordField, SExpr, SStmt, Section, Stmt, StrPart,
};
use crate::span::Span;

use super::AstVisitor;

pub fn walk_program<V: AstVisitor + ?Sized>(v: &mut V, program: &Program) {
    for stmt in &program.stmts {
        v.visit_stmt(&stmt.node, stmt.span);
    }
}

pub fn walk_stmt<V: AstVisitor + ?Sized>(v: &mut V, stmt: &Stmt, span: Span) {
    match stmt {
        Stmt::Binding(binding) => v.visit_binding(binding, span),
        Stmt::TypeDef {
            name,
            variants,
            exported,
        } => {
            v.visit_type_def(name, variants, *exported, span);
        }
        Stmt::Protocol {
            name,
            entries,
            exported,
        } => {
            v.visit_protocol(name, entries, *exported, span);
        }
        Stmt::ProtocolUnion(def) => v.visit_protocol_union(def, span),
        Stmt::McpDecl {
            name,
            tools,
            exported,
        } => {
            v.visit_mcp_decl(name, tools, *exported, span);
        }
        Stmt::TraitDecl {
            name,
            methods,
            requires,
            description,
            tags,
            exported,
        } => {
            v.visit_trait_decl(
                name,
                methods,
                requires,
                description.as_deref(),
                tags,
                *exported,
                span,
            );
        }
        Stmt::AgentDecl {
            name,
            traits,
            uses,
            init,
            on,
            methods,
            exported,
        } => {
            v.visit_agent_decl(
                name,
                traits,
                uses,
                init.as_ref(),
                on.as_ref(),
                methods,
                *exported,
                span,
            );
        }
        Stmt::FieldUpdate {
            name,
            fields,
            value,
        } => {
            v.visit_field_update(name, fields, value, span);
        }
        Stmt::Use(use_stmt) => v.visit_use(use_stmt, span),
        Stmt::Expr(sexpr) => v.visit_expr(&sexpr.node, sexpr.span),
    }
}

pub fn walk_binding<V: AstVisitor + ?Sized>(v: &mut V, binding: &Binding, _span: Span) {
    if let Some(ref ty) = binding.type_ann {
        v.visit_type_expr(&ty.node, ty.span);
    }
    v.visit_expr(&binding.value.node, binding.value.span);
}

pub fn walk_protocol<V: AstVisitor + ?Sized>(v: &mut V, entries: &[ProtocolEntry], _span: Span) {
    for entry in entries {
        if let ProtocolEntry::Field(f) = entry {
            if let Some(ref d) = f.default {
                v.visit_expr(&d.node, d.span);
            }
            if let Some(ref c) = f.constraint {
                v.visit_expr(&c.node, c.span);
            }
        }
    }
}

pub fn walk_agent_decl<V: AstVisitor + ?Sized>(
    v: &mut V,
    init: Option<&SExpr>,
    on: Option<&SExpr>,
    methods: &[AgentMethod],
    _span: Span,
) {
    if let Some(i) = init {
        v.visit_expr(&i.node, i.span);
    }
    if let Some(o) = on {
        v.visit_expr(&o.node, o.span);
    }
    for m in methods {
        v.visit_expr(&m.handler.node, m.handler.span);
    }
}

pub fn walk_field_update<V: AstVisitor + ?Sized>(v: &mut V, value: &SExpr, _span: Span) {
    v.visit_expr(&value.node, value.span);
}

pub fn walk_expr<V: AstVisitor + ?Sized>(v: &mut V, expr: &Expr, span: Span) {
    match expr {
        Expr::Literal(lit) => v.visit_literal(lit, span),
        Expr::Ident(name) => v.visit_ident(name, span),
        Expr::TypeConstructor(name) => v.visit_type_constructor(name, span),
        Expr::Binary { op, left, right } => v.visit_binary(*op, left, right, span),
        Expr::Unary { op, operand } => v.visit_unary(*op, operand, span),
        Expr::Pipe { left, right } => v.visit_pipe(left, right, span),
        Expr::Apply { func, arg } => v.visit_apply(func, arg, span),
        Expr::Section(section) => v.visit_section(section, span),
        Expr::FieldAccess { expr: e, field } => v.visit_field_access(e, field, span),
        Expr::Block(stmts) => v.visit_block(stmts, span),
        Expr::Tuple(elems) => v.visit_tuple(elems, span),
        Expr::List(elems) => v.visit_list(elems, span),
        Expr::Record(fields) => v.visit_record(fields, span),
        Expr::Map(entries) => v.visit_map(entries, span),
        Expr::Func {
            params,
            ret_type,
            body,
        } => {
            v.visit_func(params, ret_type.as_ref(), body, span);
        }
        Expr::Match { scrutinee, arms } => v.visit_match(scrutinee, arms, span),
        Expr::Ternary { cond, then_, else_ } => {
            v.visit_ternary(cond, then_, else_.as_deref(), span);
        }
        Expr::Propagate(inner) => v.visit_propagate(inner, span),
        Expr::Coalesce { expr: e, default } => v.visit_coalesce(e, default, span),
        Expr::Slice {
            expr: e,
            start,
            end,
        } => {
            v.visit_slice(e, start.as_deref(), end.as_deref(), span);
        }
        Expr::NamedArg { name, value } => v.visit_named_arg(name, value, span),
        Expr::Loop(stmts) => v.visit_loop(stmts, span),
        Expr::Break(val) => v.visit_break(val.as_deref(), span),
        Expr::Assert { expr: e, msg } => v.visit_assert(e, msg.as_deref(), span),
        Expr::Par(stmts) => v.visit_par(stmts, span),
        Expr::Sel(arms) => v.visit_sel(arms, span),
        Expr::AgentSend { target, msg } => v.visit_agent_send(target, msg, span),
        Expr::AgentAsk { target, msg } => v.visit_agent_ask(target, msg, span),
        Expr::Emit { value } => v.visit_emit(value, span),
        Expr::Yield { value } => v.visit_yield(value, span),
        Expr::With {
            name,
            value,
            body,
            mutable,
        } => {
            v.visit_with(name, value, body, *mutable, span);
        }
        Expr::WithResource { resources, body } => {
            v.visit_with_resource(resources, body, span);
        }
        Expr::Refine {
            initial,
            grade,
            revise,
            threshold,
            max_rounds,
            on_round,
        } => {
            v.visit_refine(
                initial,
                grade,
                revise,
                threshold,
                max_rounds,
                on_round.as_deref(),
                span,
            );
        }
        Expr::Shell { mode, parts } => v.visit_shell(*mode, parts, span),
        Expr::Receive(arms) => {
            for arm in arms {
                v.visit_expr(&arm.handler.node, arm.handler.span);
            }
        }
    }
}

pub fn walk_literal<V: AstVisitor + ?Sized>(v: &mut V, lit: &Literal, _span: Span) {
    if let Literal::Str(parts) = lit {
        for part in parts {
            if let StrPart::Interp(e) = part {
                v.visit_expr(&e.node, e.span);
            }
        }
    }
}

pub fn walk_binary<V: AstVisitor + ?Sized>(v: &mut V, left: &SExpr, right: &SExpr, _span: Span) {
    v.visit_expr(&left.node, left.span);
    v.visit_expr(&right.node, right.span);
}

pub fn walk_unary<V: AstVisitor + ?Sized>(v: &mut V, operand: &SExpr, _span: Span) {
    v.visit_expr(&operand.node, operand.span);
}

pub fn walk_pipe<V: AstVisitor + ?Sized>(v: &mut V, left: &SExpr, right: &SExpr, _span: Span) {
    v.visit_expr(&left.node, left.span);
    v.visit_expr(&right.node, right.span);
}

pub fn walk_apply<V: AstVisitor + ?Sized>(v: &mut V, func: &SExpr, arg: &SExpr, _span: Span) {
    v.visit_expr(&func.node, func.span);
    v.visit_expr(&arg.node, arg.span);
}

pub fn walk_section<V: AstVisitor + ?Sized>(v: &mut V, section: &Section, _span: Span) {
    match section {
        Section::Right { operand, .. } | Section::Left { operand, .. } => {
            v.visit_expr(&operand.node, operand.span);
        }
        _ => {}
    }
}

pub fn walk_field_access<V: AstVisitor + ?Sized>(
    v: &mut V,
    expr: &SExpr,
    field: &FieldKind,
    _span: Span,
) {
    v.visit_expr(&expr.node, expr.span);
    if let FieldKind::Computed(c) = field {
        v.visit_expr(&c.node, c.span);
    }
}

pub fn walk_block<V: AstVisitor + ?Sized>(v: &mut V, stmts: &[SStmt], _span: Span) {
    for s in stmts {
        v.visit_stmt(&s.node, s.span);
    }
}

pub fn walk_tuple<V: AstVisitor + ?Sized>(v: &mut V, elems: &[SExpr], _span: Span) {
    for e in elems {
        v.visit_expr(&e.node, e.span);
    }
}

pub fn walk_list<V: AstVisitor + ?Sized>(v: &mut V, elems: &[ListElem], _span: Span) {
    for e in elems {
        match e {
            ListElem::Single(se) | ListElem::Spread(se) => v.visit_expr(&se.node, se.span),
        }
    }
}

pub fn walk_record<V: AstVisitor + ?Sized>(v: &mut V, fields: &[RecordField], _span: Span) {
    for f in fields {
        v.visit_expr(&f.value.node, f.value.span);
    }
}

pub fn walk_map<V: AstVisitor + ?Sized>(v: &mut V, entries: &[MapEntry], _span: Span) {
    for e in entries {
        if let Some(ref k) = e.key {
            v.visit_expr(&k.node, k.span);
        }
        v.visit_expr(&e.value.node, e.value.span);
    }
}
