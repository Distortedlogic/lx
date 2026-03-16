use std::hash::{Hash, Hasher};

use crate::value::{Value, ValueKey};

#[macro_export]
macro_rules! record {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = indexmap::IndexMap::new();
        $(m.insert(String::from($key), $val);)*
        $crate::value::Value::Record(std::sync::Arc::new(m))
    }};
}

impl PartialEq for ValueKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.structural_eq(&other.0)
    }
}

impl Eq for ValueKey {}

impl Hash for ValueKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash_value(state);
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.structural_eq(other)
    }
}

impl Value {
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Regex(a), Value::Regex(b)) => a.as_str() == b.as_str(),
            (Value::Unit, Value::Unit) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Record(a), Value::Record(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                let mut a_sorted: Vec<_> = a.iter().collect();
                let mut b_sorted: Vec<_> = b.iter().collect();
                a_sorted.sort_by(|a, b| a.0.cmp(b.0));
                b_sorted.sort_by(|a, b| a.0.cmp(b.0));
                a_sorted
                    .iter()
                    .zip(b_sorted.iter())
                    .all(|((ak, av), (bk, bv))| ak == bk && av == bv)
            }
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Ok(a), Value::Ok(b)) => a == b,
            (Value::Err(a), Value::Err(b)) => a == b,
            (Value::Some(a), Value::Some(b)) => a == b,
            (Value::None, Value::None) => true,
            (
                Value::Tagged {
                    tag: t1,
                    values: v1,
                },
                Value::Tagged {
                    tag: t2,
                    values: v2,
                },
            ) => t1 == t2 && v1 == v2,
            (
                Value::Range {
                    start: s1,
                    end: e1,
                    inclusive: i1,
                },
                Value::Range {
                    start: s2,
                    end: e2,
                    inclusive: i2,
                },
            ) => s1 == s2 && e1 == e2 && i1 == i2,
            (Value::Protocol { name: n1, .. }, Value::Protocol { name: n2, .. }) => n1 == n2,
            (Value::ProtocolUnion { name: n1, .. }, Value::ProtocolUnion { name: n2, .. }) => {
                n1 == n2
            }
            (Value::McpDecl { name: n1, .. }, Value::McpDecl { name: n2, .. }) => n1 == n2,
            (Value::Trait { name: n1, .. }, Value::Trait { name: n2, .. }) => n1 == n2,
            (Value::Agent { name: n1, .. }, Value::Agent { name: n2, .. }) => n1 == n2,
            (Value::Func(_), _) | (_, Value::Func(_)) => false,
            (Value::BuiltinFunc(_), _) | (_, Value::BuiltinFunc(_)) => false,
            _ => false,
        }
    }

    pub(crate) fn hash_value<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Int(n) => n.hash(state),
            Value::Float(f) => f.to_bits().hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Str(s) => s.hash(state),
            Value::Regex(r) => r.as_str().hash(state),
            Value::Unit => {}
            Value::List(items) | Value::Tuple(items) => {
                items.len().hash(state);
                for item in items.iter() {
                    item.hash_value(state);
                }
            }
            Value::Record(fields) => {
                fields.len().hash(state);
                let mut pairs: Vec<_> = fields.iter().collect();
                pairs.sort_by(|a, b| a.0.cmp(b.0));
                for (k, v) in pairs {
                    k.hash(state);
                    v.hash_value(state);
                }
            }
            Value::Map(entries) => {
                entries.len().hash(state);
                for (k, v) in entries.iter() {
                    k.hash(state);
                    v.hash_value(state);
                }
            }
            Value::Ok(v) | Value::Err(v) | Value::Some(v) => v.hash_value(state),
            Value::None => {}
            Value::Tagged { tag, values } => {
                tag.hash(state);
                for v in values.iter() {
                    v.hash_value(state);
                }
            }
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                start.hash(state);
                end.hash(state);
                inclusive.hash(state);
            }
            Value::Protocol { name, .. } => name.hash(state),
            Value::ProtocolUnion { name, .. } => name.hash(state),
            Value::McpDecl { name, .. } => name.hash(state),
            Value::Trait { name, .. } => name.hash(state),
            Value::Agent { name, .. } => name.hash(state),
            Value::Func(_) | Value::BuiltinFunc(_) | Value::TaggedCtor { .. } => {}
        }
    }
}
