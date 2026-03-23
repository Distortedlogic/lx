mod walk_transform_expr;
mod walk_transform_expr2;
mod walk_transform_pattern;
mod walk_transform_type;

pub use walk_transform_expr::walk_transform_expr;
pub use walk_transform_pattern::walk_transform_pattern;
pub use walk_transform_type::walk_transform_type_expr;

use crate::ast::{
  AgentMethod, AstArena, BindTarget, Binding, ClassDeclData, ClassField, FieldDecl, Program, Stmt, StmtFieldUpdate, StmtId, TraitDeclData, TraitEntry,
  TraitMethodDecl,
};

use super::transformer::{AstTransformer, TransformOp};

pub fn walk_transform_program<T: AstTransformer + ?Sized, P>(t: &mut T, mut program: Program<P>) -> Program<P> {
  let stmts: Vec<StmtId> = program.stmts.clone();
  let folded: Vec<StmtId> = stmts.into_iter().map(|s| walk_transform_stmt(t, s, &mut program.arena)).collect();
  program.stmts = folded;
  program
}

pub fn walk_transform_stmt<T: AstTransformer + ?Sized>(t: &mut T, id: StmtId, arena: &mut AstArena) -> StmtId {
  let span = arena.stmt_span(id);
  let original = arena.stmt(id).clone();

  match t.transform_stmt(id, original.clone(), span, arena) {
    TransformOp::Stop => id,
    TransformOp::Skip(node) => {
      let final_node = t.leave_stmt(id, node, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_stmt(final_node, span)
    },
    TransformOp::Continue(node) => {
      let recursed = recurse_stmt_children(t, node, arena);
      let final_node = t.leave_stmt(id, recursed, span, arena);
      if final_node == original {
        return id;
      }
      arena.alloc_stmt(final_node, span)
    },
  }
}

fn recurse_stmt_children<T: AstTransformer + ?Sized>(t: &mut T, stmt: Stmt, arena: &mut AstArena) -> Stmt {
  match stmt {
    Stmt::Binding(binding) => {
      let type_ann = binding.type_ann.map(|te| walk_transform_type_expr(t, te, arena));
      let target = match binding.target {
        BindTarget::Pattern(pid) => BindTarget::Pattern(walk_transform_pattern(t, pid, arena)),
        other => other,
      };
      let value = walk_transform_expr(t, binding.value, arena);
      Stmt::Binding(Binding { exported: binding.exported, mutable: binding.mutable, target, type_ann, value })
    },
    Stmt::TraitDecl(data) => Stmt::TraitDecl(recurse_trait_decl(t, data, arena)),
    Stmt::ClassDecl(data) => Stmt::ClassDecl(recurse_class_decl(t, data, arena)),
    Stmt::FieldUpdate(fu) => {
      let value = walk_transform_expr(t, fu.value, arena);
      Stmt::FieldUpdate(StmtFieldUpdate { name: fu.name, fields: fu.fields, value })
    },
    Stmt::Expr(eid) => Stmt::Expr(walk_transform_expr(t, eid, arena)),
    other @ (Stmt::TypeDef(_) | Stmt::TraitUnion(_) | Stmt::Use(_)) => other,
  }
}

fn recurse_trait_decl<T: AstTransformer + ?Sized>(t: &mut T, data: TraitDeclData, arena: &mut AstArena) -> TraitDeclData {
  let entries = data
    .entries
    .into_iter()
    .map(|entry| match entry {
      TraitEntry::Field(field) => {
        let default = field.default.map(|d| walk_transform_expr(t, d, arena));
        let constraint = field.constraint.map(|c| walk_transform_expr(t, c, arena));
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
          default: inp.default.map(|d| walk_transform_expr(t, d, arena)),
          constraint: inp.constraint.map(|c| walk_transform_expr(t, c, arena)),
        })
        .collect();
      TraitMethodDecl { name: method.name, input, output: method.output }
    })
    .collect();
  let defaults = data.defaults.into_iter().map(|m| AgentMethod { name: m.name, handler: walk_transform_expr(t, m.handler, arena) }).collect();
  TraitDeclData {
    name: data.name,
    entries,
    methods,
    defaults,
    requires: data.requires,
    description: data.description,
    tags: data.tags,
    exported: data.exported,
  }
}

fn recurse_class_decl<T: AstTransformer + ?Sized>(t: &mut T, data: ClassDeclData, arena: &mut AstArena) -> ClassDeclData {
  let fields = data.fields.into_iter().map(|f| ClassField { name: f.name, default: walk_transform_expr(t, f.default, arena) }).collect();
  let methods = data.methods.into_iter().map(|m| AgentMethod { name: m.name, handler: walk_transform_expr(t, m.handler, arena) }).collect();
  ClassDeclData { name: data.name, traits: data.traits, fields, methods, exported: data.exported }
}
