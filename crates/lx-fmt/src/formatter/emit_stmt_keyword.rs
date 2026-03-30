use lx_ast::ast::{KeywordDeclData, KeywordKind, TraitEntry};

use super::Formatter;

impl Formatter<'_> {
  pub(super) fn emit_keyword_decl(&mut self, data: &KeywordDeclData) {
    if data.exported {
      self.write("+");
    }
    let kw = match data.keyword {
      KeywordKind::Agent => "Agent",
      KeywordKind::Tool => "Tool",
      KeywordKind::Prompt => "Prompt",
      KeywordKind::Store => "Store",
      KeywordKind::Session => "Session",
      KeywordKind::Guard => "Guard",
      KeywordKind::Workflow => "Workflow",
      KeywordKind::Schema => "Schema",
      KeywordKind::Mcp => "MCP",
      KeywordKind::Cli => "CLI",
      KeywordKind::Http => "HTTP",
    };
    self.write(kw);
    self.write(" ");
    self.write(data.name.as_str());
    self.emit_type_params(&data.type_params);
    self.write(" = {");
    self.indent();
    if let Some(ref entries) = data.trait_entries {
      for entry in entries {
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
    } else {
      for f in &data.fields {
        self.newline();
        self.write(f.name.as_str());
        self.write(": ");
        self.emit_expr(f.default);
      }
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
}
