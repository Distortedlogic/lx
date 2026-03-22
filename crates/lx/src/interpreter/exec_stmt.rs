use std::sync::Arc;

use async_recursion::async_recursion;
use indexmap::IndexMap;

use crate::ast::{BindTarget, SStmt, Stmt, StmtFieldUpdate, StmtTypeDef};
use crate::env::Env;
use crate::error::LxError;
use crate::sym::Sym;
use crate::value::{LxClass, LxTrait, LxVal};

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
  pub(crate) async fn eval_stmt(&mut self, stmt: &SStmt) -> Result<LxVal, LxError> {
    match &stmt.node {
      Stmt::Binding(b) => {
        let val = self.eval(&b.value).await?;
        let val = self.force_defaults(val, stmt.span).await?;
        match &b.target {
          BindTarget::Name(name) => {
            if self.env.has_mut(*name) {
              let val = Self::maybe_combine_clauses(&self.env, *name, val);
              self.env.reassign(*name, val).map_err(|e| LxError::runtime(e, stmt.span))?;
            } else {
              let val = Self::maybe_combine_clauses(&self.env, *name, val);
              let env = self.env.child();
              env.bind_with_mutability(*name, val, b.mutable);
              self.env = Arc::new(env);
            }
          },
          BindTarget::Reassign(name) => {
            self.env.reassign(*name, val).map_err(|e| LxError::runtime(e, stmt.span))?;
          },
          BindTarget::Pattern(pat) => {
            let bindings = self.try_match_pattern(&pat.node, &val).ok_or_else(|| {
              let pat_str = pat.node.to_string();
              let hint = binding_pattern_hint(&pat_str);
              let msg = match hint {
                Some(h) => format!("cannot bind {} `{}` to pattern `{pat_str}` — {h}", val.type_name(), val.short_display(),),
                None => format!("cannot bind {} `{}` to pattern `{pat_str}`", val.type_name(), val.short_display(),),
              };
              LxError::runtime(msg, stmt.span)
            })?;
            let env = self.env.child();
            for (sym, v) in bindings {
              env.bind_with_mutability(sym, v, b.mutable);
            }
            self.env = Arc::new(env);
          },
        }
        Ok(LxVal::Unit)
      },
      Stmt::Use(use_stmt) => {
        self.eval_use(use_stmt, stmt.span).await?;
        Ok(LxVal::Unit)
      },
      Stmt::TypeDef(StmtTypeDef { variants, .. }) => {
        let env = self.env.child();
        for (ctor_name, arity) in variants {
          if *arity == 0 {
            env.bind(*ctor_name, LxVal::Tagged { tag: *ctor_name, values: Arc::new(vec![]) });
          } else {
            env.bind(*ctor_name, LxVal::TaggedCtor { tag: *ctor_name, arity: *arity, applied: vec![] });
          }
        }
        self.env = Arc::new(env);
        Ok(LxVal::Unit)
      },
      Stmt::TraitUnion(def) => self.eval_trait_union(def.name, &def.variants, stmt.span),
      Stmt::TraitDecl(data) => {
        let trait_fields = self.eval_trait_fields(data.name.as_str(), &data.entries, stmt.span).await?;
        let mut method_defs = Vec::new();
        for m in &data.methods {
          let mut input = Vec::new();
          for f in &m.input {
            let default = match &f.default {
              Some(e) => Some(self.eval(e).await?),
              None => None,
            };
            input.push(crate::value::FieldDef { name: f.name, type_name: f.type_name, default, constraint: None });
          }
          method_defs.push(crate::value::TraitMethodDef { name: m.name, input, output: m.output });
        }
        let mut default_impls = IndexMap::new();
        for d in &data.defaults {
          let handler = self.eval(&d.handler).await?;
          default_impls.insert(d.name, handler);
        }
        let val = LxVal::Trait(Box::new(LxTrait {
          name: data.name,
          fields: Arc::new(trait_fields),
          methods: Arc::new(method_defs),
          defaults: Arc::new(default_impls),
          requires: Arc::new(data.requires.clone()),
          description: data.description,
          tags: Arc::new(data.tags.clone()),
        }));
        let env = self.env.child();
        env.bind(data.name, val);
        self.env = Arc::new(env);
        Ok(LxVal::Unit)
      },
      Stmt::ClassDecl(data) => {
        let mut defaults_map = IndexMap::new();
        for f in &data.fields {
          let val = self.eval(&f.default).await?;
          defaults_map.insert(f.name, val);
        }
        let mut method_map = IndexMap::new();
        for m in &data.methods {
          let handler = self.eval(&m.handler).await?;
          method_map.insert(m.name, handler);
        }
        Self::inject_traits(&mut method_map, &data.traits, &self.env, "Class", data.name.as_str(), stmt.span)?;
        let val = LxVal::Class(Box::new(LxClass {
          name: data.name,
          traits: Arc::new(data.traits.clone()),
          defaults: Arc::new(defaults_map),
          methods: Arc::new(method_map),
        }));
        let env = self.env.child();
        env.bind(data.name, val);
        self.env = Arc::new(env);
        Ok(LxVal::Unit)
      },
      Stmt::FieldUpdate(StmtFieldUpdate { name, fields, value }) => {
        let new_val = self.eval(value).await?;
        let current = self.env.get(*name).ok_or_else(|| LxError::runtime(format!("undefined variable '{name}'"), stmt.span))?;
        if let LxVal::Object(o) = &current {
          crate::stdlib::object_update_nested(o.id, fields, new_val).map_err(|e| LxError::runtime(e, stmt.span))?;
          return Ok(LxVal::Unit);
        }
        let updated = Self::update_record_field(&current, fields, new_val, stmt.span)?;
        self.env.reassign(*name, updated).map_err(|e| LxError::runtime(e, stmt.span))?;
        Ok(LxVal::Unit)
      },
      Stmt::Expr(e) => self.eval(e).await,
    }
  }

  fn maybe_combine_clauses(env: &Arc<Env>, name: Sym, val: LxVal) -> LxVal {
    let LxVal::Func(new_func) = &val else { return val };
    let Some(existing) = env.get(name) else { return val };
    match existing {
      LxVal::Func(ref ef) if ef.guard.is_some() => {
        let clauses = vec![ef.as_ref().clone(), new_func.as_ref().clone()];
        LxVal::MultiFunc(clauses)
      },
      LxVal::MultiFunc(mut clauses) => {
        clauses.push(new_func.as_ref().clone());
        LxVal::MultiFunc(clauses)
      },
      _ => val,
    }
  }
}
