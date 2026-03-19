use std::fmt;
use std::sync::Arc;

use num_bigint::BigInt;

use crate::value::{BuiltinFunc, Value};

impl From<BigInt> for Value {
    fn from(n: BigInt) -> Self {
        Value::Int(n)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(BigInt::from(n))
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(Arc::from(s))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(Arc::from(s.as_str()))
    }
}

impl From<Arc<str>> for Value {
    fn from(s: Arc<str>) -> Self {
        Value::Str(s)
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(items: Vec<T>) -> Self {
        Value::List(Arc::new(items.into_iter().map(Into::into).collect()))
    }
}

impl TryFrom<&Value> for BigInt {
    type Error = &'static str;
    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Int(n) => Ok(n.clone()),
            _ => Result::Err("expected Int"),
        }
    }
}

impl TryFrom<&Value> for f64 {
    type Error = &'static str;
    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Float(f) => Ok(*f),
            _ => Result::Err("expected Float"),
        }
    }
}

impl TryFrom<&Value> for bool {
    type Error = &'static str;
    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        match v {
            Value::Bool(b) => Ok(*b),
            _ => Result::Err("expected Bool"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Regex(r) => write!(f, "r/{}/", r.as_str()),
            Value::Unit => write!(f, "()"),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Value::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, ")")
            }
            Value::Record(fields) => {
                write!(f, "{{")?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, "  ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::Map(entries) => {
                write!(f, "Map{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, "  ")?;
                    }
                    write!(f, "{}: {v}", k.0)?;
                }
                write!(f, "}}")
            }
            Value::Func(_) => write!(f, "<func>"),
            Value::BuiltinFunc(b) => write!(f, "<builtin {}/{}>", b.name, b.arity),
            Value::Ok(v) => write!(f, "Ok {v}"),
            Value::Err(v) => write!(f, "Err {v}"),
            Value::Some(v) => write!(f, "Some {v}"),
            Value::None => write!(f, "None"),
            Value::Tagged { tag, values } => {
                write!(f, "{tag}")?;
                for v in values.iter() {
                    write!(f, " {v}")?;
                }
                Ok(())
            }
            Value::TaggedCtor { tag, .. } => write!(f, "<ctor {tag}>"),
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                if *inclusive {
                    write!(f, "{start}..={end}")
                } else {
                    write!(f, "{start}..{end}")
                }
            }
            Value::Protocol { name, .. } => write!(f, "<Protocol {name}>"),
            Value::ProtocolUnion { name, .. } => write!(f, "<Protocol {name}>"),
            Value::McpDecl { name, .. } => write!(f, "<MCP {name}>"),
            Value::Trait { name, .. } => write!(f, "<Trait {name}>"),
            Value::Agent { name, .. } => write!(f, "<Agent {name}>"),
            Value::Class { name, .. } => write!(f, "<Class {name}>"),
            Value::Object { class_name, id, .. } => write!(f, "<{class_name}#{id}>"),
            Value::Store { id } => write!(f, "<Store#{id}>"),
        }
    }
}

impl fmt::Debug for BuiltinFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<builtin {}/{}>", self.name, self.arity)
    }
}
