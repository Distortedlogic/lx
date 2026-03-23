use std::collections::HashMap;

use crate::sym::intern;

use super::module_graph::ModuleSignature;
use super::type_arena::{TypeArena, TypeId};
use super::types::Type;

pub fn build_stdlib_signatures() -> HashMap<String, ModuleSignature> {
  let mut modules = HashMap::new();
  modules.insert("math".into(), build_math());
  modules.insert("fs".into(), build_fs());
  modules.insert("env".into(), build_env());
  modules.insert("channel".into(), build_channel());
  modules.insert("time".into(), build_time());
  for name in ["store", "stream", "test", "schema", "checkpoint", "introspect", "trait", "cron", "diag", "md", "sandbox"] {
    modules.insert(name.into(), empty_sig());
  }
  modules
}

fn empty_sig() -> ModuleSignature {
  ModuleSignature { bindings: HashMap::new(), types: HashMap::new(), traits: HashMap::new(), type_arena: TypeArena::new() }
}

fn func1(ta: &mut TypeArena, param: TypeId, ret: TypeId) -> TypeId {
  ta.alloc(Type::Func { param, ret })
}

fn func2(ta: &mut TypeArena, p1: TypeId, p2: TypeId, ret: TypeId) -> TypeId {
  let inner = ta.alloc(Type::Func { param: p2, ret });
  ta.alloc(Type::Func { param: p1, ret: inner })
}

fn build_math() -> ModuleSignature {
  let mut ta = TypeArena::new();
  let mut b = HashMap::new();
  let int = ta.int();
  let float = ta.float();

  b.insert(intern("abs"), func1(&mut ta, int, int));
  b.insert(intern("ceil"), func1(&mut ta, float, int));
  b.insert(intern("floor"), func1(&mut ta, float, int));
  b.insert(intern("round"), func1(&mut ta, float, int));
  b.insert(intern("sqrt"), func1(&mut ta, float, float));
  b.insert(intern("pow"), func2(&mut ta, int, int, int));
  b.insert(intern("min"), func2(&mut ta, int, int, int));
  b.insert(intern("max"), func2(&mut ta, int, int, int));
  b.insert(intern("pi"), float);
  b.insert(intern("e"), float);
  b.insert(intern("inf"), float);

  ModuleSignature { bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
}

fn build_fs() -> ModuleSignature {
  let mut ta = TypeArena::new();
  let mut b = HashMap::new();
  let str_t = ta.str();
  let bool_t = ta.bool();
  let unit = ta.unit();
  let unknown = ta.unknown();

  let result_str = ta.alloc(Type::Result { ok: str_t, err: str_t });
  let result_unit = ta.alloc(Type::Result { ok: unit, err: str_t });
  let list_str = ta.alloc(Type::List(str_t));
  let result_list = ta.alloc(Type::Result { ok: list_str, err: str_t });

  b.insert(intern("read"), func1(&mut ta, str_t, result_str));
  b.insert(intern("write"), func2(&mut ta, str_t, str_t, result_unit));
  b.insert(intern("append"), func2(&mut ta, str_t, str_t, result_unit));
  b.insert(intern("exists"), func1(&mut ta, str_t, bool_t));
  b.insert(intern("remove"), func1(&mut ta, str_t, result_unit));
  b.insert(intern("mkdir"), func1(&mut ta, str_t, result_unit));
  b.insert(intern("ls"), func1(&mut ta, str_t, result_list));
  b.insert(intern("stat"), func1(&mut ta, str_t, unknown));

  ModuleSignature { bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
}

fn build_env() -> ModuleSignature {
  let mut ta = TypeArena::new();
  let mut b = HashMap::new();
  let str_t = ta.str();
  let unit = ta.unit();
  let unknown = ta.unknown();

  let maybe_str = ta.alloc(Type::Maybe(str_t));
  let list_str = ta.alloc(Type::List(str_t));

  b.insert(intern("get"), func1(&mut ta, str_t, maybe_str));
  b.insert(intern("cwd"), func1(&mut ta, unit, str_t));
  b.insert(intern("home"), func1(&mut ta, unit, maybe_str));
  b.insert(intern("args"), func1(&mut ta, unit, list_str));
  b.insert(intern("vars"), func1(&mut ta, unit, unknown));

  ModuleSignature { bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
}

fn build_channel() -> ModuleSignature {
  let mut ta = TypeArena::new();
  let mut b = HashMap::new();
  let int = ta.int();
  let unit = ta.unit();
  let unknown = ta.unknown();
  let str_t = ta.str();

  let result_unit = ta.alloc(Type::Result { ok: unit, err: str_t });
  let result_unknown = ta.alloc(Type::Result { ok: unknown, err: unknown });
  let maybe_unknown = ta.alloc(Type::Maybe(unknown));

  b.insert(intern("create"), func1(&mut ta, int, unknown));
  b.insert(intern("send"), func2(&mut ta, unknown, unknown, result_unit));
  b.insert(intern("recv"), func1(&mut ta, unknown, result_unknown));
  b.insert(intern("try_recv"), func1(&mut ta, unknown, maybe_unknown));
  b.insert(intern("close"), func1(&mut ta, unknown, unit));

  ModuleSignature { bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
}

fn build_time() -> ModuleSignature {
  let mut ta = TypeArena::new();
  let mut b = HashMap::new();
  let unit = ta.unit();
  let int = ta.int();
  let str_t = ta.str();
  let unknown = ta.unknown();

  let result_unknown = ta.alloc(Type::Result { ok: unknown, err: str_t });

  b.insert(intern("now"), func1(&mut ta, unit, unknown));
  b.insert(intern("sleep"), func1(&mut ta, int, unit));
  b.insert(intern("format"), func2(&mut ta, str_t, unknown, str_t));
  b.insert(intern("parse"), func2(&mut ta, str_t, str_t, result_unknown));

  ModuleSignature { bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
}
