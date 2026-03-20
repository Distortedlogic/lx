use std::sync::Arc;
use std::time::Instant;

use lx_dx::event::EventBus;
use lx_dx::langfuse::LangfuseClient;
use lx_dx::runner::ProgramRunner;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RunStatus {
    Idle,
    Running,
    Completed { duration_ms: u64 },
    Failed { error: String, duration_ms: u64 },
}

pub struct LxRunState {
    pub status: RunStatus,
    pub source_path: Option<String>,
    pub bus: Arc<EventBus>,
    pub started_at: Option<Instant>,
}

impl LxRunState {
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self {
            status: RunStatus::Idle,
            source_path: None,
            bus,
            started_at: None,
        }
    }
}

pub fn start_run(state: &mut LxRunState, source_path: String) {
    let bus = state.bus.clone();
    state.source_path = Some(source_path.clone());
    state.status = RunStatus::Running;
    state.started_at = Some(Instant::now());

    std::thread::Builder::new()
        .name("lx-desktop-run".into())
        .spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
            rt.block_on(async move {
                let langfuse = Arc::new(LangfuseClient::from_env());
                let runner = ProgramRunner::new(bus, langfuse);
                if let Err(e) = runner.run(&source_path).await {
                    eprintln!("lx run error: {e}");
                }
            });
        })
        .expect("failed to spawn lx run thread");
}
