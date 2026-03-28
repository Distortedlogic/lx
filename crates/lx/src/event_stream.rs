use std::io::Write;
use std::sync::Arc;

use indexmap::IndexMap;
use parking_lot::{Mutex, RwLock};
use serde::ser::SerializeMap;

use crate::mcp_client::McpClient;
use crate::sym::{Sym, intern};
use crate::value::LxVal;

pub struct EventStream {
  entries: RwLock<Vec<StreamEntry>>,
  last_ms: Mutex<(u64, u64)>,
  notify: tokio::sync::Notify,
  jsonl_writer: Mutex<Option<std::io::BufWriter<std::fs::File>>>,
  external_client: Mutex<Option<Arc<tokio::sync::Mutex<McpClient>>>>,
}

#[derive(Debug, Clone)]
pub struct StreamEntry {
  pub id: String,
  pub kind: String,
  pub agent: String,
  pub ts: u64,
  pub span: Option<SpanInfo>,
  pub fields: IndexMap<Sym, LxVal>,
}

impl serde::Serialize for StreamEntry {
  fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let field_count = 5 + self.fields.len();
    let mut map = serializer.serialize_map(Some(field_count))?;
    map.serialize_entry("id", &self.id)?;
    map.serialize_entry("kind", &self.kind)?;
    map.serialize_entry("agent", &self.agent)?;
    map.serialize_entry("ts", &self.ts)?;
    map.serialize_entry("span", &self.span)?;
    for (k, v) in &self.fields {
      map.serialize_entry(k.as_str(), v)?;
    }
    map.end()
  }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpanInfo {
  pub line: usize,
  pub col: usize,
}

impl EventStream {
  pub fn new(jsonl_path: Option<std::path::PathBuf>) -> Self {
    let writer = jsonl_path.and_then(|p| {
      if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
      }
      let file = std::fs::OpenOptions::new().create(true).append(true).open(p).ok()?;
      Some(std::io::BufWriter::new(file))
    });
    Self {
      entries: RwLock::new(Vec::new()),
      last_ms: Mutex::new((0, 0)),
      notify: tokio::sync::Notify::new(),
      jsonl_writer: Mutex::new(writer),
      external_client: Mutex::new(None),
    }
  }

  pub fn has_jsonl(&self) -> bool {
    self.jsonl_writer.lock().is_some()
  }

  pub fn enable_jsonl(&self, path: std::path::PathBuf) {
    if let Some(parent) = path.parent() {
      let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(file) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
      *self.jsonl_writer.lock() = Some(std::io::BufWriter::new(file));
    }
  }

  pub fn set_external_client(&self, client: Arc<tokio::sync::Mutex<McpClient>>) {
    *self.external_client.lock() = Some(client);
  }

  pub fn xadd(&self, kind: &str, agent: &str, span: Option<SpanInfo>, fields: IndexMap<Sym, LxVal>) -> String {
    let ms = chrono::Utc::now().timestamp_millis() as u64;
    let mut id_state = self.last_ms.lock();
    let seq = if ms == id_state.0 {
      id_state.1 += 1;
      id_state.1
    } else {
      *id_state = (ms, 0);
      0
    };
    let id = format!("{ms}-{seq}");
    let entry = StreamEntry { id: id.clone(), kind: kind.to_string(), agent: agent.to_string(), ts: ms, span, fields };
    self.entries.write().push(entry.clone());
    let mut guard = self.jsonl_writer.lock();
    if let Some(ref mut w) = *guard {
      match serde_json::to_string(&entry) {
        Ok(json) => {
          if let Err(e) = writeln!(w, "{json}") {
            eprintln!("event_stream: JSONL write failed: {e}");
          }
          if let Err(e) = w.flush() {
            eprintln!("event_stream: JSONL flush failed: {e}");
          }
        },
        Err(e) => {
          eprintln!("event_stream: JSONL serialization failed: {e}");
        },
      }
    }
    self.notify.notify_waiters();

    if let Some(client) = self.external_client.lock().clone() {
      let entry_json = serde_json::to_value(&entry).unwrap_or(serde_json::Value::Null);
      tokio::task::spawn(async move {
        if let Err(e) = client.lock().await.tools_call("xadd", entry_json).await {
          eprintln!("[stream:external] xadd failed: {e}");
        }
      });
    }

    id
  }

  pub fn xrange(&self, start: &str, end: &str, count: Option<usize>) -> Vec<StreamEntry> {
    let entries = self.entries.read();
    let iter = entries.iter().filter(|e| {
      let after_start = if start == "-" { true } else { id_ge(&e.id, start) };
      let before_end = if end == "+" { true } else { id_ge(end, &e.id) };
      after_start && before_end
    });
    match count {
      Some(n) => iter.take(n).cloned().collect(),
      None => iter.cloned().collect(),
    }
  }

  pub async fn xread(&self, last_id: &str, timeout_ms: Option<u64>) -> Option<StreamEntry> {
    let effective_last = if last_id == "$" {
      let entries = self.entries.read();
      entries.last().map(|e| e.id.clone()).unwrap_or_default()
    } else {
      last_id.to_string()
    };

    let deadline = timeout_ms.map(|ms| tokio::time::Instant::now() + std::time::Duration::from_millis(ms));

    loop {
      let notified = self.notify.notified();

      {
        let entries = self.entries.read();
        if let Some(entry) = entries.iter().find(|e| id_gt(&e.id, &effective_last)) {
          return Some(entry.clone());
        }
      }

      match deadline {
        Some(dl) => {
          tokio::select! {
              _ = notified => {},
              _ = tokio::time::sleep_until(dl) => {
                  let entries = self.entries.read();
                  return entries
                      .iter()
                      .find(|e| id_gt(&e.id, &effective_last))
                      .cloned();
              }
          }
        },
        None => {
          notified.await;
        },
      }
    }
  }

  pub async fn shutdown_external(&self) {
    let client = self.external_client.lock().take();
    if let Some(client) = client {
      client.lock().await.shutdown().await;
    }
  }

  pub fn xlen(&self) -> usize {
    self.entries.read().len()
  }

  pub fn xtrim(&self, maxlen: usize) {
    let mut entries = self.entries.write();
    if entries.len() > maxlen {
      let remove = entries.len() - maxlen;
      entries.drain(..remove);
    }
  }
}

