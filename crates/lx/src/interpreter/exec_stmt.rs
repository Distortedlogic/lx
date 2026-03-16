use std::sync::Arc;

use indexmap::IndexMap;

use crate::ast::{BindTarget, Stmt};
use crate::error::LxError;
use crate::value::Value;

use super::Interpreter;

fn binding_pattern_hint(pat_str: &str) -> Option<&'static str> {
    let first_word = pat_str.split_whitespace().next().unwrap_or("");
    let trimmed = first_word.trim_matches(|c| c == '(' || c == ')' || c == ',');
    match trimmed {
        "mut" => Some("lx uses `:=` for mutable bindings: `x := 0`"),
        "let" | "var" | "const" => {
            Some("lx bindings use `name = value` (or `name := value` for mutable)")
        }
        _ => None,
    }
}

impl Interpreter {
    pub(crate) fn eval_stmt(&mut self, stmt: &crate::ast::SStmt) -> Result<Value, LxError> {
        match &stmt.node {
            Stmt::Binding(b) => {
                let val = self.eval(&b.value)?;
                let val = self.force_defaults(val, stmt.span)?;
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
                self.eval_use(use_stmt, stmt.span)?;
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
                self.eval_protocol_def(name, entries, stmt.span)
            }
            Stmt::ProtocolUnion(def) => {
                self.eval_protocol_union(&def.name, &def.variants, stmt.span)
            }
            Stmt::McpDecl { name, tools, .. } => self.eval_mcp_decl(name, tools, stmt.span),
            Stmt::TraitDecl {
                name,
                handles,
                provides,
                requires,
                exported,
            } => {
                let val = Value::Trait {
                    name: Arc::from(name.as_str()),
                    handles: Arc::new(handles.iter().map(|s| Arc::from(s.as_str())).collect()),
                    provides: Arc::new(provides.iter().map(|s| Arc::from(s.as_str())).collect()),
                    requires: Arc::new(requires.iter().map(|s| Arc::from(s.as_str())).collect()),
                };
                let _ = exported;
                let mut env = self.env.child();
                env.bind(name.clone(), val);
                self.env = env.into_arc();
                Ok(Value::Unit)
            }
            Stmt::AgentDecl {
                name,
                traits,
                uses: _,
                init,
                on: _,
                methods,
                exported,
            } => {
                let mut method_map = IndexMap::new();
                for m in methods {
                    let handler = self.eval(&m.handler)?;
                    method_map.insert(m.name.clone(), handler);
                }
                let init_val = match init {
                    Some(expr) => Some(Box::new(self.eval(expr)?)),
                    None => None,
                };
                for trait_name in traits {
                    if let Some(Value::Trait { handles, .. }) = self.env.get(trait_name) {
                        for required in handles.iter() {
                            let key = required.to_string();
                            if !method_map.contains_key(&key) {
                                return Err(LxError::runtime(
                                    format!(
                                        "Agent {name} missing method '{key}' required by {trait_name}"
                                    ),
                                    stmt.span,
                                ));
                            }
                        }
                    }
                }
                let val = Value::Agent {
                    name: Arc::from(name.as_str()),
                    traits: Arc::new(traits.iter().map(|s| Arc::from(s.as_str())).collect()),
                    methods: Arc::new(method_map),
                    init: init_val,
                };
                let _ = exported;
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
                let new_val = self.eval(value)?;
                let current = self.env.get(name).ok_or_else(|| {
                    LxError::runtime(format!("undefined variable '{name}'"), stmt.span)
                })?;
                let updated = Self::update_record_field(&current, fields, new_val, stmt.span)?;
                self.env
                    .reassign(name, updated)
                    .map_err(|e| LxError::runtime(e, stmt.span))?;
                Ok(Value::Unit)
            }
            Stmt::Expr(e) => self.eval(e),
        }
    }
}
