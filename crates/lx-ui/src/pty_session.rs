use std::io::{Read, Write};
use std::sync::{Arc, LazyLock, Mutex};

use dashmap::DashMap;
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use tokio::sync::{broadcast, mpsc};

const BUFFER_CAP: usize = 256 * 1024;

pub struct PtySession {
    input_tx: mpsc::Sender<Vec<u8>>,
    output_tx: broadcast::Sender<Vec<u8>>,
    buffer: Arc<Mutex<Vec<u8>>>,
    master: parking_lot::Mutex<Box<dyn MasterPty + Send>>,
    _child: Box<dyn portable_pty::Child + Send + Sync>,
}

static SESSIONS: LazyLock<DashMap<String, Arc<PtySession>>> = LazyLock::new(DashMap::new);

pub fn get_or_create(
    id: &str,
    cols: u16,
    rows: u16,
    working_dir: Option<&str>,
    command: Option<&str>,
) -> Result<Arc<PtySession>, String> {
    if let Some(session) = SESSIONS.get(id) {
        return Ok(Arc::clone(session.value()));
    }
    let session = Arc::new(PtySession::spawn(cols, rows, working_dir, command)?);
    SESSIONS.insert(id.to_owned(), Arc::clone(&session));
    Ok(session)
}

pub fn remove(id: &str) {
    SESSIONS.remove(id);
}

impl PtySession {
    fn spawn(
        cols: u16,
        rows: u16,
        working_dir: Option<&str>,
        command: Option<&str>,
    ) -> Result<Self, String> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;

        let mut cmd_builder = match command {
            Some(cmd) => CommandBuilder::new(cmd),
            None => CommandBuilder::new_default_prog(),
        };
        if let Some(dir) = working_dir {
            cmd_builder.cwd(dir);
        }

        let child = pair
            .slave
            .spawn_command(cmd_builder)
            .map_err(|e| e.to_string())?;
        let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
        let writer = pair.master.take_writer().map_err(|e| e.to_string())?;

        let (in_tx, in_rx) = mpsc::channel::<Vec<u8>>(256);
        let (out_tx, _) = broadcast::channel::<Vec<u8>>(256);
        let buffer: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));

        let out_tx_clone = out_tx.clone();
        let buffer_clone = Arc::clone(&buffer);
        std::thread::Builder::new()
            .name("pty-reader".into())
            .spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => {
                            let data = buf[..n].to_vec();
                            let mut b = buffer_clone.lock().expect("buffer lock poisoned");
                            b.extend_from_slice(&data);
                            if b.len() > BUFFER_CAP {
                                let excess = b.len() - BUFFER_CAP;
                                b.drain(..excess);
                            }
                            let _ = out_tx_clone.send(data);
                            drop(b);
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;

        std::thread::Builder::new()
            .name("pty-writer".into())
            .spawn(move || {
                let mut writer = writer;
                let mut in_rx = in_rx;
                while let Some(data) = in_rx.blocking_recv() {
                    if writer.write_all(&data).is_err() {
                        break;
                    }
                }
            })
            .map_err(|e| e.to_string())?;

        Ok(Self {
            input_tx: in_tx,
            output_tx: out_tx,
            buffer,
            master: parking_lot::Mutex::new(pair.master),
            _child: child,
        })
    }

    pub fn subscribe(&self) -> (Vec<u8>, broadcast::Receiver<Vec<u8>>) {
        let buf = self.buffer.lock().expect("buffer lock poisoned");
        let rx = self.output_tx.subscribe();
        let snapshot = buf.clone();
        (snapshot, rx)
    }

    pub async fn send_input(&self, data: Vec<u8>) -> Result<(), String> {
        self.input_tx.send(data).await.map_err(|e| e.to_string())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.master
            .lock()
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())
    }
}
