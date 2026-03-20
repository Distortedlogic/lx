mod walk_helpers;
mod walk_pattern;

pub use walk_helpers::*;
pub use walk_pattern::*;

use crate::ast::{AgentMethod, Binding, ClassField, Expr, Program, ProtocolEntry, SExpr, Stmt};
use crate::span::Span;

use super::{AgentDeclCtx, AstVisitor, RefineCtx, TraitDeclCtx};

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
            defaults: _,
            requires,
            description,
            tags,
            exported: _,
        } => {
            let ctx = TraitDeclCtx {
                name,
                methods,
                requires,
                description: description.as_deref(),
                tags,
            };
            v.visit_trait_decl(&ctx, span);
        }
        Stmt::AgentDecl {
            name,
            traits,
            uses,
            init,
            on,
            methods,
            exported: _,
        } => {
            let ctx = AgentDeclCtx {
                name,
                traits,
                uses,
                init: init.as_ref(),
                on: on.as_ref(),
                methods,
            };
            v.visit_agent_decl(&ctx, span);
        }
        Stmt::ClassDecl {
            name,
            traits,
            fields,
            methods,
            exported,
        } => {
            v.visit_class_decl(name, traits, fields, methods, *exported, span);
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

pub fn walk_agent_decl<V: AstVisitor + ?Sized>(v: &mut V, ctx: &AgentDeclCtx<'_>, _span: Span) {
    if let Some(i) = ctx.init {
        v.visit_expr(&i.node, i.span);
    }
    if let Some(o) = ctx.on {
        v.visit_expr(&o.node, o.span);
    }
    for m in ctx.methods {
        v.visit_expr(&m.handler.node, m.handler.span);
    }
}

pub fn walk_class_decl<V: AstVisitor + ?Sized>(
    v: &mut V,
    fields: &[ClassField],
    methods: &[AgentMethod],
    _span: Span,
) {
    for f in fields {
        v.visit_expr(&f.default.node, f.default.span);
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
        Expr::StreamAsk { target, msg } => v.visit_stream_ask(target, msg, span),
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
            let ctx = RefineCtx {
                initial,
                grade,
                revise,
                threshold,
                max_rounds,
                on_round: on_round.as_deref(),
            };
            v.visit_refine(&ctx, span);
        }
        Expr::Shell { mode, parts } => v.visit_shell(*mode, parts, span),
        Expr::Receive(arms) => {
            for arm in arms {
                v.visit_expr(&arm.handler.node, arm.handler.span);
            }
        }
    }
}
