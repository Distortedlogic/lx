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
    Func(Box<LxFunc>),
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
        fields: Arc<Vec<ProtoFieldDef>>,
        methods: Arc<Vec<TraitMethodDef>>,
        defaults: Arc<IndexMap<String, Value>>,
        requires: Arc<Vec<Arc<str>>>,
        description: Option<Arc<str>>,
        tags: Arc<Vec<Arc<str>>>,
    },
    Class {
        name: Arc<str>,
        traits: Arc<Vec<Arc<str>>>,
        defaults: Arc<IndexMap<String, Value>>,
        methods: Arc<IndexMap<String, Value>>,
    },
    Object {
        class_name: Arc<str>,
        id: u64,
        traits: Arc<Vec<Arc<str>>>,
        methods: Arc<IndexMap<String, Value>>,
    },
    Store {
        id: u64,
    },
}

#[derive(Debug, Clone)]
pub struct ValueKey(pub Value);

impl Value {
    pub fn as_int(&self) -> Option<&BigInt> {
        match self {
            Value::Int(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&Arc<Vec<Value>>> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn str_field(&self, key: &str) -> Option<&str> {
        match self {
            Value::Record(fields) => fields.get(key).and_then(|v| v.as_str()),
            _ => None,
        }
    }

    pub fn int_field(&self, key: &str) -> Option<&BigInt> {
        match self {
            Value::Record(fields) => fields.get(key).and_then(|v| v.as_int()),
            _ => None,
        }
    }

    pub fn float_field(&self, key: &str) -> Option<f64> {
        match self {
            Value::Record(fields) => fields.get(key).and_then(|v| v.as_float()),
            _ => None,
        }
    }

    pub fn bool_field(&self, key: &str) -> Option<bool> {
        match self {
            Value::Record(fields) => fields.get(key).and_then(|v| v.as_bool()),
            _ => None,
        }
    }

    pub fn list_field(&self, key: &str) -> Option<&[Value]> {
        match self {
            Value::Record(fields) => fields
                .get(key)
                .and_then(|v| v.as_list())
                .map(|l| l.as_slice()),
            _ => None,
        }
    }

    pub fn record_field(&self, key: &str) -> Option<&IndexMap<String, Value>> {
        match self {
            Value::Record(fields) => fields.get(key).and_then(|v| match v {
                Value::Record(inner) => Some(inner.as_ref()),
                _ => None,
            }),
            _ => None,
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

#[derive(Debug, Clone)]
pub struct TraitMethodDef {
    pub name: String,
    pub input: Vec<ProtoFieldDef>,
    pub output: McpOutputDef,
}
