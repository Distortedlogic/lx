# Unit 4: Event Stream Core + JSONL Backend

## Goal

Implement the event stream subsystem: a persistent, append-only log with Redis Streams-style API (xadd/xrange/xread/xlen/xtrim). Includes the StreamBackend trait and the JSONL file backend.

This unit does NOT wire the stream into the interpreter — that happens in Unit 5. This unit provides the Rust types and the JSONL implementation.

## Preconditions

- `serde_json` is a workspace dependency
- `tokio` is a workspace dependency with `sync`, `time` features
- No existing event stream code in the codebase
- The existing `LxVal::Stream { id: u64 }` variant (value/mod.rs:112-114) is for the lazy data stream (`std/stream`), NOT for the event stream

## Step 1: Create module structure

Create: `crates/lx/src/event_stream/`

Files:
- `crates/lx/src/event_stream/mod.rs` — trait definition, re-exports
- `crates/lx/src/event_stream/entry.rs` — stream entry types and ID generation
- `crates/lx/src/event_stream/jsonl.rs` — JSONL file backend

Register in `crates/lx/src/lib.rs`: add `pub mod event_stream;`

## Step 2: Stream entry and ID types

File: `crates/lx/src/event_stream/entry.rs`

### Stream ID

Format: `{unix_ms}-{seq}` matching Valkey/Redis stream ID format.

```rust
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StreamId {
  pub ms: u64,
  pub seq: u64,
}

impl StreamId {
  pub fn min() -> Self { Self { ms: 0, seq: 0 } }
  pub fn max() -> Self { Self { ms: u64::MAX, seq: u64::MAX } }
}

impl fmt::Display for StreamId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}-{}", self.ms, self.seq)
  }
}

impl std::str::FromStr for StreamId {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, String> {
    let (ms_str, seq_str) = s.split_once('-')
      .ok_or_else(|| format!("invalid stream ID: {s}"))?;
    Ok(Self {
      ms: ms_str.parse().map_err(|_| format!("invalid ms in ID: {s}"))?,
      seq: seq_str.parse().map_err(|_| format!("invalid seq in ID: {s}"))?,
    })
  }
}
```

### ID Generator

Uses a `Mutex` for correctness under concurrent agent writes (the architecture doc requires distinct sequential IDs for concurrent xadds in the same millisecond):

```rust
use std::sync::Mutex;

pub struct IdGenerator {
  state: Mutex<(u64, u64)>, // (last_ms, seq)
}

impl IdGenerator {
  pub fn new() -> Self {
    Self { state: Mutex::new((0, 0)) }
  }

  pub fn next(&self) -> StreamId {
    let now_ms = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock")
      .as_millis() as u64;

    let mut state = self.state.lock().expect("id gen lock");
    if now_ms == state.0 {
      state.1 += 1;
      StreamId { ms: now_ms, seq: state.1 }
    } else {
      *state = (now_ms, 0);
      StreamId { ms: now_ms, seq: 0 }
    }
  }
}
```

### Stream Entry

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEntry {
  pub id: String,
  pub kind: String,
  pub agent: String,
  pub ts: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub span: Option<SpanInfo>,
  #[serde(flatten)]
  pub fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
  pub line: u32,
  pub col: u32,
}
```

The `#[serde(flatten)]` on `fields` allows kind-specific fields to be serialized alongside the fixed fields without nesting.

## Step 3: StreamBackend trait

File: `crates/lx/src/event_stream/mod.rs`

```rust
mod entry;
mod jsonl;

pub use entry::{IdGenerator, SpanInfo, StreamEntry, StreamId};
pub use jsonl::JsonlBackend;

pub trait StreamBackend: Send + Sync {
  fn xadd(&self, entry: StreamEntry) -> Result<String, String>;

  fn xrange(&self, start: &str, end: &str, count: Option<usize>) -> Result<Vec<StreamEntry>, String>;

  fn xread(&self, last_id: &str, timeout_ms: Option<u64>) -> Result<Option<StreamEntry>, String>;

  fn xlen(&self) -> Result<usize, String>;

  fn xtrim(&self, maxlen: usize) -> Result<usize, String>;

  fn load_all(&self) -> Result<Vec<StreamEntry>, String>;
}
```

The `load_all` method is for the resume system (Unit 6) — it reads the entire stream on startup.

## Step 4: JSONL Backend

File: `crates/lx/src/event_stream/jsonl.rs`

The JSONL backend appends one JSON line per entry to a file. Reading uses line-by-line parsing.

```rust
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Mutex;

use super::entry::{IdGenerator, StreamEntry, StreamId};
use super::StreamBackend;

pub struct JsonlBackend {
  path: PathBuf,
  writer: Mutex<File>,
  id_gen: IdGenerator,
}

impl JsonlBackend {
  pub fn new(path: PathBuf) -> Result<Self, String> {
    if let Some(parent) = path.parent() {
      fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
    }
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&path)
      .map_err(|e| format!("open {}: {e}", path.display()))?;
    Ok(Self {
      path,
      writer: Mutex::new(file),
      id_gen: IdGenerator::new(),
    })
  }
}
```

### xadd

