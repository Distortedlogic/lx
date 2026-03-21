use std::sync::Arc;
use std::sync::mpsc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use parking_lot::Mutex;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::ser::SerializeMap;
use strum::IntoStaticStr;

use crate::ast::SExpr;
use crate::env::Env;
use crate::error::LxError;
use crate::span::Span;

#[derive(Debug, Clone, IntoStaticStr)]
pub enum LxVal {
    Int(BigInt),
    Float(f64),
    Bool(bool),
    Str(Arc<str>),
    Regex(Arc<regex::Regex>),
    Unit,

    List(Arc<Vec<LxVal>>),
    Record(Arc<IndexMap<String, LxVal>>),
    Map(Arc<IndexMap<ValueKey, LxVal>>),
    Tuple(Arc<Vec<LxVal>>),

    #[strum(serialize = "Func")]
    Func(Box<LxFunc>),
    #[strum(serialize = "Func")]
    BuiltinFunc(BuiltinFunc),

    Ok(Box<LxVal>),
    Err(Box<LxVal>),
    Some(Box<LxVal>),
    None,

    Tagged {
        tag: Arc<str>,
        values: Arc<Vec<LxVal>>,
    },
    #[strum(serialize = "Func")]
    TaggedCtor {
        tag: Arc<str>,
        arity: usize,
        applied: Vec<LxVal>,
    },
    Range {
        start: i64,
        end: i64,
        inclusive: bool,
    },
    TraitUnion {
        name: Arc<str>,
        variants: Arc<Vec<Arc<str>>>,
    },
    McpDecl {
        name: Arc<str>,
        tools: Arc<Vec<McpToolDef>>,
    },
    Trait {
        name: Arc<str>,
        fields: Arc<Vec<FieldDef>>,
        methods: Arc<Vec<TraitMethodDef>>,
        defaults: Arc<IndexMap<String, LxVal>>,
        requires: Arc<Vec<Arc<str>>>,
        description: Option<Arc<str>>,
        tags: Arc<Vec<Arc<str>>>,
    },
    Class {
        name: Arc<str>,
        traits: Arc<Vec<Arc<str>>>,
        defaults: Arc<IndexMap<String, LxVal>>,
        methods: Arc<IndexMap<String, LxVal>>,
    },
    Object {
        class_name: Arc<str>,
        id: u64,
        traits: Arc<Vec<Arc<str>>>,
        methods: Arc<IndexMap<String, LxVal>>,
    },
    Store {
        id: u64,
    },
    Stream {
        rx: Arc<Mutex<mpsc::Receiver<LxVal>>>,
        cancel_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
    },
}

#[derive(Debug, Clone)]
pub struct ValueKey(pub LxVal);

