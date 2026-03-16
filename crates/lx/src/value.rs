use std::hash::{Hash, Hasher};
use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use strum::IntoStaticStr;

use crate::ast::SExpr;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;

#[derive(Debug, Clone, IntoStaticStr)]
pub enum Value {
    Int(BigInt),
    Float(f64),
    Bool(bool),
    Str(Arc<str>),
    Regex(Arc<regex::Regex>),
    Unit,

    List(Arc<Vec<Value>>),
    Record(Arc<IndexMap<String, Value>>),
    Map(Arc<IndexMap<ValueKey, Value>>),
    Tuple(Arc<Vec<Value>>),

    #[strum(serialize = "Func")]
    Func(LxFunc),
    #[strum(serialize = "Func")]
    BuiltinFunc(BuiltinFunc),

    Ok(Box<Value>),
    Err(Box<Value>),
    Some(Box<Value>),
    None,

    Tagged {
        tag: Arc<str>,
        values: Arc<Vec<Value>>,
    },
    #[strum(serialize = "Func")]
    TaggedCtor {
        tag: Arc<str>,
        arity: usize,
        applied: Vec<Value>,
    },
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    Protocol {
        name: Arc<str>,
        fields: Arc<Vec<ProtoFieldDef>>,
    },
    ProtocolUnion {
        name: Arc<str>,
        variants: Arc<Vec<Arc<str>>>,
    },
    McpDecl {
        name: Arc<str>,
        tools: Arc<Vec<McpToolDef>>,
    },
    Trait {
        name: Arc<str>,
        handles: Arc<Vec<Arc<str>>>,
        provides: Arc<Vec<Arc<str>>>,
        requires: Arc<Vec<Arc<str>>>,
    },
}

#[derive(Debug, Clone)]
pub struct ValueKey(pub Value);

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
    fn structural_eq(&self, other: &Self) -> bool {
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
                a_sorted.sort_by_key(|(k, _)| (*k).clone());
                b_sorted.sort_by_key(|(k, _)| (*k).clone());
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
            (Value::Func(_), _) | (_, Value::Func(_)) => false,
            (Value::BuiltinFunc(_), _) | (_, Value::BuiltinFunc(_)) => false,
            _ => false,
        }
    }

    fn hash_value<H: Hasher>(&self, state: &mut H) {
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
                pairs.sort_by_key(|(k, _)| (*k).clone());
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
            Value::Func(_) | Value::BuiltinFunc(_) | Value::TaggedCtor { .. } => {}
        }
    }

    pub fn as_int(&self) -> Option<&BigInt> {
        match self {
            Value::Int(n) => Some(n),
            _ => Option::None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => Option::None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => Option::None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => Option::None,
        }
    }

    pub fn as_list(&self) -> Option<&Arc<Vec<Value>>> {
        match self {
            Value::List(l) => Some(l),
            _ => Option::None,
        }
    }

    pub fn is_truthy_err(&self) -> bool {
        matches!(self, Value::Err(_) | Value::None)
    }

    pub fn type_name(&self) -> &'static str {
        self.into()
    }

    pub fn short_display(&self) -> String {
        let s = format!("{self}");
        if s.len() > 80 {
            format!("{}...", &s[..77])
        } else {
            s
        }
    }
}

#[derive(Debug, Clone)]
pub struct LxFunc {
    pub params: Vec<String>,
    pub defaults: Vec<Option<Value>>,
    pub body: Arc<SExpr>,
    pub closure: Arc<Env>,
    pub arity: usize,
    pub applied: Vec<Value>,
    pub source_text: Arc<str>,
    pub source_name: Arc<str>,
}

pub type BuiltinFn =
    fn(&[Value], Span, &std::sync::Arc<crate::backends::RuntimeCtx>) -> Result<Value, LxError>;

#[derive(Clone)]
pub struct BuiltinFunc {
    pub name: &'static str,
    pub arity: usize,
    pub func: BuiltinFn,
    pub applied: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct ProtoFieldDef {
    pub name: String,
    pub type_name: String,
    pub default: Option<Value>,
    pub constraint: Option<Arc<crate::ast::SExpr>>,
}

#[derive(Debug, Clone)]
pub struct McpToolDef {
    pub name: String,
    pub input: Vec<ProtoFieldDef>,
    pub output: McpOutputDef,
}

#[derive(Debug, Clone)]
pub enum McpOutputDef {
    Simple(String),
    Record(Vec<ProtoFieldDef>),
    List(Box<McpOutputDef>),
}
