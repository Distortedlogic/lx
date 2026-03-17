pub(crate) mod agent;
mod agent_capability;
mod agent_dialogue;
mod agent_dispatch;
mod agent_gate;
mod agent_handoff;
mod agent_intercept;
mod agent_mock;
mod agent_negotiate;
mod agent_pubsub;
mod agent_reconcile;
mod agent_reconcile_strat;
mod agent_supervise;
mod agents_auditor;
mod agents_grader;
mod agents_monitor;
mod agents_planner;
mod agents_reviewer;
mod agents_router;
mod ai;
mod ai_structured;
mod audit;
mod budget;
mod circuit;
mod context;
mod cron;
mod ctx;
mod describe;
pub mod diag;
mod diag_walk;
mod env;
mod fs;
mod git;
mod git_branch;
mod git_diff;
mod git_diff_parse;
mod git_log;
mod git_ops;
mod git_status;
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
mod pool;
mod profile;
mod profile_io;
mod profile_strategy;
mod prompt;
mod re;
mod retry;
mod saga;
mod tasks;
mod test;
mod time;
mod trace;
mod trace_progress;
mod trace_query;
mod user;

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
            "git" => git::build(),
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
            "budget" => budget::build(),
            "circuit" => circuit::build(),
            "context" => context::build(),
            "describe" => describe::build(),
            "diag" => diag::build(),
            "knowledge" => knowledge::build(),
            "memory" => memory::build(),
            "plan" => plan::build(),
            "pool" => pool::build(),
            "prompt" => prompt::build(),
            "retry" => retry::build(),
            "saga" => saga::build(),
            "test" => test::build(),
            "introspect" => introspect::build(),
            "profile" => profile::build(),
            "trace" => trace::build(),
            "user" => user::build(),
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
            | "git"
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
            | "budget"
            | "circuit"
            | "context"
            | "describe"
            | "diag"
            | "knowledge"
            | "memory"
            | "plan"
            | "pool"
            | "prompt"
            | "retry"
            | "saga"
            | "test"
            | "introspect"
            | "profile"
            | "trace"
            | "user"
    )
}
