use crate::ast::{
    AgentMethod, BindTarget, Binding, ClassDeclData, ClassField, FieldDecl,
    Program, SStmt, Stmt, StmtFieldUpdate, TraitDeclData, TraitEntry,
    TraitMethodDecl,
};
use miette::SourceSpan;

use super::AstFolder;

pub fn fold_program<F: AstFolder + ?Sized>(f: &mut F, program: Program) -> Program {
    let stmts = program
        .stmts
        .into_iter()
        .map(|s| f.fold_stmt(s.node, s.span))
        .collect();
    Program { stmts }
}

pub fn fold_stmt<F: AstFolder + ?Sized>(f: &mut F, stmt: Stmt, span: SourceSpan) -> SStmt {
    match stmt {
        Stmt::Binding(binding) => f.fold_binding(binding, span),
        Stmt::TypeDef(td) => SStmt::new(Stmt::TypeDef(td), span),
        Stmt::TraitUnion(tu) => SStmt::new(Stmt::TraitUnion(tu), span),
        Stmt::TraitDecl(data) => fold_trait_decl(f, data, span),
        Stmt::ClassDecl(data) => fold_class_decl(f, data, span),
        Stmt::FieldUpdate(fu) => fold_field_update(f, fu, span),
        Stmt::Use(u) => SStmt::new(Stmt::Use(u), span),
        Stmt::Expr(sexpr) => {
            let folded = f.fold_expr(sexpr.node, sexpr.span);
            SStmt::new(Stmt::Expr(folded), span)
        },
    }
}

pub fn fold_binding<F: AstFolder + ?Sized>(
    f: &mut F,
    binding: Binding,
    span: SourceSpan,
) -> SStmt {
    let type_ann = binding.type_ann.map(|t| f.fold_type_expr(t.node, t.span));
    let target = match binding.target {
        BindTarget::Pattern(pat) => {
            BindTarget::Pattern(f.fold_pattern(pat.node, pat.span))
        },
        other => other,
    };
    let value = f.fold_expr(binding.value.node, binding.value.span);
    SStmt::new(
        Stmt::Binding(Binding {
            exported: binding.exported,
            mutable: binding.mutable,
            target,
            type_ann,
            value,
        }),
        span,
    )
}

fn fold_trait_decl<F: AstFolder + ?Sized>(
    f: &mut F,
    data: TraitDeclData,
    span: SourceSpan,
) -> SStmt {
    let entries = data
        .entries
        .into_iter()
        .map(|entry| match entry {
            TraitEntry::Field(field) => {
                let default = field.default.map(|d| f.fold_expr(d.node, d.span));
                let constraint =
                    field.constraint.map(|c| f.fold_expr(c.node, c.span));
                TraitEntry::Field(Box::new(FieldDecl {
                    name: field.name,
                    type_name: field.type_name,
                    default,
                    constraint,
                }))
            },
            other => other,
        })
        .collect();
    let methods = data
        .methods
        .into_iter()
        .map(|method| {
            let input = method
                .input
                .into_iter()
                .map(|inp| FieldDecl {
                    name: inp.name,
                    type_name: inp.type_name,
                    default: inp.default.map(|d| f.fold_expr(d.node, d.span)),
                    constraint: inp.constraint.map(|c| f.fold_expr(c.node, c.span)),
                })
                .collect();
            TraitMethodDecl { name: method.name, input, output: method.output }
        })
        .collect();
    let defaults = data
        .defaults
        .into_iter()
        .map(|m| AgentMethod {
            name: m.name,
            handler: f.fold_expr(m.handler.node, m.handler.span),
        })
        .collect();
    SStmt::new(
        Stmt::TraitDecl(TraitDeclData {
            name: data.name,
            entries,
            methods,
            defaults,
            requires: data.requires,
            description: data.description,
            tags: data.tags,
            exported: data.exported,
        }),
        span,
    )
}

fn fold_class_decl<F: AstFolder + ?Sized>(
    f: &mut F,
    data: ClassDeclData,
    span: SourceSpan,
) -> SStmt {
    let fields = data
        .fields
        .into_iter()
        .map(|field| ClassField {
            name: field.name,
            default: f.fold_expr(field.default.node, field.default.span),
        })
        .collect();
    let methods = data
        .methods
        .into_iter()
        .map(|m| AgentMethod {
            name: m.name,
            handler: f.fold_expr(m.handler.node, m.handler.span),
        })
        .collect();
    SStmt::new(
        Stmt::ClassDecl(ClassDeclData {
            name: data.name,
            traits: data.traits,
            fields,
            methods,
            exported: data.exported,
        }),
        span,
    )
}

fn fold_field_update<F: AstFolder + ?Sized>(
    f: &mut F,
    fu: StmtFieldUpdate,
    span: SourceSpan,
) -> SStmt {
    let value = f.fold_expr(fu.value.node, fu.value.span);
    SStmt::new(
        Stmt::FieldUpdate(StmtFieldUpdate { name: fu.name, fields: fu.fields, value }),
        span,
    )
}
