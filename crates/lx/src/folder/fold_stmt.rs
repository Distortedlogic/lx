use crate::ast::{
  AgentMethod, AstArena, BindTarget, Binding, ClassDeclData, ClassField, FieldDecl, Program, Stmt, StmtFieldUpdate, StmtId, TraitDeclData, TraitEntry,
  TraitMethodDecl,
};

use super::AstFolder;

pub fn fold_program<F: AstFolder + ?Sized, P>(f: &mut F, mut program: Program<P>) -> Program<P> {
  let stmts: Vec<StmtId> = program.stmts.clone();
  let folded: Vec<StmtId> = stmts.into_iter().map(|s| f.fold_stmt(s, &mut program.arena)).collect();
  program.stmts = folded;
  program
}

pub fn fold_stmt<F: AstFolder + ?Sized>(f: &mut F, id: StmtId, arena: &mut AstArena) -> StmtId {
  let span = arena.stmt_span(id);
  let stmt = arena.stmt(id).clone();
  match stmt {
    Stmt::Binding(binding) => f.fold_binding(id, binding, span, arena),
    Stmt::TypeDef(def) => f.fold_type_def(id, def, span, arena),
    Stmt::TraitUnion(def) => f.fold_trait_union(id, def, span, arena),
    Stmt::Use(u) => f.fold_use(id, u, span, arena),
    Stmt::TraitDecl(data) => fold_trait_decl(f, data, span, arena),
    Stmt::ClassDecl(data) => fold_class_decl(f, data, span, arena),
    Stmt::FieldUpdate(fu) => fold_field_update(f, fu, span, arena),
    Stmt::Expr(eid) => {
      let folded = f.fold_expr(eid, arena);
      if folded == eid {
        return id;
      }
      arena.alloc_stmt(Stmt::Expr(folded), span)
    },
  }
}

pub fn fold_binding<F: AstFolder + ?Sized>(f: &mut F, binding: Binding, span: miette::SourceSpan, arena: &mut AstArena) -> StmtId {
  let type_ann = binding.type_ann.map(|t| f.fold_type_expr(t, arena));
  let target = match binding.target {
    BindTarget::Pattern(pat) => BindTarget::Pattern(f.fold_pattern(pat, arena)),
    other => other,
  };
  let value = f.fold_expr(binding.value, arena);
  arena.alloc_stmt(Stmt::Binding(Binding { exported: binding.exported, mutable: binding.mutable, target, type_ann, value }), span)
}

fn fold_trait_decl<F: AstFolder + ?Sized>(f: &mut F, data: TraitDeclData, span: miette::SourceSpan, arena: &mut AstArena) -> StmtId {
  let entries = data
    .entries
    .into_iter()
    .map(|entry| match entry {
      TraitEntry::Field(field) => {
        let default = field.default.map(|d| f.fold_expr(d, arena));
        let constraint = field.constraint.map(|c| f.fold_expr(c, arena));
        TraitEntry::Field(Box::new(FieldDecl { name: field.name, type_name: field.type_name, default, constraint }))
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
          default: inp.default.map(|d| f.fold_expr(d, arena)),
          constraint: inp.constraint.map(|c| f.fold_expr(c, arena)),
        })
        .collect();
      TraitMethodDecl { name: method.name, input, output: method.output }
    })
    .collect();
  let defaults = data.defaults.into_iter().map(|m| AgentMethod { name: m.name, handler: f.fold_expr(m.handler, arena) }).collect();
  arena.alloc_stmt(
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

fn fold_class_decl<F: AstFolder + ?Sized>(f: &mut F, data: ClassDeclData, span: miette::SourceSpan, arena: &mut AstArena) -> StmtId {
  let fields = data.fields.into_iter().map(|field| ClassField { name: field.name, default: f.fold_expr(field.default, arena) }).collect();
  let methods = data.methods.into_iter().map(|m| AgentMethod { name: m.name, handler: f.fold_expr(m.handler, arena) }).collect();
  arena.alloc_stmt(Stmt::ClassDecl(ClassDeclData { name: data.name, traits: data.traits, fields, methods, exported: data.exported }), span)
}

fn fold_field_update<F: AstFolder + ?Sized>(f: &mut F, fu: StmtFieldUpdate, span: miette::SourceSpan, arena: &mut AstArena) -> StmtId {
  let value = f.fold_expr(fu.value, arena);
  arena.alloc_stmt(Stmt::FieldUpdate(StmtFieldUpdate { name: fu.name, fields: fu.fields, value }), span)
}
