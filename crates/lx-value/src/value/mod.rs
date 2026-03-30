mod display;
mod func;
mod impls;
mod methods;
mod serde_impl;

pub use func::{AsyncBuiltinFn, BuiltinFunc, BuiltinKind, DynAsyncBuiltinFn, LxFunc, SyncBuiltinFn, mk_dyn_async};

use std::sync::Arc;

use indexmap::IndexMap;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use strum::IntoStaticStr;

use crate::error::LxError;
use lx_ast::ast::{AstArena, ExprId, Field, MethodSpec};
use lx_span::sym::Sym;
use miette::SourceSpan;

pub type FieldDef = Field<LxVal, ConstraintExpr>;

#[derive(Debug, Clone)]
pub struct ConstraintExpr {
  pub expr_id: ExprId,
  pub arena: Arc<AstArena>,
}
pub type TraitMethodDef = MethodSpec<FieldDef>;

#[derive(Debug, Clone)]
pub struct LxTrait {
  pub name: Sym,
  pub fields: Arc<Vec<FieldDef>>,
  pub methods: Arc<Vec<TraitMethodDef>>,
  pub defaults: Arc<IndexMap<Sym, LxVal>>,
  pub requires: Arc<Vec<Sym>>,
  pub description: Option<Sym>,
  pub tags: Arc<Vec<Sym>>,
}

#[derive(Debug, Clone)]
pub struct LxClass {
  pub name: Sym,
  pub traits: Arc<Vec<Sym>>,
  pub defaults: Arc<IndexMap<Sym, LxVal>>,
  pub methods: Arc<IndexMap<Sym, LxVal>>,
}

#[derive(Debug, Clone)]
pub struct LxObject {
  pub class_name: Sym,
  pub id: u64,
  pub traits: Arc<Vec<Sym>>,
  pub methods: Arc<IndexMap<Sym, LxVal>>,
}

#[derive(Debug, Clone, IntoStaticStr, derive_more::From)]
pub enum LxVal {
  #[from(BigInt, i64)]
  Int(BigInt),
  #[from]
  Float(f64),
  #[from]
  Bool(bool),
  #[from(Arc<str>, String)]
  Str(Arc<str>),
  Unit,

  List(Arc<Vec<LxVal>>),
  Record(Arc<IndexMap<Sym, LxVal>>),
  Map(Arc<IndexMap<ValueKey, LxVal>>),
  Tuple(Arc<Vec<LxVal>>),

  #[strum(serialize = "Func")]
  Func(Box<LxFunc>),
  #[strum(serialize = "Func")]
  MultiFunc(Vec<LxFunc>),
  #[strum(serialize = "Func")]
  BuiltinFunc(BuiltinFunc),

  Ok(Box<LxVal>),
  Err(Box<LxVal>),
  Some(Box<LxVal>),
  None,

  Tagged {
    tag: Sym,
    values: Arc<Vec<LxVal>>,
  },
  #[strum(serialize = "Func")]
  TaggedCtor {
    tag: Sym,
    arity: usize,
    applied: Vec<LxVal>,
  },
  Range {
    start: i64,
    end: i64,
    inclusive: bool,
  },
  TraitUnion {
    name: Sym,
    variants: Arc<Vec<Sym>>,
  },
  Trait(Box<LxTrait>),
  Class(Box<LxClass>),
  Object(Box<LxObject>),
  #[strum(serialize = "Type")]
  Type(Sym),
  Store {
    id: u64,
  },
  Stream {
    id: u64,
  },
  Channel {
    name: Sym,
  },
  #[strum(serialize = "ToolModule")]
  ToolModule(Arc<dyn crate::ToolModuleHandle>),
}

#[derive(Debug, Clone)]
pub struct ValueKey(pub LxVal);

