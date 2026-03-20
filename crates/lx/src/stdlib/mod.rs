pub(crate) mod agent;
mod agent_adapter;
mod agent_capability;
mod agent_dialogue;
mod agent_dialogue_branch;
mod agent_dialogue_persist;
mod agent_dispatch;
pub mod agent_errors;
mod agent_gate;
mod agent_handoff;
mod agent_intercept;
mod agent_ipc;
pub(crate) mod agent_lifecycle;
pub(crate) mod agent_lifecycle_run;
mod agent_mock;
mod agent_negotiate;
mod agent_negotiate_fmt;
mod agent_pipeline;
mod agent_pipeline_ctrl;
mod agent_pipeline_io;
mod agent_pubsub;
mod agent_reconcile;
mod agent_reconcile_score;
mod agent_reconcile_strat;
pub(crate) mod agent_reload;
mod agent_route;
mod agent_route_table;
pub(crate) mod agent_stream;
mod agent_supervise;
mod agents_auditor;
mod agents_grader;
mod agents_planner;
mod agents_reviewer;
mod agents_router;
mod ai;
mod ai_structured;
mod audit;
mod budget;
mod cron;
mod ctx;
pub(crate) mod deadline;
mod describe;
pub mod diag;
mod diag_walk;
mod diff;
mod diff_merge;
mod durable;
mod durable_io;
mod durable_run;
mod env;
mod flow;
mod fs;
mod git;
mod git_branch;
mod git_diff;
mod git_diff_parse;
mod git_log;
mod git_ops;
mod git_status;
mod git_worktree;
mod http;
mod introspect;
mod json;
pub mod json_conv;
mod math;
pub(crate) mod mcp;
mod md;
mod md_build;
mod pane;
mod pipeline;
mod pipeline_io;
mod plan;
mod profile;
mod profile_io;
mod profile_strategy;
mod re;
mod registry;
mod registry_query;
mod registry_store;
mod repo;
mod repo_lock;
mod repo_worktree;
pub(crate) mod retry;
mod saga;
mod step_deps;
mod store;
mod store_dispatch;

pub(crate) use store_dispatch::{
    build_constructor, object_get_field, object_insert, object_update_nested, store_clone,
    store_len, store_method,
};
mod taskgraph;
mod test;
mod time;
mod trait_ops;
mod user;
mod workspace;
mod workspace_edit;
mod ws;
mod yield_types;

use crate::interpreter::ModuleExports;

pub(crate) fn get_std_module(path: &[String]) -> Option<ModuleExports> {
    if path.len() < 2 || path[0] != "std" {
        return None;
    }
    let bindings = if path[1] == "agents" && path.len() >= 3 {
        match path[2].as_str() {
            "auditor" => agents_auditor::build(),
            "grader" => agents_grader::build(),
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
            "flow" => flow::build(),
            "re" => re::build(),
            "md" => md::build(),
            "agent" => agent::build(),
            "mcp" => mcp::build(),
            "http" => http::build(),
            "introspect" => introspect::build(),
            "time" => time::build(),
            "cron" => cron::build(),
            "deadline" => deadline::build(),
            "ai" => ai::build(),
            "audit" => audit::build(),
            "budget" => budget::build(),
            "describe" => describe::build(),
            "diff" => diff::build(),
            "diag" => diag::build(),
            "durable" => durable::build(),
            "pane" => pane::build(),
            "pipeline" => pipeline::build(),
            "plan" => plan::build(),
            "retry" => retry::build(),
            "saga" => saga::build(),
            "store" => store::build(),
            "taskgraph" => taskgraph::build(),
            "test" => test::build(),
            "profile" => profile::build(),
            "registry" => registry::build(),
            "repo" => repo::build(),
            "trait" => trait_ops::build(),
            "user" => user::build(),
            "workspace" => workspace::build(),
            "ws" => ws::build(),
            "yield" => yield_types::build(),
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
            "auditor" | "grader" | "planner" | "reviewer" | "router"
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
            | "flow"
            | "re"
            | "md"
            | "agent"
            | "mcp"
            | "http"
            | "introspect"
            | "time"
            | "cron"
            | "ai"
            | "audit"
            | "budget"
            | "deadline"
            | "describe"
            | "diff"
            | "diag"
            | "durable"
            | "pane"
            | "pipeline"
            | "plan"
            | "retry"
            | "saga"
            | "store"
            | "taskgraph"
            | "test"
            | "profile"
            | "registry"
            | "repo"
            | "trait"
            | "user"
            | "workspace"
            | "ws"
            | "yield"
    )
}
