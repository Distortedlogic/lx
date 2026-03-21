use std::sync::Arc;
use std::sync::mpsc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use parking_lot::Mutex;
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
