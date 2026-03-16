pub(crate) mod agent;
mod agent_dialogue;
mod agent_intercept;
mod agent_reconcile;
mod agent_reconcile_strat;
mod agents_auditor;
mod agents_grader;
mod agents_monitor;
mod agents_planner;
mod agents_reviewer;
mod agents_router;
mod ai;
mod audit;
mod circuit;
mod cron;
mod ctx;
pub mod diag;
mod diag_walk;
mod env;
mod fs;
mod http;
mod introspect;
mod json;
pub mod json_conv;
mod knowledge;
mod math;
pub(crate) mod mcp;
mod md;
mod md_build;
mod memory;
mod plan;
mod re;
mod saga;
mod tasks;
mod time;
mod trace;
mod trace_progress;
mod trace_query;

use crate::interpreter::ModuleExports;

pub(crate) fn get_std_module(path: &[String]) -> Option<ModuleExports> {
    if path.len() < 2 || path[0] != "std" {
        return None;
    }
    let bindings = if path[1] == "agents" && path.len() >= 3 {
        match path[2].as_str() {
            "auditor" => agents_auditor::build(),
            "grader" => agents_grader::build(),
            "monitor" => agents_monitor::build(),
            "planner" => agents_planner::build(),
            "reviewer" => agents_reviewer::build(),
            "router" => agents_router::build(),
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
            "diag" => diag::build(),
            "knowledge" => knowledge::build(),
            "memory" => memory::build(),
            "plan" => plan::build(),
            "saga" => saga::build(),
            "introspect" => introspect::build(),
            "trace" => trace::build(),
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
        return matches!(
            path[2].as_str(),
            "auditor" | "grader" | "monitor" | "planner" | "reviewer" | "router"
        );
    }
    matches!(
        path[1].as_str(),
        "json"
            | "ctx"
            | "math"
            | "fs"
            | "env"
            | "re"
            | "md"
            | "agent"
            | "mcp"
            | "http"
            | "time"
            | "cron"
            | "ai"
            | "tasks"
            | "audit"
            | "circuit"
            | "diag"
            | "knowledge"
            | "memory"
            | "plan"
            | "saga"
            | "introspect"
            | "trace"
    )
}
