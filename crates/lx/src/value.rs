use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
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
        methods: Arc<Vec<TraitMethodDef>>,
        defaults: Arc<IndexMap<String, Value>>,
        requires: Arc<Vec<Arc<str>>>,
        description: Option<Arc<str>>,
        tags: Arc<Vec<Arc<str>>>,
    },
    Agent {
        name: Arc<str>,
        traits: Arc<Vec<Arc<str>>>,
        methods: Arc<IndexMap<String, Value>>,
        init: Option<Box<Value>>,
        uses: Arc<Vec<(Arc<str>, Arc<str>)>>,
        on: Option<Box<Value>>,
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

static OBJECTS: LazyLock<DashMap<u64, IndexMap<String, Value>>> = LazyLock::new(DashMap::new);
static NEXT_OBJ_ID: AtomicU64 = AtomicU64::new(1);

pub fn object_store_insert(fields: IndexMap<String, Value>) -> u64 {
    let id = NEXT_OBJ_ID.fetch_add(1, Ordering::Relaxed);
    OBJECTS.insert(id, fields);
    id
}

pub fn object_store_get_field(id: u64, field: &str) -> Option<Value> {
    OBJECTS.get(&id).and_then(|f| f.get(field).cloned())
}

pub fn object_store_set_field(id: u64, field: &str, value: Value) {
    if let Some(mut f) = OBJECTS.get_mut(&id) {
        f.insert(field.to_string(), value);
    }
}

pub fn object_store_update_nested(id: u64, path: &[String], value: Value) -> Result<(), String> {
    let Some(mut fields) = OBJECTS.get_mut(&id) else {
        return Err("object not found".into());
    };
    match path {
        [field] => {
            fields.insert(field.clone(), value);
            Ok(())
        }
        [field, rest @ ..] => {
            let inner = fields
                .get(field)
                .ok_or_else(|| format!("field '{field}' not found"))?
                .clone();
            let updated = update_nested_record(&inner, rest, value)?;
            fields.insert(field.clone(), updated);
            Ok(())
        }
        [] => Err("empty field path".into()),
    }
}

fn update_nested_record(val: &Value, path: &[String], new_val: Value) -> Result<Value, String> {
    let Value::Record(rec) = val else {
        return Err(format!(
            "field update requires Record, got {}",
            val.type_name()
        ));
    };
    match path {
        [field] => {
            let mut new_rec = rec.as_ref().clone();
            new_rec.insert(field.clone(), new_val);
            Ok(Value::Record(Arc::new(new_rec)))
        }
        [field, rest @ ..] => {
            let inner = rec
                .get(field)
                .ok_or_else(|| format!("field '{field}' not found"))?;
            let updated = update_nested_record(inner, rest, new_val)?;
            let mut new_rec = rec.as_ref().clone();
            new_rec.insert(field.clone(), updated);
            Ok(Value::Record(Arc::new(new_rec)))
        }
        [] => Err("empty field path".into()),
    }
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
