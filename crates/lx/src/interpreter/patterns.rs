use std::sync::Arc;

use crate::ast::{Literal, MatchArm, Pattern, SExpr};
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

impl super::Interpreter {
    pub(super) fn eval_match(
        &mut self,
        scrutinee: &SExpr,
        arms: &[MatchArm],
        span: Span,
    ) -> Result<Value, LxError> {
        let val = self.eval(scrutinee)?;
        for arm in arms {
            if let Some(bindings) = self.try_match_pattern(&arm.pattern.node, &val) {
                let saved = Arc::clone(&self.env);
                let mut scope = self.env.child();
                for (name, v) in bindings {
                    scope.bind(name, v);
                }
                self.env = scope.into_arc();
                if let Some(guard) = &arm.guard {
                    let gv = self.eval(guard)?;
                    match gv.as_bool() {
                        Some(false) => {
                            self.env = saved;
                            continue;
                        }
                        Some(true) => {}
                        _ => {
                            self.env = saved;
                            return Err(LxError::type_err(
                                format!("match guard must be Bool, got {} `{gv}`", gv.type_name()),
                                span,
                            ));
                        }
                    }
                }
                let result = self.eval(&arm.body);
                self.env = saved;
                return result;
            }
        }
        Err(LxError::runtime(
            format!("no matching pattern for {} `{val}`", val.type_name()),
            span,
        ))
    }

    pub(super) fn try_match_pattern(
        &self,
        pattern: &Pattern,
        value: &Value,
    ) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(vec![]),
            Pattern::Bind(name) => Some(vec![(name.clone(), value.clone())]),
            Pattern::Literal(lit) => {
                let matches = match (lit, value) {
                    (Literal::Int(a), Value::Int(b)) => a == b,
                    (Literal::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
                    (Literal::Bool(a), Value::Bool(b)) => a == b,
                    (Literal::Unit, Value::Unit) => true,
                    (Literal::RawStr(a), Value::Str(b)) => a.as_str() == b.as_ref(),
                    (Literal::Str(parts), Value::Str(b)) => {
                        let mut s = String::new();
                        for part in parts {
                            match part {
                                crate::ast::StrPart::Text(t) => s.push_str(t),
                                crate::ast::StrPart::Interp(_) => return None,
                            }
                        }
                        s.as_str() == b.as_ref()
                    }
                    _ => false,
                };
                if matches { Some(vec![]) } else { None }
            }
            Pattern::Tuple(pats) => {
                let Value::Tuple(items) = value else {
                    return None;
                };
                if pats.len() != items.len() {
                    return None;
                }
                let mut bindings = vec![];
                for (p, v) in pats.iter().zip(items.as_ref()) {
                    bindings.extend(self.try_match_pattern(&p.node, v)?);
                }
                Some(bindings)
            }
            Pattern::List { elems, rest } => {
                let Value::List(items) = value else {
                    return None;
                };
                if rest.is_some() {
                    if items.len() < elems.len() {
                        return None;
                    }
                } else if items.len() != elems.len() {
                    return None;
                }
                let mut bindings = vec![];
                for (p, v) in elems.iter().zip(items.as_ref()) {
                    bindings.extend(self.try_match_pattern(&p.node, v)?);
                }
                if let Some(rest_name) = rest {
                    let remaining = items[elems.len()..].to_vec();
                    bindings.push((rest_name.clone(), Value::List(Arc::new(remaining))));
                }
                Some(bindings)
            }
            Pattern::Record { fields, rest } => {
                let Value::Record(rec) = value else {
                    return None;
                };
                let mut bindings = vec![];
                for fp in fields {
                    let val = rec.get(&fp.name)?;
                    if let Some(sub_pat) = &fp.pattern {
                        bindings.extend(self.try_match_pattern(&sub_pat.node, val)?);
                    } else {
                        bindings.push((fp.name.clone(), val.clone()));
                    }
                }
                if let Some(rest_name) = rest {
                    let matched_keys: std::collections::HashSet<&str> =
                        fields.iter().map(|f| f.name.as_str()).collect();
                    let remaining: indexmap::IndexMap<String, Value> = rec
                        .iter()
                        .filter(|(k, _)| !matched_keys.contains(k.as_str()))
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect();
                    bindings.push((rest_name.clone(), Value::Record(Arc::new(remaining))));
                }
                Some(bindings)
            }
            Pattern::Constructor { name, args } => match (name.as_str(), value) {
                ("Ok", Value::Ok(v)) if args.len() == 1 => self.try_match_pattern(&args[0].node, v),
                ("Err", Value::Err(v)) if args.len() == 1 => {
                    self.try_match_pattern(&args[0].node, v)
                }
                ("Some", Value::Some(v)) if args.len() == 1 => {
                    self.try_match_pattern(&args[0].node, v)
                }
                ("None", Value::None) if args.is_empty() => Some(vec![]),
                (tag, Value::Tagged { tag: vtag, values })
                    if tag == vtag.as_ref() && args.len() == values.len() =>
                {
                    let mut bindings = vec![];
                    for (p, v) in args.iter().zip(values.as_ref()) {
                        bindings.extend(self.try_match_pattern(&p.node, v)?);
                    }
                    Some(bindings)
                }
                _ => None,
            },
        }
    }
}