pub fn entry_to_lxval(entry: &StreamEntry) -> LxVal {
  let mut map = IndexMap::new();
  map.insert(intern("id"), LxVal::str(&entry.id));
  map.insert(intern("kind"), LxVal::str(&entry.kind));
  map.insert(intern("agent"), LxVal::str(&entry.agent));
  map.insert(intern("ts"), LxVal::int(entry.ts as i64));
  map.insert(
    intern("span"),
    match &entry.span {
      Some(s) => {
        let mut span_map = IndexMap::new();
        span_map.insert(intern("line"), LxVal::int(s.line as i64));
        span_map.insert(intern("col"), LxVal::int(s.col as i64));
        LxVal::record(span_map)
      },
      None => LxVal::None,
    },
  );
  for (k, v) in &entry.fields {
    map.insert(*k, v.clone());
  }
  LxVal::record(map)
}

fn parse_id(id: &str) -> Option<(u64, u64)> {
  let (ms_str, seq_str) = id.split_once('-')?;
  let ms = ms_str.parse::<u64>().ok()?;
  let seq = seq_str.parse::<u64>().ok()?;
  Some((ms, seq))
}

fn id_ge(a: &str, b: &str) -> bool {
  match (parse_id(a), parse_id(b)) {
    (Some((ms_a, seq_a)), Some((ms_b, seq_b))) => (ms_a, seq_a) >= (ms_b, seq_b),
    _ => a >= b,
  }
}

fn id_gt(a: &str, b: &str) -> bool {
  match (parse_id(a), parse_id(b)) {
    (Some((ms_a, seq_a)), Some((ms_b, seq_b))) => (ms_a, seq_a) > (ms_b, seq_b),
    _ => a > b,
  }
}
