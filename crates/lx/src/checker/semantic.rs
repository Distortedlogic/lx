use std::collections::HashMap;

use la_arena::ArenaMap;

use crate::ast::ExprId;
use crate::sym::Sym;
use miette::SourceSpan;

use super::type_arena::{TypeArena, TypeId};

pub type ScopeId = usize;
pub type DefinitionId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
  Module,
  Function,
  Block,
  MatchArm,
  Loop,
  Par,
  With,
}

#[derive(Clone, Copy)]
pub enum DefKind {
  Binding,
  FuncParam,
  PatternBind,
  Import,
  TypeDef,
  TraitDef,
  ClassDef,
  WithBinding,
  ResourceBinding,
}

pub struct Scope {
  pub parent: Option<ScopeId>,
  pub span: SourceSpan,
  pub kind: ScopeKind,
}

pub struct DefinitionInfo {
  pub name: Sym,
  pub kind: DefKind,
  pub span: SourceSpan,
  pub ty: Option<TypeId>,
  pub scope: ScopeId,
  pub mutable: bool,
}

pub struct Reference {
  pub expr_id: ExprId,
  pub definition: DefinitionId,
}

pub struct SemanticModel {
  pub scopes: Vec<Scope>,
  pub definitions: Vec<DefinitionInfo>,
  pub references: Vec<Reference>,
  pub expr_types: ArenaMap<ExprId, TypeId>,
  pub type_defs: HashMap<Sym, Vec<Sym>>,
  pub trait_fields: HashMap<Sym, Vec<(Sym, TypeId)>>,
  pub type_arena: TypeArena,
}

impl SemanticModel {
  pub fn type_of_expr(&self, id: ExprId) -> Option<TypeId> {
    self.expr_types.get(id).copied()
  }

  pub fn type_of_def(&self, id: DefinitionId) -> Option<TypeId> {
    self.definitions[id].ty
  }

  pub fn display_type(&self, id: TypeId) -> String {
    self.type_arena.display(id)
  }

  pub fn references_to(&self, def: DefinitionId) -> Vec<ExprId> {
    self.references.iter().filter(|r| r.definition == def).map(|r| r.expr_id).collect()
  }
}

pub struct SemanticModelBuilder {
  pub(crate) scopes: Vec<Scope>,
  pub(crate) definitions: Vec<DefinitionInfo>,
  pub(crate) references: Vec<Reference>,
  scope_stack: Vec<ScopeId>,
  def_lookup: HashMap<(ScopeId, Sym), DefinitionId>,
}

impl Default for SemanticModelBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl SemanticModelBuilder {
  pub fn new() -> Self {
    let root = Scope { parent: None, span: (0, 0).into(), kind: ScopeKind::Module };
    Self { scopes: vec![root], definitions: Vec::new(), references: Vec::new(), scope_stack: vec![0], def_lookup: HashMap::new() }
  }

  pub fn push_scope(&mut self, kind: ScopeKind, span: SourceSpan) -> ScopeId {
    let id = self.scopes.len();
    let parent = Some(self.current_scope());
    self.scopes.push(Scope { parent, span, kind });
    self.scope_stack.push(id);
    id
  }

  pub fn pop_scope(&mut self) {
    self.scope_stack.pop();
  }

  pub fn current_scope(&self) -> ScopeId {
    *self.scope_stack.last().expect("scope stack empty")
  }

  pub fn add_definition(&mut self, name: Sym, kind: DefKind, span: SourceSpan, mutable: bool) -> DefinitionId {
    let scope = self.current_scope();
    let id = self.definitions.len();
    self.definitions.push(DefinitionInfo { name, kind, span, ty: None, scope, mutable });
    self.def_lookup.insert((scope, name), id);
    id
  }

  pub fn set_definition_type(&mut self, id: DefinitionId, ty: TypeId) {
    self.definitions[id].ty = Some(ty);
  }

  pub fn add_reference(&mut self, expr_id: ExprId, def_id: DefinitionId) {
    self.references.push(Reference { expr_id, definition: def_id });
  }

  pub fn resolve_in_scope(&self, name: Sym) -> Option<DefinitionId> {
    for &scope_id in self.scope_stack.iter().rev() {
      if let Some(&def_id) = self.def_lookup.get(&(scope_id, name)) {
        return Some(def_id);
      }
    }
    None
  }

  pub fn lookup_type(&self, name: Sym) -> Option<TypeId> {
    self.resolve_in_scope(name).and_then(|id| self.definitions[id].ty)
  }

  pub fn names_in_scope(&self) -> Vec<Sym> {
    let mut names = Vec::new();
    for &scope_id in &self.scope_stack {
      for &(sid, name) in self.def_lookup.keys() {
        if sid == scope_id {
          names.push(name);
        }
      }
    }
    names.sort();
    names.dedup();
    names
  }

  pub fn build(
    self,
    expr_types: ArenaMap<ExprId, TypeId>,
    type_defs: HashMap<Sym, Vec<Sym>>,
    trait_fields: HashMap<Sym, Vec<(Sym, TypeId)>>,
    type_arena: TypeArena,
  ) -> SemanticModel {
    SemanticModel { scopes: self.scopes, definitions: self.definitions, references: self.references, expr_types, type_defs, trait_fields, type_arena }
  }
}
