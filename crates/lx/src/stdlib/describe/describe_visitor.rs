use crate::ast::{BindTarget, Binding, McpToolDecl, SExpr, UseKind, UseStmt};
use crate::span::Span;
use crate::visitor::{
    AgentDeclCtx, AstVisitor, RefineCtx, walk_apply, walk_loop, walk_match, walk_par, walk_refine,
    walk_sel,
};

use super::describe_helpers::{
    expr_name, extract_agent_spawn, extract_ai_call, extract_ai_call_from_apply,
    extract_mcp_connect, extract_msg_label,
};

pub(super) struct ImportInfo {
    pub path: String,
    pub kind: String,
}

pub(super) struct AgentInfo {
    pub name: String,
    pub traits: Vec<String>,
    pub methods: Vec<String>,
    pub declared: bool,
    pub spawned_by: String,
}

pub(super) struct MessageInfo {
    pub from: String,
    pub to: String,
    pub style: String,
    pub label: String,
}

pub(super) struct ControlFlowInfo {
    pub kind: String,
    pub label: String,
}

pub(super) struct ResourceInfo {
    pub kind: String,
    pub name: String,
    pub source: String,
}

pub(super) struct AiCallInfo {
    pub context: String,
}

pub(super) struct ProgramDescription {
    pub imports: Vec<ImportInfo>,
    pub agents: Vec<AgentInfo>,
    pub messages: Vec<MessageInfo>,
    pub control_flow: Vec<ControlFlowInfo>,
    pub resources: Vec<ResourceInfo>,
    pub ai_calls: Vec<AiCallInfo>,
    pub exports: Vec<String>,
}

pub(super) struct Describer {
    imports: Vec<ImportInfo>,
    agents: Vec<AgentInfo>,
    messages: Vec<MessageInfo>,
    control_flow: Vec<ControlFlowInfo>,
    resources: Vec<ResourceInfo>,
    ai_calls: Vec<AiCallInfo>,
    exports: Vec<String>,
    context_stack: Vec<String>,
}

impl Describer {
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
            agents: Vec::new(),
            messages: Vec::new(),
            control_flow: Vec::new(),
            resources: Vec::new(),
            ai_calls: Vec::new(),
            exports: Vec::new(),
            context_stack: vec!["main".into()],
        }
    }

    pub fn into_description(self) -> ProgramDescription {
        ProgramDescription {
            imports: self.imports,
            agents: self.agents,
            messages: self.messages,
            control_flow: self.control_flow,
            resources: self.resources,
            ai_calls: self.ai_calls,
            exports: self.exports,
        }
    }

    fn current_context(&self) -> &str {
        self.context_stack.last().map_or("main", |s| s.as_str())
    }
}

impl AstVisitor for Describer {
    fn visit_use(&mut self, stmt: &UseStmt, _span: Span) {
        let path = stmt.path.join("/");
        let kind = match &stmt.kind {
            UseKind::Whole => "whole".to_string(),
            UseKind::Alias(a) => format!("alias:{a}"),
            UseKind::Selective(items) => format!("selective:{}", items.join(",")),
        };
        self.imports.push(ImportInfo { path, kind });
    }

    fn visit_binding(&mut self, binding: &Binding, _span: Span) {
        let name = match &binding.target {
            BindTarget::Name(n) | BindTarget::Reassign(n) => Some(n.clone()),
            _ => None,
        };
        if let Some(ref var_name) = name {
            if binding.exported {
                self.exports.push(var_name.clone());
            }
            if let Some(spawn_label) = extract_agent_spawn(&binding.value) {
                self.agents.push(AgentInfo {
                    name: spawn_label,
                    traits: Vec::new(),
                    methods: Vec::new(),
                    declared: false,
                    spawned_by: self.current_context().to_string(),
                });
                return;
            }
            if let Some(mcp_label) = extract_mcp_connect(&binding.value) {
                self.resources.push(ResourceInfo {
                    kind: "mcp".into(),
                    name: mcp_label,
                    source: "connect".into(),
                });
                return;
            }
            if let Some(ctx) = extract_ai_call(&binding.value) {
                self.ai_calls.push(AiCallInfo { context: ctx });
                return;
            }
            let _ = var_name;
        }
        self.visit_expr(&binding.value.node, binding.value.span);
    }

