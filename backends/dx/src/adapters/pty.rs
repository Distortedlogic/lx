use std::io::{self, Write};
use std::sync::Mutex;

use crate::adapters::ansi;
use crate::event::{EventBus, RuntimeEvent};

pub struct PtyWriter<W: Write + Send + 'static> {
    writer: Mutex<W>,
}

impl<W: Write + Send + 'static> PtyWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: Mutex::new(writer),
        }
    }

    pub fn write_event(&self, event: &RuntimeEvent) -> io::Result<()> {
        let formatted = ansi::format_event(event);
        let mut w = self
            .writer
            .lock()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("lock: {e}")))?;
        let is_progress = matches!(event, RuntimeEvent::Progress { .. });
        if is_progress {
            write!(w, "{formatted}")?;
        } else {
            writeln!(w, "{formatted}")?;
        }
        w.flush()
    }
}

pub fn spawn_pty_writer<W: Write + Send + 'static>(
    bus: &EventBus,
    agent_id: String,
    writer: W,
) -> tokio::task::JoinHandle<()> {
    let mut rx = bus.subscribe();
    let pty = PtyWriter::new(writer);
    tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let is_kill = matches!(
                        &event,
                        RuntimeEvent::AgentKilled { agent_id: id, .. } if id == &agent_id
                    );
                    let matches = event
                        .agent_id()
                        .map_or(false, |id| id == agent_id);
                    if matches {
                        if let Err(e) = pty.write_event(&event) {
                            eprintln!("pty writer error for {agent_id}: {e}");
                            break;
                        }
                    }
                    if is_kill {
                        break;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("pty writer for {agent_id}: lagged {n} events");
                }
            }
        }
    })
}
