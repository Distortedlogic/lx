pub mod ai;
pub mod emit;
pub mod log;
pub mod shell;
pub mod user;
pub mod yield_;

use std::sync::Arc;

use std::time::Instant;

use lx::backends::{AgentEvent, ClaudeCodeAiBackend, ProcessShellBackend, ReqwestHttpBackend, RuntimeCtx};

use crate::event::{EventBus, RuntimeEvent};
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
        user: Arc::new(DxUserBackend::new(bus.clone(), agent_id)),
        source_dir: parking_lot::Mutex::new(None),
        on_agent_event: Some(Arc::new(move |event: AgentEvent| {
            match event {
                AgentEvent::Spawned { id, name } => {
                    bus.send(RuntimeEvent::AgentSpawned {
                        agent_id: id,
                        name,
                        config: serde_json::Value::Null,
                        ts: Instant::now(),
                    });
                }
                AgentEvent::Killed { id } => {
                    bus.send(RuntimeEvent::AgentKilled {
                        agent_id: id,
                        ts: Instant::now(),
                    });
                }
            }
        })),
    })
}
