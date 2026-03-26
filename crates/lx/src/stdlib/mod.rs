mod channel;
mod checkpoint;
mod cron;
pub mod diag;
mod env;
mod fs;
pub(crate) mod helpers;
mod http;
mod introspect;
mod math;
mod md;
mod sandbox;
#[path = "sandbox/sandbox_exec.rs"]
mod sandbox_exec;
#[path = "sandbox/sandbox_policy.rs"]
mod sandbox_policy;
#[path = "sandbox/sandbox_scope.rs"]
mod sandbox_scope;
mod schema;
mod store;
#[path = "store/store_dispatch.rs"]
mod store_dispatch;
mod stream;

pub(crate) use store_dispatch::{build_constructor, object_get_field, object_insert, object_update_nested, store_clone, store_len, store_method};
#[path = "test_mod/mod.rs"]
mod test;
mod time;
mod trait_ops;
pub(crate) mod wasm;
pub(crate) mod wasm_marshal;

use crate::interpreter::ModuleExports;

pub(crate) fn get_std_module(path: &[&str]) -> Option<ModuleExports> {
  if path.len() < 2 || path[0] != "std" {
    return None;
  }
  let bindings = match path[1] {
    "channel" => channel::build(),
    "checkpoint" => checkpoint::build(),
    "math" => math::build(),
    "fs" => fs::build(),
    "env" => env::build(),
    "http" => http::build(),
    "md" => md::build(),
    "introspect" => introspect::build(),
    "time" => time::build(),
    "cron" => cron::build(),
    "diag" => diag::build(),
    "sandbox" => sandbox::build(),
    "store" => store::build(),
    "stream" => stream::build(),
    "test" => test::build(),
    "trait" => trait_ops::build(),
    "schema" => schema::build(),
    _ => return None,
  };
  Some(ModuleExports { bindings, variant_ctors: Vec::new() })
}

pub(crate) fn lx_std_module_source(name: &str) -> Option<&'static str> {
  match name {
    "agent" => Some(include_str!("../../std/agent.lx")),
    "tool" => Some(include_str!("../../std/tool.lx")),
    "prompt" => Some(include_str!("../../std/prompt.lx")),
    "collection" => Some(include_str!("../../std/collection.lx")),
    "session" => Some(include_str!("../../std/session.lx")),
    "guard" => Some(include_str!("../../std/guard.lx")),
    "workflow" => Some(include_str!("../../std/workflow.lx")),
    "schema_trait" => Some(include_str!("../../std/schema_trait.lx")),
    "tools/bash" => Some(include_str!("../../std/tools/bash.lx")),
    "tools/read" => Some(include_str!("../../std/tools/read.lx")),
    "tools/write" => Some(include_str!("../../std/tools/write.lx")),
    "tools/edit" => Some(include_str!("../../std/tools/edit.lx")),
    "tools/glob" => Some(include_str!("../../std/tools/glob.lx")),
    "tools/grep" => Some(include_str!("../../std/tools/grep.lx")),
    "tools/web_search" => Some(include_str!("../../std/tools/web_search.lx")),
    "tools/web_fetch" => Some(include_str!("../../std/tools/web_fetch.lx")),
    _ => None,
  }
}

pub(crate) const DEFAULT_TOOL_MODULES: &[(&str, &str)] = &[
  ("tools/bash", "Bash"),
  ("tools/read", "Read"),
  ("tools/write", "Write"),
  ("tools/edit", "Edit"),
  ("tools/glob", "Glob"),
  ("tools/grep", "Grep"),
  ("tools/web_search", "WebSearch"),
  ("tools/web_fetch", "WebFetch"),
];

pub(crate) fn std_module_exists(path: &[&str]) -> bool {
  if path.len() < 2 || path[0] != "std" {
    return false;
  }
  matches!(
    path[1],
    "channel"
      | "checkpoint"
      | "math"
      | "fs"
      | "env"
      | "http"
      | "md"
      | "introspect"
      | "time"
      | "cron"
      | "diag"
      | "sandbox"
      | "store"
      | "stream"
      | "test"
      | "trait"
      | "schema"
  ) || lx_std_module_source(path[1]).is_some()
}
