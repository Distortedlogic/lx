pub mod ai;
pub mod emit;
pub mod log;
pub mod shell;
pub mod user;
pub mod yield_;

use std::sync::Arc;

use lx::backends::{ClaudeCodeAiBackend, ProcessShellBackend, ReqwestHttpBackend, RuntimeCtx};

use crate::event::EventBus;
use crate::langfuse::LangfuseClient;

use ai::DxAiBackend;
use emit::DxEmitBackend;
use log::LangfuseLogBackend;
use shell::DxShellBackend;
use user::DxUserBackend;
use yield_::DxYieldBackend;

pub fn build_runtime_ctx(
    bus: Arc<EventBus>,
    langfuse: Arc<LangfuseClient>,
    agent_id: String,
) -> Arc<RuntimeCtx> {
    Arc::new(RuntimeCtx {
        ai: Arc::new(DxAiBackend {
            inner: Box::new(ClaudeCodeAiBackend),
            bus: bus.clone(),
            langfuse: langfuse.clone(),
            agent_id: agent_id.clone(),
        }),
        emit: Arc::new(DxEmitBackend {
            bus: bus.clone(),
            agent_id: agent_id.clone(),
        }),
        http: Arc::new(ReqwestHttpBackend),
        shell: Arc::new(DxShellBackend {
            inner: ProcessShellBackend,
            bus: bus.clone(),
            agent_id: agent_id.clone(),
        }),
        yield_: Arc::new(DxYieldBackend::new(bus.clone(), agent_id.clone())),
        log: Arc::new(LangfuseLogBackend {
            bus: bus.clone(),
            langfuse,
            agent_id: agent_id.clone(),
        }),
        user: Arc::new(DxUserBackend::new(bus, agent_id)),
    })
}
