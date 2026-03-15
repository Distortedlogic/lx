pub(crate) mod agent;
mod cron;
mod ctx;
mod env;
mod fs;
mod http;
mod json;
pub mod json_conv;
mod math;
pub(crate) mod mcp;
mod md;
mod md_build;
mod re;
mod time;

use crate::interpreter::ModuleExports;

pub(crate) fn get_std_module(path: &[String]) -> Option<ModuleExports> {
    if path.len() < 2 || path[0] != "std" {
        return None;
    }
    let bindings = match path[1].as_str() {
        "json" => json::build(),
        "ctx" => ctx::build(),
        "math" => math::build(),
        "fs" => fs::build(),
        "env" => env::build(),
        "re" => re::build(),
        "md" => md::build(),
        "agent" => agent::build(),
        "mcp" => mcp::build(),
        "http" => http::build(),
        "time" => time::build(),
        "cron" => cron::build(),
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
    matches!(path[1].as_str(), "json" | "ctx" | "math" | "fs" | "env" | "re" | "md" | "agent" | "mcp" | "http" | "time" | "cron")
}
