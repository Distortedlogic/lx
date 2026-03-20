use std::sync::Arc;

use async_recursion::async_recursion;
use indexmap::IndexMap;

use crate::ast::{BindTarget, Stmt};
use crate::error::LxError;
use crate::value::Value;

use super::Interpreter;

fn binding_pattern_hint(pat_str: &str) -> Option<&'static str> {
    match pat_str
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_matches(|c| c == '(' || c == ')' || c == ',')
    {
        "mut" => Some("lx uses `:=` for mutable bindings: `x := 0`"),
        "let" | "var" | "const" => {
            Some("lx bindings use `name = value` (or `name := value` for mutable)")
        }
        _ => None,
    }
}

impl Interpreter {
    #[async_recursion(?Send)]
    pub(crate) async fn eval_stmt(&mut self, stmt: &crate::ast::SStmt) -> Result<Value, LxError> {
        match &stmt.node {
            Stmt::Binding(b) => {
                let val = self.eval(&b.value).await?;
                let val = self.force_defaults(val, stmt.span).await?;
                match &b.target {
                    BindTarget::Name(name) => {
                        if self.env.has_mut(name) {
                            self.env
                                .reassign(name, val)
                                .map_err(|e| LxError::runtime(e, stmt.span))?;
                        } else {
                            let mut env = self.env.child();
                            if b.mutable {
                                env.bind_mut(name.clone(), val);
                            } else {
                                env.bind(name.clone(), val);
                            }
                            self.env = env.into_arc();
                        }
                    }
                    BindTarget::Reassign(name) => {
                        self.env
                            .reassign(name, val)
                            .map_err(|e| LxError::runtime(e, stmt.span))?;
                    }
                    BindTarget::Pattern(pat) => {
                        let bindings =
                            self.try_match_pattern(&pat.node, &val).ok_or_else(|| {
                                let pat_str = format!("{}", pat.node);
                                let hint = binding_pattern_hint(&pat_str);
                                let msg = match hint {
                                    Some(h) => format!(
                                        "cannot bind {} `{}` to pattern `{pat_str}` — {h}",
                                        val.type_name(),
                                        val.short_display(),
                                    ),
                                    None => format!(
                                        "cannot bind {} `{}` to pattern `{pat_str}`",
                                        val.type_name(),
                                        val.short_display(),
                                    ),
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
                    }
                }
                Ok(Value::Unit)
            }
            Stmt::Use(use_stmt) => {
                self.eval_use(use_stmt, stmt.span).await?;
                Ok(Value::Unit)
            }
            Stmt::TypeDef { variants, .. } => {
                let mut env = self.env.child();
                for (ctor_name, arity) in variants {
                    let tag: Arc<str> = Arc::from(ctor_name.as_str());
                    if *arity == 0 {
                        env.bind(
                            ctor_name.clone(),
                            Value::Tagged {
                                tag,
                                values: Arc::new(vec![]),
                            },
                        );
                    } else {
                        env.bind(
                            ctor_name.clone(),
                            Value::TaggedCtor {
                                tag,
                                arity: *arity,
                                applied: vec![],
                            },
                        );
                    }
                }
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::Protocol { name, entries, .. } => {
                self.eval_protocol_def(name, entries, stmt.span).await
            }
            Stmt::ProtocolUnion(def) => {
                self.eval_protocol_union(&def.name, &def.variants, stmt.span)
            }
            Stmt::McpDecl { name, tools, .. } => self.eval_mcp_decl(name, tools, stmt.span).await,
            Stmt::TraitDecl {
                name,
                methods,
                defaults,
                requires,
                description,
                tags,
                ..
            } => {
                let mut method_defs = Vec::new();
                for m in methods {
                    let mut input = Vec::new();
                    for f in &m.input {
                        let default = match &f.default {
                            Some(e) => Some(self.eval(e).await?),
                            None => None,
                        };
                        input.push(crate::value::ProtoFieldDef {
                            name: f.name.clone(),
                            type_name: f.type_name.clone(),
                            default,
                            constraint: None,
                        });
                    }
                    let output = self.resolve_mcp_output(&m.output);
                    method_defs.push(crate::value::TraitMethodDef {
                        name: m.name.clone(),
                        input,
                        output,
                    });
                }
                let mut default_impls = IndexMap::new();
                for d in defaults {
                    let handler = self.eval(&d.handler).await?;
                    default_impls.insert(d.name.clone(), handler);
                }
                let val = Value::Trait {
                    name: Arc::from(name.as_str()),
                    fields: Arc::new(Vec::new()),
                    methods: Arc::new(method_defs),
                    defaults: Arc::new(default_impls),
                    requires: Arc::new(requires.iter().map(|s| Arc::from(s.as_str())).collect()),
                    description: description.as_ref().map(|s| Arc::from(s.as_str())),
                    tags: Arc::new(tags.iter().map(|s| Arc::from(s.as_str())).collect()),
                };
                let mut env = self.env.child();
                env.bind(name.clone(), val);
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::AgentDecl {
                name,
                traits,
                init,
                on,
                methods,
                ..
            } => {
                let mut method_map = IndexMap::new();
                for m in methods {
                    let handler = self.eval(&m.handler).await?;
                    method_map.insert(m.name.clone(), handler);
                }
                if let Some(expr) = init {
                    method_map.insert("init".into(), self.eval(expr).await?);
                }
                if let Some(expr) = on {
                    method_map.insert("on".into(), self.eval(expr).await?);
                }
                if self.env.get("Agent").is_none() {
                    let use_stmt = crate::ast::UseStmt {
                        path: vec!["pkg".into(), "agent".into()],
                        kind: crate::ast::UseKind::Selective(vec!["Agent".into()]),
                    };
                    self.eval_use(&use_stmt, stmt.span).await?;
                }
                let mut all_traits: Vec<String> = vec!["Agent".into()];
                for t in traits {
                    if t != "Agent" {
                        all_traits.push(t.clone());
                    }
                }
                Self::inject_traits(
                    &mut method_map,
                    &all_traits,
                    &self.env,
                    "Agent",
                    name,
                    stmt.span,
                )?;
                let val = Value::Class {
                    name: Arc::from(name.as_str()),
                    traits: Arc::new(all_traits.iter().map(|s| Arc::from(s.as_str())).collect()),
                    defaults: Arc::new(IndexMap::new()),
                    methods: Arc::new(method_map),
                };
                let mut env = self.env.child();
                env.bind(name.clone(), val);
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::ClassDecl {
                name,
                traits,
                fields,
                methods,
                ..
            } => {
                let mut defaults_map = IndexMap::new();
                for f in fields {
                    let val = self.eval(&f.default).await?;
                    defaults_map.insert(f.name.clone(), val);
                }
                let mut method_map = IndexMap::new();
                for m in methods {
                    let handler = self.eval(&m.handler).await?;
                    method_map.insert(m.name.clone(), handler);
                }
                Self::inject_traits(&mut method_map, traits, &self.env, "Class", name, stmt.span)?;
                let val = Value::Class {
                    name: Arc::from(name.as_str()),
                    traits: Arc::new(traits.iter().map(|s| Arc::from(s.as_str())).collect()),
                    defaults: Arc::new(defaults_map),
                    methods: Arc::new(method_map),
                };
                let mut env = self.env.child();
                env.bind(name.clone(), val);
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::FieldUpdate {
                name,
                fields,
                value,
            } => {
                let new_val = self.eval(value).await?;
                let current = self.env.get(name).ok_or_else(|| {
                    LxError::runtime(format!("undefined variable '{name}'"), stmt.span)
                })?;
                if let Value::Object { id, .. } = &current {
                    crate::stdlib::object_update_nested(*id, fields, new_val)
                        .map_err(|e| LxError::runtime(e, stmt.span))?;
                    return Ok(Value::Unit);
                }
                let updated = Self::update_record_field(&current, fields, new_val, stmt.span)?;
                self.env
                    .reassign(name, updated)
                    .map_err(|e| LxError::runtime(e, stmt.span))?;
                Ok(Value::Unit)
            }
            Stmt::Expr(e) => self.eval(e).await,
        }
    }
}
