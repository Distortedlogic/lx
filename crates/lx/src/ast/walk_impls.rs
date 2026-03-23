use super::{AgentMethod, AstArena, ClassDeclData, ClassField, FieldDecl, NodeId, TraitDeclData, TraitEntry, TraitMethodDecl, WithKind};
use crate::visitor::transformer::AstTransformer;
use crate::visitor::walk_transform::walk_transform_expr;

impl WithKind {
  pub fn recurse_children<T: AstTransformer + ?Sized>(self, t: &mut T, arena: &mut AstArena) -> Self {
    match self {
      WithKind::Binding { name, value, mutable } => WithKind::Binding { name, value: walk_transform_expr(t, value, arena), mutable },
      WithKind::Resources { resources } => {
        let folded = resources.into_iter().map(|(e, sym)| (walk_transform_expr(t, e, arena), sym)).collect();
        WithKind::Resources { resources: folded }
      },
      WithKind::Context { fields } => {
        let folded = fields.into_iter().map(|(sym, e)| (sym, walk_transform_expr(t, e, arena))).collect();
        WithKind::Context { fields: folded }
      },
    }
  }

  pub fn children(&self) -> Vec<NodeId> {
    match self {
      WithKind::Binding { value, .. } => vec![NodeId::Expr(*value)],
      WithKind::Resources { resources } => resources.iter().map(|(e, _)| NodeId::Expr(*e)).collect(),
      WithKind::Context { fields } => fields.iter().map(|(_, e)| NodeId::Expr(*e)).collect(),
    }
  }
}

impl TraitDeclData {
  pub fn recurse_children<T: AstTransformer + ?Sized>(self, t: &mut T, arena: &mut AstArena) -> Self {
    let entries = self
      .entries
      .into_iter()
      .map(|entry| match entry {
        TraitEntry::Field(field) => TraitEntry::Field(Box::new(recurse_field_decl(t, &field, arena))),
        other => other,
      })
      .collect();
    let methods = self
      .methods
      .into_iter()
      .map(|method| {
        let input = method.input.into_iter().map(|inp| recurse_field_decl(t, &inp, arena)).collect();
        TraitMethodDecl { name: method.name, input, output: method.output }
      })
      .collect();
    let defaults = recurse_agent_methods(t, self.defaults, arena);
    TraitDeclData {
      name: self.name,
      type_params: self.type_params,
      entries,
      methods,
      defaults,
      requires: self.requires,
      description: self.description,
      tags: self.tags,
      exported: self.exported,
    }
  }

  pub fn children(&self) -> Vec<NodeId> {
    let mut result = Vec::new();
    for entry in &self.entries {
      if let TraitEntry::Field(f) = entry {
        result.extend(f.default.iter().map(|id| NodeId::Expr(*id)));
        result.extend(f.constraint.iter().map(|id| NodeId::Expr(*id)));
      }
    }
    for method in &self.methods {
      for input in &method.input {
        result.extend(input.default.iter().map(|id| NodeId::Expr(*id)));
        result.extend(input.constraint.iter().map(|id| NodeId::Expr(*id)));
      }
    }
    for d in &self.defaults {
      result.push(NodeId::Expr(d.handler));
    }
    result
  }
}

impl ClassDeclData {
  pub fn recurse_children<T: AstTransformer + ?Sized>(self, t: &mut T, arena: &mut AstArena) -> Self {
    let fields = self.fields.into_iter().map(|f| ClassField { name: f.name, default: walk_transform_expr(t, f.default, arena) }).collect();
    let methods = recurse_agent_methods(t, self.methods, arena);
    ClassDeclData { name: self.name, type_params: self.type_params, traits: self.traits, fields, methods, exported: self.exported }
  }

  pub fn children(&self) -> Vec<NodeId> {
    let mut result = Vec::new();
    for f in &self.fields {
      result.push(NodeId::Expr(f.default));
    }
    for m in &self.methods {
      result.push(NodeId::Expr(m.handler));
    }
    result
  }
}

fn recurse_field_decl<T: AstTransformer + ?Sized>(t: &mut T, field: &FieldDecl, arena: &mut AstArena) -> FieldDecl {
  FieldDecl {
    name: field.name,
    type_name: field.type_name,
    default: field.default.map(|d| walk_transform_expr(t, d, arena)),
    constraint: field.constraint.map(|c| walk_transform_expr(t, c, arena)),
  }
}

fn recurse_agent_methods<T: AstTransformer + ?Sized>(t: &mut T, methods: Vec<AgentMethod>, arena: &mut AstArena) -> Vec<AgentMethod> {
  methods.into_iter().map(|m| AgentMethod { name: m.name, handler: walk_transform_expr(t, m.handler, arena) }).collect()
}
