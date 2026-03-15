pub(crate) mod agent;
mod agents_auditor;
mod ai;
mod audit;
mod circuit;
mod cron;
mod ctx;
mod env;
mod fs;
mod http;
mod introspect;
mod json;
mod knowledge;
pub mod json_conv;
mod math;
pub(crate) mod mcp;
mod md;
mod plan;
mod md_build;
mod re;
mod tasks;
mod time;

use crate::interpreter::ModuleExports;

pub(crate) fn get_std_module(path: &[String]) -> Option<ModuleExports> {
    if path.len() < 2 || path[0] != "std" {
        return None;
    }
    let bindings = if path[1] == "agents" && path.len() >= 3 {
        match path[2].as_str() {
            "auditor" => agents_auditor::build(),
            _ => return None,
        }
    } else {
        match path[1].as_str() {
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
            "ai" => ai::build(),
            "tasks" => tasks::build(),
            "audit" => audit::build(),
            "circuit" => circuit::build(),
            "knowledge" => knowledge::build(),
            "plan" => plan::build(),
            "introspect" => introspect::build(),
            _ => return None,
        }
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
    if path[1] == "agents" && path.len() >= 3 {
        return matches!(path[2].as_str(), "auditor");
    }
    matches!(path[1].as_str(), "json" | "ctx" | "math" | "fs" | "env" | "re" | "md" | "agent" | "mcp" | "http" | "time" | "cron" | "ai" | "tasks" | "audit" | "circuit" | "knowledge" | "plan" | "introspect")
}
