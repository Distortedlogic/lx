use crate::sym::intern;
use std::sync::Arc;

use crate::ast::{Literal, MatchArm, Pattern, SExpr};
use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

impl super::Interpreter {
  pub(super) async fn eval_match(&mut self, scrutinee: &SExpr, arms: &[MatchArm], span: SourceSpan) -> Result<LxVal, LxError> {
    let val = self.eval(scrutinee).await?;
    for arm in arms {
      if let Some(bindings) = self.try_match_pattern(&arm.pattern.node, &val) {
        let saved = Arc::clone(&self.env);
        let mut scope = self.env.child();
        for (name, v) in bindings {
          scope.bind(intern(&name), v);
        }
        self.env = scope.into_arc();
        if let Some(guard) = &arm.guard {
          let gv = self.eval(guard).await?;
          match gv.as_bool() {
            Some(false) => {
              self.env = saved;
              continue;
            },
            Some(true) => {},
            _ => {
              self.env = saved;
              return Err(LxError::type_err(format!("match guard must be Bool, got {} `{}`", gv.type_name(), gv.short_display()), span));
            },
          }
        }
        let result = self.eval(&arm.body).await;
        self.env = saved;
        return result;
      }
    }
    Err(LxError::runtime(format!("no matching pattern for {} `{}`", val.type_name(), val.short_display()), span))
  }

  pub(super) fn try_match_pattern(&self, pattern: &Pattern, value: &LxVal) -> Option<Vec<(String, LxVal)>> {
    match pattern {
      Pattern::Wildcard => Some(vec![]),
      Pattern::Bind(name) => Some(vec![(name.clone(), value.clone())]),
      Pattern::Literal(lit) => {
        let matches = match (lit, value) {
          (Literal::Int(a), LxVal::Int(b)) => a == b,
          (Literal::Float(a), LxVal::Float(b)) => a.to_bits() == b.to_bits(),
          (Literal::Bool(a), LxVal::Bool(b)) => a == b,
          (Literal::Unit, LxVal::Unit) => true,
          (Literal::RawStr(a), LxVal::Str(b)) => a.as_str() == b.as_ref(),
          (Literal::Str(parts), LxVal::Str(b)) => {
            let mut s = String::new();
            for part in parts {
              match part {
                crate::ast::StrPart::Text(t) => s.push_str(t),
                crate::ast::StrPart::Interp(_) => return None,
              }
            }
            s.as_str() == b.as_ref()
          },
          _ => false,
        };
        matches.then(Vec::new)
      },
      Pattern::Tuple(pats) => {
        let LxVal::Tuple(items) = value else {
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
      },
      Pattern::List { elems, rest } => {
        let LxVal::List(items) = value else {
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
          bindings.push((rest_name.clone(), LxVal::list(remaining)));
        }
        Some(bindings)
      },
      Pattern::Record { fields, rest } => {
        let LxVal::Record(rec) = value else {
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
          let matched_keys: std::collections::HashSet<&str> = fields.iter().map(|f| f.name.as_str()).collect();
          let remaining: indexmap::IndexMap<String, LxVal> =
            rec.iter().filter(|(k, _)| !matched_keys.contains(k.as_str())).map(|(k, v)| (k.clone(), v.clone())).collect();
          bindings.push((rest_name.clone(), LxVal::record(remaining)));
        }
        Some(bindings)
      },
      Pattern::Constructor { name, args } => match (name.as_str(), value) {
        ("Ok", LxVal::Ok(v)) if args.len() == 1 => self.try_match_pattern(&args[0].node, v),
        ("Err", LxVal::Err(v)) if args.len() == 1 => self.try_match_pattern(&args[0].node, v),
        ("Some", LxVal::Some(v)) if args.len() == 1 => self.try_match_pattern(&args[0].node, v),
        ("None", LxVal::None) if args.is_empty() => Some(vec![]),
        (tag, LxVal::Tagged { tag: vtag, values }) if tag == vtag.as_ref() && args.len() == values.len() => {
          let mut bindings = vec![];
          for (p, v) in args.iter().zip(values.as_ref()) {
            bindings.extend(self.try_match_pattern(&p.node, v)?);
          }
          Some(bindings)
        },
        _ => None,
      },
    }
  }
}
