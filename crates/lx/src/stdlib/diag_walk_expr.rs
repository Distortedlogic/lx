use crate::ast::{Expr, SExpr};
use crate::span::Span;
use crate::visitor::{AstVisitor, walk_expr};

use super::Walker;
use super::diag_helpers::{
    expr_label, extract_field_call_parts, extract_msg_label, extract_spawn_label,
    is_resource_action, is_resource_create, is_resource_module, resolve_target,
};

pub(super) fn visit_expr_diag(w: &mut Walker, expr: &Expr, span: Span) {
    match expr {
        Expr::AgentSend { target, msg } => {
            let to = resolve_target(w, &target.node);
            let ctx = w.context.clone();
            w.add_edge(&ctx, &to, extract_msg_label(&msg.node), "dashed");
        }
        Expr::AgentAsk { target, msg } => {
            let to = resolve_target(w, &target.node);
            let ctx = w.context.clone();
            w.add_edge(&ctx, &to, extract_msg_label(&msg.node), "solid");
        }
        Expr::Par(stmts) => {
            let fork_id = w.add_node("fork", "par".into(), "fork");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &fork_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = fork_id;
            w.walk_stmts(stmts);
            w.context = saved;
        }
        Expr::Sel(arms) => {
            let dec_id = w.add_node("sel", "sel".into(), "decision");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &dec_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = dec_id;
            for arm in arms {
                w.visit_expr(&arm.handler.node, arm.handler.span);
            }
            w.context = saved;
        }
        Expr::Match { scrutinee, arms } => {
            let label = format!("{}?", expr_label(&scrutinee.node));
            let dec_id = w.add_node("match", label, "decision");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &dec_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = dec_id;
            for arm in arms {
                w.visit_expr(&arm.body.node, arm.body.span);
            }
            w.context = saved;
        }
        Expr::Ternary { cond, then_, else_ } => {
            let label = format!("{}?", expr_label(&cond.node));
            let dec_id = w.add_node("cond", label, "decision");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &dec_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = dec_id.clone();
            w.visit_expr(&then_.node, then_.span);
            if let Some(e) = else_ {
                w.context = dec_id;
                w.visit_expr(&e.node, e.span);
            }
            w.context = saved;
        }
        Expr::Loop(stmts) => {
            let loop_id = w.add_node("loop", "loop".into(), "loop");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &loop_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = loop_id;
            w.walk_stmts(stmts);
            w.context = saved;
        }
        Expr::Refine {
            initial,
            grade,
            revise,
            ..
        } => {
            let refine_id = w.add_node("loop", "refine".into(), "loop");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &refine_id, String::new(), "solid");
            let saved = w.context.clone();
            w.context = refine_id;
            w.visit_expr(&initial.node, initial.span);
            w.visit_expr(&grade.node, grade.span);
            w.visit_expr(&revise.node, revise.span);
            w.context = saved;
        }
        Expr::Shell { .. } => {
            let id = w.add_node("io", "shell".into(), "io");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &id, String::new(), "solid");
        }
        Expr::Apply { func, arg } => {
            if let Some((module, method, args)) = uncurry_call(expr) {
                if let Some(kind) = classify_call(module, method) {
                    handle_call(w, module, method, kind, &args);
                    return;
                }
                if w.imported_modules.contains(module) {
                    let label = format!("{module}.{method}");
                    let id = w.add_node("tool", label, "tool");
                    let ctx = w.context.clone();
                    w.add_edge(&ctx, &id, String::new(), "solid");
                    for a in &args {
                        w.visit_expr(&a.node, a.span);
                    }
                    return;
                }
            }
            if let Some((name, fn_args)) = uncurry_fn_call(expr)
                && let Some(node_id) = w.fn_nodes.get(name).cloned()
            {
                let ctx = w.context.clone();
                w.add_edge(&ctx, &node_id, String::new(), "solid");
                for a in &fn_args {
                    w.visit_expr(&a.node, a.span);
                }
                return;
            }
            w.visit_expr(&func.node, func.span);
            w.visit_expr(&arg.node, arg.span);
        }
        _ => walk_expr(w, expr, span),
    }
}

