use std::sync::Arc;

use async_recursion::async_recursion;
use indexmap::IndexMap;

use crate::ast::{BindTarget, Stmt};
use crate::error::LxError;
use crate::value::LxVal;

use super::Interpreter;

fn binding_pattern_hint(pat_str: &str) -> Option<&'static str> {
  match pat_str.split_whitespace().next().unwrap_or("").trim_matches(|c| c == '(' || c == ')' || c == ',') {
    "mut" => Some("lx uses `:=` for mutable bindings: `x := 0`"),
    "let" | "var" | "const" => Some("lx bindings use `name = value` (or `name := value` for mutable)"),
    _ => None,
  }
}

impl Interpreter {
  #[async_recursion(?Send)]
  pub(crate) async fn eval_stmt(&mut self, stmt: &crate::ast::SStmt) -> Result<LxVal, LxError> {
    match &stmt.node {
      Stmt::Binding(b) => {
        let val = self.eval(&b.value).await?;
        let val = self.force_defaults(val, stmt.span).await?;
        match &b.target {
          BindTarget::Name(name) => {
            if self.env.has_mut(name) {
              self.env.reassign(name, val).map_err(|e| LxError::runtime(e, stmt.span))?;
            } else {
              let mut env = self.env.child();
              if b.mutable {
                env.bind_mut(name.clone(), val);
              } else {
                env.bind(name.clone(), val);
              }
              self.env = env.into_arc();
            }
          },
          BindTarget::Reassign(name) => {
            self.env.reassign(name, val).map_err(|e| LxError::runtime(e, stmt.span))?;
          },
          BindTarget::Pattern(pat) => {
            let bindings = self.try_match_pattern(&pat.node, &val).ok_or_else(|| {
              let pat_str = format!("{}", pat.node);
              let hint = binding_pattern_hint(&pat_str);
              let msg = match hint {
                Some(h) => format!("cannot bind {} `{}` to pattern `{pat_str}` — {h}", val.type_name(), val.short_display(),),
                None => format!("cannot bind {} `{}` to pattern `{pat_str}`", val.type_name(), val.short_display(),),
              };
              LxError::runtime(msg, stmt.span)
            })?;
            let mut env = self.env.child();
            for (name, v) in bindings {
              if b.mutable {
                env.bind_mut(name, v);
              } else {
                env.bind(name, v);
              }
            }
            self.env = env.into_arc();
          },
        }
        Ok(LxVal::Unit)
      },
      Stmt::Use(use_stmt) => {
        self.eval_use(use_stmt, stmt.span).await?;
        Ok(LxVal::Unit)
      },
      Stmt::TypeDef { variants, .. } => {
        let mut env = self.env.child();
        for (ctor_name, arity) in variants {
          let tag: Arc<str> = Arc::from(ctor_name.as_str());
          if *arity == 0 {
            env.bind(ctor_name.clone(), LxVal::Tagged { tag, values: Arc::new(vec![]) });
          } else {
            env.bind(ctor_name.clone(), LxVal::TaggedCtor { tag, arity: *arity, applied: vec![] });
          }
        }
        self.env = env.into_arc();
        Ok(LxVal::Unit)
      },
      Stmt::TraitUnion(def) => self.eval_trait_union(&def.name, &def.variants, stmt.span),
      Stmt::TraitDecl(data) => {
        let trait_fields = self.eval_trait_fields(&data.name, &data.entries, stmt.span).await?;
        let mut method_defs = Vec::new();
        for m in &data.methods {
          let mut input = Vec::new();
          for f in &m.input {
            let default = match &f.default {
              Some(e) => Some(self.eval(e).await?),
              None => None,
            };
            input.push(crate::value::FieldDef { name: f.name.clone(), type_name: f.type_name.clone(), default, constraint: None });
          }
          method_defs.push(crate::value::TraitMethodDef { name: m.name.clone(), input, output: m.output.clone() });
        }
        let mut default_impls = IndexMap::new();
        for d in &data.defaults {
          let handler = self.eval(&d.handler).await?;
          default_impls.insert(d.name.clone(), handler);
        }
        let val = LxVal::Trait {
          name: Arc::from(data.name.as_str()),
          fields: Arc::new(trait_fields),
          methods: Arc::new(method_defs),
          defaults: Arc::new(default_impls),
          requires: Arc::new(data.requires.iter().map(|s| Arc::from(s.as_str())).collect()),
          description: data.description.as_ref().map(|s| Arc::from(s.as_str())),
          tags: Arc::new(data.tags.iter().map(|s| Arc::from(s.as_str())).collect()),
        };
        let mut env = self.env.child();
        env.bind(data.name.clone(), val);
        self.env = env.into_arc();
        Ok(LxVal::Unit)
      },
      Stmt::ClassDecl(data) => {
        let mut defaults_map = IndexMap::new();
        for f in &data.fields {
          let val = self.eval(&f.default).await?;
          defaults_map.insert(f.name.clone(), val);
        }
        let mut method_map = IndexMap::new();
        for m in &data.methods {
          let handler = self.eval(&m.handler).await?;
          method_map.insert(m.name.clone(), handler);
        }
        Self::inject_traits(&mut method_map, &data.traits, &self.env, "Class", &data.name, stmt.span)?;
        let val = LxVal::Class {
          name: Arc::from(data.name.as_str()),
          traits: Arc::new(data.traits.iter().map(|s| Arc::from(s.as_str())).collect()),
          defaults: Arc::new(defaults_map),
          methods: Arc::new(method_map),
        };
        let mut env = self.env.child();
        env.bind(data.name.clone(), val);
        self.env = env.into_arc();
        Ok(LxVal::Unit)
      },
      Stmt::FieldUpdate { name, fields, value } => {
        let new_val = self.eval(value).await?;
        let current = self.env.get(name).ok_or_else(|| LxError::runtime(format!("undefined variable '{name}'"), stmt.span))?;
        if let LxVal::Object { id, .. } = &current {
          crate::stdlib::object_update_nested(*id, fields, new_val).map_err(|e| LxError::runtime(e, stmt.span))?;
          return Ok(LxVal::Unit);
        }
        let updated = Self::update_record_field(&current, fields, new_val, stmt.span)?;
        self.env.reassign(name, updated).map_err(|e| LxError::runtime(e, stmt.span))?;
        Ok(LxVal::Unit)
      },
      Stmt::Expr(e) => self.eval(e).await,
    }
  }
}
