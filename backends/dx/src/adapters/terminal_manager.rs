use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use tokio::task::JoinHandle;

use crate::adapters::pty::{PtyWriter, spawn_pty_writer};
use crate::event::{EventBus, RuntimeEvent};

pub type WriterFactory = Box<dyn Fn(&str, &str) -> Box<dyn Write + Send> + Send + Sync>;

pub struct AgentTerminalManager {
    bus: Arc<EventBus>,
    factory: WriterFactory,
}

impl AgentTerminalManager {
    pub fn new(bus: Arc<EventBus>, factory: WriterFactory) -> Self {
        Self { bus, factory }
    }

    pub fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            let main_writer = (self.factory)("main", "main");
            let main_pty = Arc::new(PtyWriter::new(main_writer));
            let mut agent_handles: HashMap<String, JoinHandle<()>> = HashMap::new();
            let mut rx = self.bus.subscribe();

            loop {
                match rx.recv().await {
                    Ok(event) => match &event {
                        RuntimeEvent::AgentSpawned { agent_id, name, .. } => {
                            let writer = (self.factory)(agent_id, name);
                            let handle = spawn_pty_writer(&self.bus, agent_id.clone(), writer);
                            agent_handles.insert(agent_id.clone(), handle);
                        }
                        RuntimeEvent::AgentKilled { agent_id, .. } => {
                            if let Some(handle) = agent_handles.remove(agent_id) {
                                handle.abort();
                            }
                        }
                        RuntimeEvent::ProgramStarted { .. }
                        | RuntimeEvent::ProgramFinished { .. } => {
                            if let Err(e) = main_pty.write_event(&event) {
                                eprintln!("main pty error: {e}");
                            }
                        }
                        other => {
                            if other.agent_id().map_or(false, |id| id == "main") {
                                if let Err(e) = main_pty.write_event(&event) {
                                    eprintln!("main pty error: {e}");
                                }
                            }
                        }
                    },
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("terminal manager: lagged {n} events");
                    }
                }
            }
        })
    }
}
