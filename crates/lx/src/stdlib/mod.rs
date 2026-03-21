pub(crate) mod agent;
#[path = "agent/agent_dialogue.rs"]
mod agent_dialogue;
#[path = "agent/agent_dialogue_branch.rs"]
mod agent_dialogue_branch;
#[path = "agent/agent_errors.rs"]
pub mod agent_errors;
#[path = "agent/agent_gate.rs"]
mod agent_gate;
#[path = "agent/agent_ipc.rs"]
mod agent_ipc;
#[path = "agent/agent_lifecycle.rs"]
pub(crate) mod agent_lifecycle;
#[path = "agent/agent_lifecycle_run.rs"]
pub(crate) mod agent_lifecycle_run;
#[path = "agent/agent_pipeline.rs"]
mod agent_pipeline;
#[path = "agent/agent_pipeline_ctrl.rs"]
mod agent_pipeline_ctrl;
#[path = "agent/agent_pipeline_io.rs"]
mod agent_pipeline_io;
#[path = "agent/agent_pubsub.rs"]
mod agent_pubsub;
#[path = "agent/agent_reload.rs"]
pub(crate) mod agent_reload;
#[path = "agent/agent_route.rs"]
mod agent_route;
#[path = "agent/agent_route_table.rs"]
mod agent_route_table;
#[path = "agent/agent_stream.rs"]
pub(crate) mod agent_stream;
#[path = "agent/agent_supervise.rs"]
mod agent_supervise;
#[path = "ai_mod/mod.rs"]
mod ai;
#[path = "ai_mod/ai_structured.rs"]
mod ai_structured;
mod cron;
pub(crate) mod deadline;
mod describe;
pub mod diag;
#[path = "diag/diag_walk.rs"]
mod diag_walk;
mod diff;
#[path = "diff/diff_merge.rs"]
mod diff_merge;
mod env;
mod flow;
mod fs;
mod http;
mod introspect;
mod json;
pub mod json_conv;
mod math;
pub(crate) mod mcp;
mod md;
mod pane;
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
        "json" => json::build(),
        "math" => math::build(),
        "fs" => fs::build(),
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
        "describe" => describe::build(),
        "diff" => diff::build(),
        "diag" => diag::build(),
        "pane" => pane::build(),
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
        "json"
            | "math"
            | "fs"
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
            | "deadline"
            | "describe"
            | "diff"
            | "diag"
            | "pane"
            | "sandbox"
            | "store"
            | "test"
            | "trait"
            | "user"
            | "ws"
            | "yield"
    )
}
