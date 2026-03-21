mod cron;
pub mod diag;
#[path = "diag/diag_walk.rs"]
mod diag_walk;
mod env;
mod fs;
mod http;
mod introspect;
mod math;
mod md;
mod re;
mod sandbox;
#[path = "sandbox/sandbox_exec.rs"]
mod sandbox_exec;
#[path = "sandbox/sandbox_policy.rs"]
mod sandbox_policy;
#[path = "sandbox/sandbox_scope.rs"]
mod sandbox_scope;
mod store;
#[path = "store/store_dispatch.rs"]
mod store_dispatch;

pub(crate) use store_dispatch::{
    build_constructor, object_get_field, object_insert, object_update_nested, store_clone,
    store_len, store_method,
};
#[path = "test_mod/mod.rs"]
mod test;
mod time;
mod trait_ops;
mod user;
mod ws;
mod yield_types;

use crate::interpreter::ModuleExports;

pub(crate) fn get_std_module(path: &[String]) -> Option<ModuleExports> {
    if path.len() < 2 || path[0] != "std" {
        return None;
    }
    let bindings = match path[1].as_str() {
        "math" => math::build(),
        "fs" => fs::build(),
        "env" => env::build(),
        "re" => re::build(),
        "md" => md::build(),
        "http" => http::build(),
        "introspect" => introspect::build(),
        "time" => time::build(),
        "cron" => cron::build(),
        "diag" => diag::build(),
        "sandbox" => sandbox::build(),
        "store" => store::build(),
        "test" => test::build(),
        "trait" => trait_ops::build(),
        "user" => user::build(),
        "ws" => ws::build(),
        "yield" => yield_types::build(),
        _ => return None,
    };
    Some(ModuleExports {
        bindings,
        variant_ctors: Vec::new(),
    })
}

pub(crate) fn std_module_exists(path: &[String]) -> bool {
    if path.len() < 2 || path[0] != "std" {
        return false;
    }
    matches!(
        path[1].as_str(),
        "math"
            | "fs"
            | "env"
            | "re"
            | "md"
            | "http"
            | "introspect"
            | "time"
            | "cron"
            | "diag"
            | "sandbox"
            | "store"
            | "test"
            | "trait"
            | "user"
            | "ws"
            | "yield"
    )
}
