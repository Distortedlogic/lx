use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use lx_dx::adapters::terminal_manager::{AgentTerminalManager, WriterFactory};
use lx_dx::event::EventBus;
use lx_dx::langfuse::LangfuseClient;
use lx_dx::runner::ProgramRunner;
use lx_ui::pane_tree::PaneNode;
use lx_ui::pty_session;
use serde::{Deserialize, Serialize};

use crate::layout::shell::SpawnSender;

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
    Self { status: RunStatus::Idle, source_path: None, bus, started_at: None }
  }
}

struct PtySessionWriter {
  session: Arc<pty_session::PtySession>,
  rt: tokio::runtime::Handle,
}

impl Write for PtySessionWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.rt.block_on(self.session.send_input(buf.to_vec())).map_err(io::Error::other)?;
    Ok(buf.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

fn make_writer_factory(run_id: String, source_dir: String, spawn_tx: Option<SpawnSender>) -> WriterFactory {
  let rt = tokio::runtime::Handle::current();
  Box::new(move |agent_id: &str, agent_name: &str| -> Box<dyn Write + Send> {
    let terminal_id = format!("lx-{run_id}-{agent_id}");

    if let Some(ref tx) = spawn_tx {
      let pane = PaneNode::Terminal { id: terminal_id.clone(), working_dir: source_dir.clone(), command: None };
      let title = format!("lx: {agent_name}");
      let req = crate::layout::shell::TerminalSpawnRequest { id: terminal_id.clone(), title, pane };
      let _ = tx.send(req);
    }

    match pty_session::get_or_create(&terminal_id, 120, 40, Some(&source_dir), None) {
      Ok(sess) => Box::new(PtySessionWriter { session: sess, rt: rt.clone() }),
      Err(e) => {
        eprintln!("failed to create PTY session for lx agent {agent_id}: {e}");
        Box::new(io::sink())
      },
    }
  })
}

pub fn start_run(shared: Arc<Mutex<LxRunState>>, source_path: String, spawn_tx: Option<SpawnSender>) {
  {
    let mut state = shared.lock().expect("lock poisoned");
    state.source_path = Some(source_path.clone());
    state.status = RunStatus::Running;
    state.started_at = Some(Instant::now());
  }

  let bus = shared.lock().expect("lock poisoned").bus.clone();
  let shared_clone = Arc::clone(&shared);
  let source_dir = Path::new(&source_path).parent().map_or_else(|| ".".to_owned(), |p| p.display().to_string());

  let run_id = uuid::Uuid::new_v4().to_string();

  std::thread::Builder::new()
    .name(format!("lx-run-{run_id}"))
    .spawn(move || {
      let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
      rt.block_on(async move {
        let factory = make_writer_factory(run_id, source_dir, spawn_tx);
        let terminal_manager = AgentTerminalManager::new(bus.clone(), factory);
        terminal_manager.start();

        let langfuse = Arc::new(LangfuseClient::from_env());
        let runner = ProgramRunner::new(bus, langfuse);
        let result = runner.run(&source_path).await;
        let mut state = shared_clone.lock().expect("lock poisoned");
        let duration_ms = state.started_at.map(|s| s.elapsed().as_millis() as u64).unwrap_or(0);
        match result {
          Ok(_) => {
            state.status = RunStatus::Completed { duration_ms };
          },
          Err(e) => {
            state.status = RunStatus::Failed { error: e.to_string(), duration_ms };
          },
        }
      });
    })
    .expect("failed to spawn lx run thread");
}
