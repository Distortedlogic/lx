use std::sync::Arc;

use indexmap::IndexMap;

use crate::ast::{BindTarget, Stmt};
use crate::error::LxError;
use crate::value::{self, Value};

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
                methods,
                defaults,
                requires,
                description,
                tags,
                exported,
            } => {
                let mut method_defs = Vec::new();
                for m in methods {
                    let mut input = Vec::new();
                    for f in &m.input {
                        let default = match &f.default {
                            Some(e) => Some(self.eval(e)?),
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
                    let handler = self.eval(&d.handler)?;
                    default_impls.insert(d.name.clone(), handler);
                }
                let val = Value::Trait {
                    name: Arc::from(name.as_str()),
                    methods: Arc::new(method_defs),
                    defaults: Arc::new(default_impls),
                    requires: Arc::new(requires.iter().map(|s| Arc::from(s.as_str())).collect()),
                    description: description.as_ref().map(|s| Arc::from(s.as_str())),
                    tags: Arc::new(tags.iter().map(|s| Arc::from(s.as_str())).collect()),
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
                uses,
                init,
                on,
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
                let uses_resolved: Vec<(Arc<str>, Arc<str>)> = uses
                    .iter()
                    .map(|(binding, module)| {
                        (Arc::from(binding.as_str()), Arc::from(module.as_str()))
                    })
                    .collect();
                let on_val = match on {
                    Some(expr) => Some(Box::new(self.eval(expr)?)),
                    None => None,
                };
                for trait_name in traits {
                    if let Some(Value::Trait {
                        methods: trait_methods,
                        defaults: trait_defaults,
                        ..
                    }) = self.env.get(trait_name)
                    {
                        for (dname, dval) in trait_defaults.iter() {
                            if !method_map.contains_key(dname) {
                                method_map.insert(dname.clone(), dval.clone());
                            }
                        }
                        for required in trait_methods.iter() {
                            if !method_map.contains_key(&required.name) {
                                return Err(LxError::runtime(
                                    format!(
                                        "Agent {name} missing method '{}' required by Trait {trait_name}",
                                        required.name
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
                    uses: Arc::new(uses_resolved),
                    on: on_val,
                };
                let _ = exported;
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
                exported,
            } => {
                let mut defaults_map = IndexMap::new();
                for f in fields {
                    let val = self.eval(&f.default)?;
                    defaults_map.insert(f.name.clone(), val);
                }
                let mut method_map = IndexMap::new();
                for m in methods {
                    let handler = self.eval(&m.handler)?;
                    method_map.insert(m.name.clone(), handler);
                }
                for trait_name in traits {
                    if let Some(Value::Trait {
                        methods: trait_methods,
                        defaults: trait_defaults,
                        ..
                    }) = self.env.get(trait_name)
                    {
                        for (dname, dval) in trait_defaults.iter() {
                            if !method_map.contains_key(dname) {
                                method_map.insert(dname.clone(), dval.clone());
                            }
                        }
                        for required in trait_methods.iter() {
                            if !method_map.contains_key(&required.name) {
                                return Err(LxError::runtime(
                                    format!(
                                        "Class {name} missing method '{}' required by Trait {trait_name}",
                                        required.name
                                    ),
                                    stmt.span,
                                ));
                            }
                        }
                    }
                }
                let val = Value::Class {
                    name: Arc::from(name.as_str()),
                    traits: Arc::new(traits.iter().map(|s| Arc::from(s.as_str())).collect()),
                    defaults: Arc::new(defaults_map),
                    methods: Arc::new(method_map),
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
                if let Value::Object { id, .. } = &current {
                    value::object_store_update_nested(*id, fields, new_val)
                        .map_err(|e| LxError::runtime(e, stmt.span))?;
                    return Ok(Value::Unit);
                }
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