fn uncurry_call(expr: &Expr) -> Option<(&str, &str, Vec<&SExpr>)> {
    let mut args = Vec::new();
    let mut current = expr;
    while let Expr::Apply { func, arg } = current {
        args.push(arg.as_ref());
        current = &func.node;
    }
    let (module, method) = extract_field_call_parts(current)?;
    args.reverse();
    Some((module, method, args))
}

fn uncurry_fn_call(expr: &Expr) -> Option<(&str, Vec<&SExpr>)> {
    let mut args = Vec::new();
    let mut current = expr;
    while let Expr::Apply { func, arg } = current {
        args.push(arg.as_ref());
        current = &func.node;
    }
    let Expr::Ident(name) = current else {
        return None;
    };
    args.reverse();
    Some((name.as_str(), args))
}

fn classify_call(module: &str, method: &str) -> Option<&'static str> {
    match module {
        "ai" => Some("tool"),
        "agent" => Some(match method {
            "dispatch" => "fork",
            "gate" => "decision",
            _ => "agent",
        }),
        "mcp" => Some("tool"),
        "pool" => Some("fork"),
        "saga" => Some("fork"),
        "plan" => Some("fork"),
        "retry" => Some("loop"),
        "cron" => Some("loop"),
        "circuit" => Some(match method {
            "check" => "decision",
            _ => "resource",
        }),
        "user" => Some("user"),
        "trace" | "knowledge" | "memory" | "budget" | "context" | "tasks" | "profile" => {
            if is_resource_create(method) {
                None
            } else {
                Some("resource")
            }
        }
        "http" | "fs" | "git" => Some("io"),
        _ => None,
    }
}

fn handle_call(w: &mut Walker, module: &str, method: &str, kind: &str, args: &[&SExpr]) {
    match (module, method) {
        ("agent", "kill") => {
            if let Some(first) = args.first() {
                let target = resolve_target(w, &first.node);
                let ctx = w.context.clone();
                w.add_edge(&ctx, &target, "kill".into(), "dashed");
            }
            return;
        }
        ("agent", "dispatch") => {
            let id = w.add_node("fork", "dispatch".into(), "fork");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &id, String::new(), "solid");
            if let Some(first) = args.first()
                && let Expr::Ident(var) = &first.node
                && let Some(entries) = w.handler_maps.get(var).cloned()
            {
                for entry in &entries {
                    if let Some(node_id) = w.fn_nodes.get(entry).cloned() {
                        w.add_edge(&id, &node_id, entry.clone(), "solid");
                    }
                }
            }
            return;
        }
        ("agent", "spawn") => {
            let label = args
                .first()
                .map(|a| extract_spawn_label(&a.node))
                .unwrap_or_else(|| "agent".into());
            let id = w.add_node("agent", label, "agent");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &id, String::new(), "solid");
            return;
        }
        ("mcp", "call") => {
            if let Some(first) = args.first()
                && let Expr::Ident(var) = &first.node
                && let Some(node_id) = w.mcp_vars.get(var).cloned()
            {
                let ctx = w.context.clone();
                w.add_edge(&ctx, &node_id, "call".into(), "solid");
                return;
            }
        }
        ("mcp", "connect") => {
            let label = args
                .first()
                .map(|a| extract_spawn_label(&a.node))
                .unwrap_or_else(|| "mcp".into());
            let id = w.add_node("tool", label, "tool");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &id, String::new(), "solid");
            return;
        }
        ("cron", "every") => {
            let id = w.add_node("loop", "cron".into(), "loop");
            let ctx = w.context.clone();
            w.add_edge(&ctx, &id, String::new(), "solid");
            if let Some(last) = args.last() {
                let saved = w.context.clone();
                w.context = id;
                w.visit_expr(&last.node, last.span);
                w.context = saved;
            }
            return;
        }
        _ => {}
    }
    if is_resource_module(module) && is_resource_action(method) {
        for arg in args {
            if let Expr::Ident(var) = &arg.node
                && let Some(node_id) = w.resource_vars.get(var).cloned()
            {
                let ctx = w.context.clone();
                w.add_edge(&ctx, &node_id, method.to_string(), "dashed");
                return;
            }
        }
    }

    let label = format!("{module}.{method}");
    let id = w.add_node(kind, label, kind);
    let ctx = w.context.clone();
    w.add_edge(&ctx, &id, String::new(), "solid");
    for arg in args {
        w.visit_expr(&arg.node, arg.span);
    }
}
