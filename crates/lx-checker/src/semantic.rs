use std::collections::HashMap;

use la_arena::ArenaMap;

use lx_ast::ast::ExprId;
use lx_span::sym::Sym;
use miette::SourceSpan;

use super::type_arena::{TypeArena, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(u32);

impl ScopeId {
  pub fn new(id: usize) -> Self {
    Self(id as u32)
  }
  pub fn index(self) -> usize {
    self.0 as usize
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefinitionId(u32);

impl DefinitionId {
  pub fn new(id: usize) -> Self {
    Self(id as u32)
  }
  pub fn index(self) -> usize {
    self.0 as usize
  }
}

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
  pub def_references: HashMap<DefinitionId, Vec<ExprId>>,
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
    self.definitions[id.index()].ty
  }

  pub fn display_type(&self, id: TypeId) -> String {
    self.type_arena.display(id)
  }

  pub fn references_to(&self, def: DefinitionId) -> &[ExprId] {
    self.def_references.get(&def).map(|v| v.as_slice()).unwrap_or(&[])
  }
}

pub struct SemanticModelBuilder {
  pub(crate) scopes: Vec<Scope>,
  pub(crate) definitions: Vec<DefinitionInfo>,
  pub(crate) references: Vec<Reference>,
  def_references: HashMap<DefinitionId, Vec<ExprId>>,
  scope_stack: Vec<ScopeId>,
  def_lookup: HashMap<(ScopeId, Sym), DefinitionId>,
  scope_definitions: HashMap<ScopeId, Vec<DefinitionId>>,
}

impl Default for SemanticModelBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl SemanticModelBuilder {
  pub fn new() -> Self {
    let root = Scope { parent: None, span: (0, 0).into(), kind: ScopeKind::Module };
    Self {
      scopes: vec![root],
      definitions: Vec::new(),
      references: Vec::new(),
      def_references: HashMap::new(),
      scope_stack: vec![ScopeId::new(0)],
      def_lookup: HashMap::new(),
      scope_definitions: HashMap::new(),
    }
  }

  pub fn push_scope(&mut self, kind: ScopeKind, span: SourceSpan) -> ScopeId {
    let id = ScopeId::new(self.scopes.len());
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
    let id = DefinitionId::new(self.definitions.len());
    self.definitions.push(DefinitionInfo { name, kind, span, ty: None, scope, mutable });
    self.def_lookup.insert((scope, name), id);
    self.scope_definitions.entry(scope).or_default().push(id);
    id
  }

  pub fn set_definition_type(&mut self, id: DefinitionId, ty: TypeId) {
    self.definitions[id.index()].ty = Some(ty);
  }

  pub fn add_reference(&mut self, expr_id: ExprId, def_id: DefinitionId) {
    self.references.push(Reference { expr_id, definition: def_id });
    self.def_references.entry(def_id).or_default().push(expr_id);
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
    self.resolve_in_scope(name).and_then(|id| self.definitions[id.index()].ty)
  }

  pub fn names_in_scope(&self) -> Vec<Sym> {
    let mut names: Vec<_> = self
      .scope_stack
      .iter()
      .filter_map(|scope_id| self.scope_definitions.get(scope_id))
      .flat_map(|defs| defs.iter().map(|&d| self.definitions[d.index()].name))
      .collect();
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
    SemanticModel {
      scopes: self.scopes,
      definitions: self.definitions,
      references: self.references,
      def_references: self.def_references,
      expr_types,
      type_defs,
      trait_fields,
      type_arena,
    }
  }
}