pub enum KeyedRef<'a> {
  Record(&'a IndexMap<Sym, LxVal>),
  Map(&'a IndexMap<ValueKey, LxVal>),
}

macro_rules! require_methods {
  ($($name:ident, $as_method:ident, $type_label:expr, $ret:ty);+ $(;)?) => {
    $(
      pub fn $name(&self, ctx: &str, span: SourceSpan) -> Result<$ret, LxError> {
        self.$as_method().ok_or_else(|| LxError::type_err(
          format!("{ctx} expects {}, got {}", $type_label, self.type_name()), span, None))
      }
    )+
  };
}

macro_rules! typed_field_methods {
  ($($name:ident, $as_method:ident, $ret:ty);+ $(;)?) => {
    $(
      pub fn $name(&self, key: &str) -> Option<$ret> {
        match self {
          LxVal::Record(fields) => fields.get(&lx_span::sym::intern(key)).and_then(|v| v.$as_method()),
          _ => ::std::option::Option::None,
        }
      }
    )+
  };
}

impl LxVal {
  pub fn int(n: impl Into<BigInt>) -> Self {
    LxVal::Int(n.into())
  }
  pub fn str(s: impl AsRef<str>) -> Self {
    LxVal::Str(Arc::from(s.as_ref()))
  }
  pub fn list(items: Vec<LxVal>) -> Self {
    LxVal::List(Arc::new(items))
  }
  pub fn record(fields: IndexMap<Sym, LxVal>) -> Self {
    LxVal::Record(Arc::new(fields))
  }
  pub fn tuple(items: Vec<LxVal>) -> Self {
    LxVal::Tuple(Arc::new(items))
  }
  pub fn ok(v: LxVal) -> Self {
    LxVal::Ok(Box::new(v))
  }
  pub fn ok_unit() -> Self {
    LxVal::Ok(Box::new(LxVal::Unit))
  }
  pub fn some(v: LxVal) -> Self {
    LxVal::Some(Box::new(v))
  }
  pub fn err(v: LxVal) -> Self {
    LxVal::Err(Box::new(v))
  }
  pub fn err_str(s: impl AsRef<str>) -> Self {
    LxVal::Err(Box::new(LxVal::str(s)))
  }
  pub fn typ(name: &str) -> Self {
    LxVal::Type(lx_span::sym::intern(name))
  }

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

  require_methods! {
    require_str, as_str, "Str", &str;
    require_int, as_int, "Int", &BigInt;
    require_float, as_float, "Float", f64;
    require_bool, as_bool, "Bool", bool;
  }

  pub fn require_list(&self, ctx: &str, span: SourceSpan) -> Result<&[LxVal], LxError> {
    self.as_list().map(|l| l.as_slice()).ok_or_else(|| LxError::type_err(format!("{ctx} expects List, got {}", self.type_name()), span, None))
  }

  pub fn require_record(&self, ctx: &str, span: SourceSpan) -> Result<&IndexMap<Sym, LxVal>, LxError> {
    match self {
      LxVal::Record(r) => Ok(r.as_ref()),
      _ => Err(LxError::type_err(format!("{ctx} expects Record, got {}", self.type_name()), span, None)),
    }
  }

  pub fn require_usize(&self, ctx: &str, span: SourceSpan) -> Result<usize, LxError> {
    let n = self.require_int(ctx, span)?;
    n.to_usize().ok_or_else(|| LxError::type_err(format!("{ctx} expects non-negative Int that fits usize, got {n}"), span, None))
  }

  pub fn require_keyed(&self, ctx: &str, span: SourceSpan) -> Result<KeyedRef<'_>, LxError> {
    match self {
      LxVal::Record(r) => Ok(KeyedRef::Record(r.as_ref())),
      LxVal::Map(m) => Ok(KeyedRef::Map(m.as_ref())),
      _ => Err(LxError::type_err(format!("{ctx} expects Record or Map, got {}", self.type_name()), span, None)),
    }
  }

  typed_field_methods! {
    str_field, as_str, &str;
    int_field, as_int, &BigInt;
    float_field, as_float, f64;
    bool_field, as_bool, bool;
  }

  pub fn list_field(&self, key: &str) -> Option<&[LxVal]> {
    match self {
      LxVal::Record(fields) => fields.get(&lx_span::sym::intern(key)).and_then(|v| v.as_list()).map(|l| l.as_slice()),
      _ => ::std::option::Option::None,
    }
  }

  pub fn record_field(&self, key: &str) -> Option<&IndexMap<Sym, LxVal>> {
    match self {
      LxVal::Record(fields) => fields.get(&lx_span::sym::intern(key)).and_then(|v| match v {
        LxVal::Record(inner) => Some(inner.as_ref()),
        _ => ::std::option::Option::None,
      }),
      _ => ::std::option::Option::None,
    }
  }

  pub fn get_field(&self, key: &str) -> Option<&LxVal> {
    match self {
      LxVal::Record(fields) => fields.get(&lx_span::sym::intern(key)),
      _ => ::std::option::Option::None,
    }
  }

  pub fn is_truthy_err(&self) -> bool {
    matches!(self, LxVal::Err(_) | LxVal::None)
  }

  pub fn type_name(&self) -> &'static str {
    self.into()
  }

  pub fn short_display(&self) -> String {
    let s = self.to_string();
    if s.len() > 80 { format!("{}...", &s[..77]) } else { s }
  }
}