```rust
impl StreamBackend for JsonlBackend {
  fn xadd(&self, mut entry: StreamEntry) -> Result<String, String> {
    let id = self.id_gen.next();
    let id_str = id.to_string();
    entry.id = id_str.clone();
    entry.ts = id.ms;

    let line = serde_json::to_string(&entry).map_err(|e| format!("serialize: {e}"))?;
    let mut writer = self.writer.lock().map_err(|_| "writer lock poisoned")?;
    writeln!(writer, "{line}").map_err(|e| format!("write: {e}"))?;
    writer.flush().map_err(|e| format!("flush: {e}"))?;

    Ok(id_str)
  }
```

### xrange

```rust
  fn xrange(&self, start: &str, end: &str, count: Option<usize>) -> Result<Vec<StreamEntry>, String> {
    let start_id = if start == "-" { StreamId::min() } else { start.parse()? };
    let end_id = if end == "+" { StreamId::max() } else { end.parse()? };

    let file = File::open(&self.path).map_err(|e| format!("open: {e}"))?;
    let reader = BufReader::new(file);
    let mut results = Vec::new();

    for line in reader.lines() {
      let line = line.map_err(|e| format!("read: {e}"))?;
      if line.trim().is_empty() { continue; }
      let entry: StreamEntry = serde_json::from_str(&line)
        .map_err(|e| format!("parse entry: {e}"))?;
      let entry_id: StreamId = entry.id.parse()?;
      if entry_id >= start_id && entry_id <= end_id {
        results.push(entry);
        if let Some(max) = count {
          if results.len() >= max { break; }
        }
      }
    }
    Ok(results)
  }
```

### xread

```rust
  fn xread(&self, last_id: &str, timeout_ms: Option<u64>) -> Result<Option<StreamEntry>, String> {
    let after_id = if last_id == "$" {
      // "$" means "from now" — get current last ID
      let entries = self.xrange("-", "+", None)?;
      entries.last().map(|e| e.id.clone()).unwrap_or_else(|| "0-0".to_string())
    } else {
      last_id.to_string()
    };
    let after: StreamId = after_id.parse()?;

    // Poll: check if a new entry exists after the given ID
    // For a proper implementation, use the notify + timeout
    let start = std::time::Instant::now();
    let timeout = timeout_ms.map(std::time::Duration::from_millis);

    loop {
      let entries = self.xrange(&after.to_string(), "+", Some(2))?;
      // Find the first entry strictly after after_id
      for entry in &entries {
        let eid: StreamId = entry.id.parse()?;
        if eid > after {
          return Ok(Some(entry.clone()));
        }
      }
      match timeout {
        Some(dur) if start.elapsed() >= dur => return Ok(None),
        None => {
          // Block indefinitely — wait on notify
          // This is sync context, so use std thread parking
          std::thread::sleep(std::time::Duration::from_millis(50));
        },
        Some(_) => {
          std::thread::sleep(std::time::Duration::from_millis(50));
        },
      }
    }
  }
```

Note: The xread polling approach uses 50ms sleep intervals. This is simple and acceptable for lx's use case (workflows, not high-frequency trading). A more sophisticated approach could use inotify/kqueue for file change notification, but that adds platform-specific complexity.

### xlen, xtrim, load_all

```rust
  fn xlen(&self) -> Result<usize, String> {
    let file = File::open(&self.path).map_err(|e| format!("open: {e}"))?;
    let reader = BufReader::new(file);
    Ok(reader.lines().filter(|l| l.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false)).count())
  }

  fn xtrim(&self, maxlen: usize) -> Result<usize, String> {
    // Hold writer lock for the entire operation to prevent concurrent xadd
    // from writing to the old file descriptor during the rewrite
    let mut writer = self.writer.lock().map_err(|_| "writer lock poisoned")?;

    let entries = self.xrange("-", "+", None)?;
    if entries.len() <= maxlen { return Ok(0); }
    let trimmed = entries.len() - maxlen;
    let keep = &entries[trimmed..];

    let tmp_path = self.path.with_extension("jsonl.tmp");
    {
      let mut tmp = File::create(&tmp_path).map_err(|e| format!("create tmp: {e}"))?;
      for entry in keep {
        let line = serde_json::to_string(entry).map_err(|e| format!("serialize: {e}"))?;
        writeln!(tmp, "{line}").map_err(|e| format!("write: {e}"))?;
      }
    }
    fs::rename(&tmp_path, &self.path).map_err(|e| format!("rename: {e}"))?;

    // Reopen writer while still holding the lock
    let file = OpenOptions::new().append(true).open(&self.path)
      .map_err(|e| format!("reopen: {e}"))?;
    *writer = file;

    Ok(trimmed)
  }

  fn load_all(&self) -> Result<Vec<StreamEntry>, String> {
    self.xrange("-", "+", None)
  }
}
```

## Verification

Run `just diagnose`. All types should compile. Write a unit test in `crates/lx/src/event_stream/jsonl.rs` (or a separate test file) that:
1. Creates a JSONL backend with a temp file
2. xadds 3 entries
3. Verifies xrange returns them in order
4. Verifies xlen returns 3
5. Verifies xtrim to 2 removes the oldest entry

Use `#[cfg(test)] mod tests { ... }` for the test module.
