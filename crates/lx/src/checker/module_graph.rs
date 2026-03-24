use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::ast::{BindTarget, Core, Program, Stmt};
use crate::source::FileId;
use crate::sym::Sym;

use super::semantic::SemanticModel;
use super::type_arena::{TypeArena, TypeId};

pub struct ModuleSignature {
  pub file: Option<crate::source::FileId>,
  pub bindings: HashMap<Sym, TypeId>,
  pub types: HashMap<Sym, Vec<Sym>>,
  pub traits: HashMap<Sym, Vec<(Sym, TypeId)>>,
  pub type_arena: TypeArena,
}

pub struct SourceFile {
  pub id: FileId,
  pub path: PathBuf,
  pub source: Arc<str>,
}

pub struct ModuleGraph {
  pub files: Vec<SourceFile>,
  pub file_by_path: HashMap<PathBuf, FileId>,
  pub signatures: HashMap<FileId, ModuleSignature>,
}

impl Default for ModuleGraph {
  fn default() -> Self {
    Self::new()
  }
}

impl ModuleGraph {
  pub fn new() -> Self {
    Self { files: Vec::new(), file_by_path: HashMap::new(), signatures: HashMap::new() }
  }

  pub fn add_file(&mut self, path: PathBuf, source: Arc<str>) -> FileId {
    let id = FileId::new(self.files.len() as u32);
    self.file_by_path.insert(path.clone(), id);
    self.files.push(SourceFile { id, path, source });
    id
  }

  pub fn get_file(&self, id: FileId) -> &SourceFile {
    &self.files[id.index() as usize]
  }

  pub fn resolve_use_path(&self, path: &[Sym], _from: FileId) -> Option<FileId> {
    let mut file_path = PathBuf::new();
    for seg in path {
      file_path.push(seg.as_str());
    }
    file_path.set_extension("lx");
    self.file_by_path.get(&file_path).copied()
  }
}

pub fn extract_signature(program: &Program<Core>, semantic: &SemanticModel) -> ModuleSignature {
  let mut bindings = HashMap::new();
  let mut types = HashMap::new();
  let mut traits = HashMap::new();

  for &sid in &program.stmts {
    let stmt = program.arena.stmt(sid);
    match stmt {
      Stmt::Binding(b) if b.exported => {
        if let BindTarget::Name(name) = &b.target {
          let ty = semantic.expr_types.get(b.value).copied().unwrap_or(semantic.type_arena.unknown());
          bindings.insert(*name, ty);
        }
      },
      Stmt::TypeDef(td) if td.exported => {
        let variant_names: Vec<Sym> = td.variants.iter().map(|(n, _)| *n).collect();
        types.insert(td.name, variant_names);
      },
      Stmt::TraitDecl(data) if data.exported => {
        let fields: Vec<(Sym, TypeId)> = semantic.trait_fields.get(&data.name).cloned().unwrap_or_default();
        traits.insert(data.name, fields);
      },
      Stmt::ClassDecl(data) if data.exported => {
        let unknown = semantic.type_arena.unknown();
        bindings.insert(data.name, unknown);
      },
      Stmt::Binding(_)
      | Stmt::TypeDef(_)
      | Stmt::TraitDecl(_)
      | Stmt::ClassDecl(_)
      | Stmt::TraitUnion(_)
      | Stmt::FieldUpdate(_)
      | Stmt::Use(_)
      | Stmt::Expr(_) => {},
    }
  }

  let type_arena = semantic.type_arena.clone();
  ModuleSignature { file: Some(program.file), bindings, types, traits, type_arena }
}
