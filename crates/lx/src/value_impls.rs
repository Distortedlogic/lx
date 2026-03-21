use std::hash::{Hash, Hasher};

use crate::value::{LxVal, ValueKey};

#[macro_export]
macro_rules! record {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut m = indexmap::IndexMap::new();
        $(m.insert(String::from($key), $val);)*
        $crate::value::LxVal::Record(std::sync::Arc::new(m))
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

impl PartialEq for LxVal {
    fn eq(&self, other: &Self) -> bool {
        self.structural_eq(other)
    }
}

impl LxVal {
    pub(crate) fn structural_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LxVal::Int(a), LxVal::Int(b)) => a == b,
            (LxVal::Float(a), LxVal::Float(b)) => a.to_bits() == b.to_bits(),
            (LxVal::Bool(a), LxVal::Bool(b)) => a == b,
            (LxVal::Str(a), LxVal::Str(b)) => a == b,
            (LxVal::Regex(a), LxVal::Regex(b)) => a.as_str() == b.as_str(),
            (LxVal::Unit, LxVal::Unit) => true,
            (LxVal::List(a), LxVal::List(b)) => a == b,
            (LxVal::Tuple(a), LxVal::Tuple(b)) => a == b,
            (LxVal::Record(a), LxVal::Record(b)) => {
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
            (LxVal::Map(a), LxVal::Map(b)) => a == b,
            (LxVal::Ok(a), LxVal::Ok(b)) => a == b,
            (LxVal::Err(a), LxVal::Err(b)) => a == b,
            (LxVal::Some(a), LxVal::Some(b)) => a == b,
            (LxVal::None, LxVal::None) => true,
            (
                LxVal::Tagged {
                    tag: t1,
                    values: v1,
                },
                LxVal::Tagged {
                    tag: t2,
                    values: v2,
                },
            ) => t1 == t2 && v1 == v2,
            (
                LxVal::Range {
                    start: s1,
                    end: e1,
                    inclusive: i1,
                },
                LxVal::Range {
                    start: s2,
                    end: e2,
                    inclusive: i2,
                },
            ) => s1 == s2 && e1 == e2 && i1 == i2,
            (LxVal::TraitUnion { name: n1, .. }, LxVal::TraitUnion { name: n2, .. }) => n1 == n2,
            (LxVal::McpDecl { name: n1, .. }, LxVal::McpDecl { name: n2, .. }) => n1 == n2,
            (LxVal::Trait { name: n1, .. }, LxVal::Trait { name: n2, .. }) => n1 == n2,
            (LxVal::Class { name: n1, .. }, LxVal::Class { name: n2, .. }) => n1 == n2,
            (LxVal::Object { id: i1, .. }, LxVal::Object { id: i2, .. }) => i1 == i2,
            (LxVal::Store { id: i1 }, LxVal::Store { id: i2 }) => i1 == i2,
            (LxVal::Stream { .. }, _) | (_, LxVal::Stream { .. }) => false,
            (LxVal::Func(_), _) | (_, LxVal::Func(_)) => false,
            (LxVal::BuiltinFunc(_), _) | (_, LxVal::BuiltinFunc(_)) => false,
            _ => false,
        }
    }

    pub(crate) fn hash_value<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            LxVal::Int(n) => n.hash(state),
            LxVal::Float(f) => f.to_bits().hash(state),
            LxVal::Bool(b) => b.hash(state),
            LxVal::Str(s) => s.hash(state),
            LxVal::Regex(r) => r.as_str().hash(state),
            LxVal::Unit => {}
            LxVal::List(items) | LxVal::Tuple(items) => {
                items.len().hash(state);
                for item in items.iter() {
                    item.hash_value(state);
                }
            }
            LxVal::Record(fields) => {
                fields.len().hash(state);
                let mut pairs: Vec<_> = fields.iter().collect();
                pairs.sort_by(|a, b| a.0.cmp(b.0));
                for (k, v) in pairs {
                    k.hash(state);
                    v.hash_value(state);
                }
            }
            LxVal::Map(entries) => {
                entries.len().hash(state);
                for (k, v) in entries.iter() {
                    k.hash(state);
                    v.hash_value(state);
                }
            }
            LxVal::Ok(v) | LxVal::Err(v) | LxVal::Some(v) => v.hash_value(state),
            LxVal::None => {}
            LxVal::Tagged { tag, values } => {
                tag.hash(state);
                for v in values.iter() {
                    v.hash_value(state);
                }
            }
            LxVal::Range {
                start,
                end,
                inclusive,
            } => {
                start.hash(state);
                end.hash(state);
                inclusive.hash(state);
            }
            LxVal::TraitUnion { name, .. } => name.hash(state),
            LxVal::McpDecl { name, .. } => name.hash(state),
            LxVal::Trait { name, .. } => name.hash(state),
            LxVal::Class { name, .. } => name.hash(state),
            LxVal::Object { id, .. } => id.hash(state),
            LxVal::Store { id } => id.hash(state),
            LxVal::Func(_)
            | LxVal::BuiltinFunc(_)
            | LxVal::TaggedCtor { .. }
            | LxVal::Stream { .. } => {}
        }
    }
}