impl LxVal {
    pub fn as_int(&self) -> Option<&BigInt> {
        match self {
            LxVal::Int(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            LxVal::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            LxVal::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            LxVal::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Arc<Vec<LxVal>>> {
        match self {
            LxVal::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn str_field(&self, key: &str) -> Option<&str> {
        match self {
            LxVal::Record(fields) => fields.get(key).and_then(|v| v.as_str()),
            _ => None,
        }
    }

    pub fn int_field(&self, key: &str) -> Option<&BigInt> {
        match self {
            LxVal::Record(fields) => fields.get(key).and_then(|v| v.as_int()),
            _ => None,
        }
    }

    pub fn float_field(&self, key: &str) -> Option<f64> {
        match self {
            LxVal::Record(fields) => fields.get(key).and_then(|v| v.as_float()),
            _ => None,
        }
    }

    pub fn bool_field(&self, key: &str) -> Option<bool> {
        match self {
            LxVal::Record(fields) => fields.get(key).and_then(|v| v.as_bool()),
            _ => None,
        }
    }

    pub fn list_field(&self, key: &str) -> Option<&[LxVal]> {
        match self {
            LxVal::Record(fields) => fields
                .get(key)
                .and_then(|v| v.as_list())
                .map(|l| l.as_slice()),
            _ => None,
        }
    }

    pub fn record_field(&self, key: &str) -> Option<&IndexMap<String, LxVal>> {
        match self {
            LxVal::Record(fields) => fields.get(key).and_then(|v| match v {
                LxVal::Record(inner) => Some(inner.as_ref()),
                _ => None,
            }),
            _ => None,
        }
    }

    pub fn is_truthy_err(&self) -> bool {
        matches!(self, LxVal::Err(_) | LxVal::None)
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
    pub defaults: Vec<Option<LxVal>>,
    pub body: Arc<SExpr>,
    pub closure: Arc<Env>,
    pub arity: usize,
    pub applied: Vec<LxVal>,
    pub source_text: Arc<str>,
    pub source_name: Arc<str>,
}

pub type SyncBuiltinFn =
    fn(&[LxVal], Span, &std::sync::Arc<crate::backends::RuntimeCtx>) -> Result<LxVal, LxError>;

pub type AsyncBuiltinFn =
    fn(
        Vec<LxVal>,
        Span,
        std::sync::Arc<crate::backends::RuntimeCtx>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<LxVal, LxError>>>>;

#[derive(Clone, Copy)]
pub enum BuiltinKind {
    Sync(SyncBuiltinFn),
    Async(AsyncBuiltinFn),
}

#[derive(Clone)]
pub struct BuiltinFunc {
    pub name: &'static str,
    pub arity: usize,
    pub kind: BuiltinKind,
    pub applied: Vec<LxVal>,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub type_name: String,
    pub default: Option<LxVal>,
    pub constraint: Option<Arc<crate::ast::SExpr>>,
}

#[derive(Debug, Clone)]
pub struct McpToolDef {
    pub name: String,
    pub input: Vec<FieldDef>,
    pub output: McpOutputDef,
}

#[derive(Debug, Clone)]
pub enum McpOutputDef {
    Simple(String),
    Record(Vec<FieldDef>),
    List(Box<McpOutputDef>),
}

#[derive(Debug, Clone)]
pub struct TraitMethodDef {
    pub name: String,
    pub input: Vec<FieldDef>,
    pub output: McpOutputDef,
}

impl Serialize for LxVal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            LxVal::Int(n) => {
                if let Ok(i) = i64::try_from(n) {
                    serializer.serialize_i64(i)
                } else {
                    serializer.serialize_str(&n.to_string())
                }
            }
            LxVal::Float(f) => serializer.serialize_f64(*f),
            LxVal::Bool(b) => serializer.serialize_bool(*b),
            LxVal::Str(s) => serializer.serialize_str(s),
            LxVal::Unit | LxVal::None => serializer.serialize_none(),
            LxVal::List(items) => items.serialize(serializer),
            LxVal::Tuple(items) => items.serialize(serializer),
            LxVal::Record(fields) => {
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (k, v) in fields.as_ref() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            LxVal::Map(entries) => {
                let mut map = serializer.serialize_map(Some(entries.len()))?;
                for (k, v) in entries.as_ref() {
                    map.serialize_entry(&format!("{}", k.0), v)?;
                }
                map.end()
            }
            LxVal::Ok(v) => v.serialize(serializer),
            LxVal::Some(v) => v.serialize(serializer),
            LxVal::Err(v) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("__err", v.as_ref())?;
                map.end()
            }
            LxVal::Tagged { tag, values } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("__tag", tag.as_ref())?;
                map.serialize_entry("__values", values.as_ref())?;
                map.end()
            }
            LxVal::Range { start, end, inclusive } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("start", start)?;
                map.serialize_entry("end", end)?;
                map.serialize_entry("inclusive", inclusive)?;
                map.end()
            }
            LxVal::Store { id } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("__store", id)?;
                map.end()
            }
            LxVal::Object { class_name, id, .. } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("__object", &id)?;
                map.serialize_entry("__class", class_name.as_ref())?;
                map.end()
            }
            _ => serializer.serialize_str(&format!("<{}>", self.type_name())),
        }
    }
}

impl<'de> Deserialize<'de> for LxVal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json = serde_json::Value::deserialize(deserializer)?;
        Ok(LxVal::from(json))
    }
}

impl From<serde_json::Value> for LxVal {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::Null => LxVal::None,
            serde_json::Value::Bool(b) => LxVal::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    LxVal::Int(BigInt::from(i))
                } else {
                    LxVal::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_json::Value::String(s) => LxVal::Str(Arc::from(s.as_str())),
            serde_json::Value::Array(arr) => {
                LxVal::List(Arc::new(arr.into_iter().map(LxVal::from).collect()))
            }
            serde_json::Value::Object(obj) => {
                let mut rec = IndexMap::new();
                for (k, v) in obj {
                    rec.insert(k, LxVal::from(v));
                }
                LxVal::Record(Arc::new(rec))
            }
        }
    }
}

impl From<&LxVal> for serde_json::Value {
    fn from(v: &LxVal) -> Self {
        serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
    }
}