    fn visit_agent_decl(&mut self, ctx: &AgentDeclCtx<'_>, _span: Span) {
        let method_names: Vec<String> = ctx.methods.iter().map(|m| m.name.clone()).collect();
        self.agents.push(AgentInfo {
            name: ctx.name.to_string(),
            traits: ctx.traits.to_vec(),
            methods: method_names,
            declared: true,
            spawned_by: String::new(),
        });
    }

    fn visit_mcp_decl(&mut self, name: &str, _tools: &[McpToolDecl], exported: bool, _span: Span) {
        self.resources.push(ResourceInfo {
            kind: "mcp".into(),
            name: name.to_string(),
            source: "declared".into(),
        });
        if exported {
            self.exports.push(name.to_string());
        }
    }

    fn visit_agent_send(&mut self, target: &SExpr, msg: &SExpr, _span: Span) {
        self.messages.push(MessageInfo {
            from: self.current_context().to_string(),
            to: expr_name(&target.node),
            style: "send".into(),
            label: extract_msg_label(&msg.node),
        });
    }

    fn visit_agent_ask(&mut self, target: &SExpr, msg: &SExpr, _span: Span) {
        self.messages.push(MessageInfo {
            from: self.current_context().to_string(),
            to: expr_name(&target.node),
            style: "ask".into(),
            label: extract_msg_label(&msg.node),
        });
    }

    fn visit_par(&mut self, stmts: &[crate::ast::SStmt], span: Span) {
        self.control_flow.push(ControlFlowInfo {
            kind: "par".into(),
            label: format!("{} branches", stmts.len()),
        });
        walk_par(self, stmts, span);
    }

    fn visit_sel(&mut self, arms: &[crate::ast::SelArm], span: Span) {
        self.control_flow.push(ControlFlowInfo {
            kind: "sel".into(),
            label: format!("{} arms", arms.len()),
        });
        walk_sel(self, arms, span);
    }

    fn visit_match(&mut self, scrutinee: &SExpr, arms: &[crate::ast::MatchArm], span: Span) {
        self.control_flow.push(ControlFlowInfo {
            kind: "match".into(),
            label: format!("on {}", expr_name(&scrutinee.node)),
        });
        walk_match(self, scrutinee, arms, span);
    }

    fn visit_refine(&mut self, ctx: &RefineCtx<'_>, span: Span) {
        self.control_flow.push(ControlFlowInfo {
            kind: "refine".into(),
            label: "grade/revise loop".into(),
        });
        walk_refine(self, ctx, span);
    }

    fn visit_loop(&mut self, stmts: &[crate::ast::SStmt], span: Span) {
        self.control_flow.push(ControlFlowInfo {
            kind: "loop".into(),
            label: String::new(),
        });
        walk_loop(self, stmts, span);
    }

    fn visit_with_resource(
        &mut self,
        resources: &[(SExpr, String)],
        body: &[crate::ast::SStmt],
        span: Span,
    ) {
        for (_, name) in resources {
            self.resources.push(ResourceInfo {
                kind: "scoped".into(),
                name: name.clone(),
                source: "with-resource".into(),
            });
        }
        crate::visitor::walk_with_resource(self, resources, body, span);
    }

    fn visit_apply(&mut self, func: &SExpr, arg: &SExpr, span: Span) {
        if let Some(ctx) = extract_ai_call_from_apply(&func.node) {
            self.ai_calls.push(AiCallInfo { context: ctx });
            return;
        }
        walk_apply(self, func, arg, span);
    }
}
