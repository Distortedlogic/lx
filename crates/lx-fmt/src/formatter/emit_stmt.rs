use lx_ast::ast::{BindTarget, Binding, ClassDeclData, Stmt, StmtFieldUpdate, StmtId, StmtTypeDef, TraitDeclData, TraitEntry, TraitUnionDef, UseKind, UseStmt};

use super::Formatter;

impl Formatter<'_> {
  pub(super) fn emit_stmt(&mut self, id: StmtId) {
    let stmt = self.arena.stmt(id);
    match stmt {
      Stmt::Binding(b) => self.emit_binding(b),
      Stmt::TypeDef(td) => self.emit_type_def(td),
      Stmt::TraitUnion(tu) => self.emit_trait_union(tu),
      Stmt::TraitDecl(data) => self.emit_trait_decl(data),
      Stmt::ClassDecl(data) => self.emit_class_decl(data),
      Stmt::KeywordDecl(data) => self.emit_keyword_decl(data),
      Stmt::FieldUpdate(fu) => self.emit_field_update(fu),
      Stmt::Use(u) => self.emit_use(u),
      Stmt::ChannelDecl(name) => {
        self.write("channel ");
        self.write(name.as_str());
      },
      Stmt::Expr(eid) => self.emit_expr(*eid),
    }
  }

  fn emit_binding(&mut self, b: &Binding) {
    if b.exported {
      self.write("+");
    }
    match &b.target {
      BindTarget::Name(name) => {
        self.write(name.as_str());
        if let Some(ty) = b.type_ann {
          self.write(": ");
          self.emit_type_expr(ty);
        }
        if b.mutable {
          self.write(" := ");
        } else {
          self.write(" = ");
        }
      },
      BindTarget::Reassign(name) => {
        self.write(name.as_str());
        self.write(" <- ");
      },
      BindTarget::Pattern(pid) => {
        self.emit_pattern(*pid);
        if let Some(ty) = b.type_ann {
          self.write(": ");
          self.emit_type_expr(ty);
        }
        self.write(" = ");
      },
    }
    self.emit_expr(b.value);
  }

  fn emit_type_def(&mut self, td: &StmtTypeDef) {
    if td.exported {
      self.write("+");
    }
    self.write("type ");
    self.write(td.name.as_str());
    self.emit_type_params(&td.type_params);
    self.write(" =");
    for (name, arity) in &td.variants {
      self.newline();
      self.write("| ");
      self.write(name.as_str());
      for _ in 0..*arity {
        self.write(" _");
      }
    }
  }

  fn emit_trait_union(&mut self, tu: &TraitUnionDef) {
    if tu.exported {
      self.write("+");
    }
    self.write("Trait ");
    self.write(tu.name.as_str());
    self.emit_type_params(&tu.type_params);
    self.write(" = ");
    for (i, v) in tu.variants.iter().enumerate() {
      if i > 0 {
        self.write(" | ");
      }
      self.write(v.as_str());
    }
  }

  fn emit_trait_decl(&mut self, data: &TraitDeclData) {
    if data.exported {
      self.write("+");
    }
    self.write("Trait ");
    self.write(data.name.as_str());
    self.emit_type_params(&data.type_params);
    self.write(" = {");
    self.indent();
    for entry in &data.entries {
      self.newline();
      match entry {
        TraitEntry::Field(f) => {
          self.write(f.name.as_str());
          self.write(": ");
          self.write(f.type_name.as_str());
          if let Some(default) = f.default {
            self.write(" = ");
            self.emit_expr(default);
          }
        },
        TraitEntry::Spread(name) => {
          self.write("..");
          self.write(name.as_str());
        },
      }
    }
    for method in &data.methods {
      self.newline();
      self.write(method.name.as_str());
      self.write(": ");
      for (i, input) in method.input.iter().enumerate() {
        if i > 0 {
          self.write(" -> ");
        }
        self.write(input.type_name.as_str());
      }
      self.write(" -> ");
      self.write(method.output.as_str());
    }
    for default in &data.defaults {
      self.newline();
      self.write(default.name.as_str());
      self.write(" = ");
      self.emit_expr(default.handler);
    }
    self.dedent();
    self.newline();
    self.write("}");
  }

  fn emit_class_decl(&mut self, data: &ClassDeclData) {
    if data.exported {
      self.write("+");
    }
    self.write("Class ");
    self.write(data.name.as_str());
    self.emit_type_params(&data.type_params);
    if !data.traits.is_empty() {
      self.write(" : ");
      if data.traits.len() > 1 {
        self.write("[");
      }
      for (i, t) in data.traits.iter().enumerate() {
        if i > 0 {
          self.write("; ");
        }
        self.write(t.as_str());
      }
      if data.traits.len() > 1 {
        self.write("]");
      }
    }
    self.write(" = {");
    self.indent();
    for f in &data.fields {
      self.newline();
      self.write(f.name.as_str());
      self.write(": ");
      self.emit_expr(f.default);
    }
    for m in &data.methods {
      self.newline();
      self.write(m.name.as_str());
      self.write(" = ");
      self.emit_expr(m.handler);
    }
    self.dedent();
    self.newline();
    self.write("}");
  }

  fn emit_field_update(&mut self, fu: &StmtFieldUpdate) {
    self.write(fu.name.as_str());
    for f in &fu.fields {
      self.write(".");
      self.write(f.as_str());
    }
    self.write(" <- ");
    self.emit_expr(fu.value);
  }

  fn emit_use(&mut self, u: &UseStmt) {
    self.write("use ");
    for (i, seg) in u.path.iter().enumerate() {
      if i > 0 {
        self.write("/");
      }
      self.write(seg.as_str());
    }
    match &u.kind {
      UseKind::Whole => {},
      UseKind::Alias(alias) => {
        self.write(" : ");
        self.write(alias.as_str());
      },
      UseKind::Selective(names) => {
        self.write(" { ");
        for (i, n) in names.iter().enumerate() {
          if i > 0 {
            self.write("; ");
          }
          self.write(n.as_str());
        }
        self.write(" }");
      },
    }
  }

  pub(super) fn emit_type_params(&mut self, params: &[lx_span::sym::Sym]) {
    if params.is_empty() {
      return;
    }
    self.write("[");
    for (i, p) in params.iter().enumerate() {
      if i > 0 {
        self.write("; ");
      }
      self.write(p.as_str());
    }
    self.write("]");
  }
}
