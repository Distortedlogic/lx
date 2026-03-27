# Unit 4: EventStream Trait + JSONL Backend

Define the `EventStream` trait and implement a JSONL file-based backend for persistent event logging.

## Prerequisites

None. This unit has no dependencies on other units.

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify
- Prefer established crates over custom code

## Current State

- `RuntimeCtx` is in `crates/lx/src/runtime/mod.rs` (lines 20-39) using `SmartDefault` derive
- `RuntimeCtx` already has backend traits: `EmitBackend`, `HttpBackend`, `YieldBackend`, `LogBackend`, `LlmBackend`
- The runtime module has submodules: `defaults.rs`, `noop.rs`, `restricted.rs`
- `crates/lx/src/runtime/mod.rs` re-exports all three: `pub use defaults::*; pub use noop::*; pub use restricted::*;`

## Files to Create

- `crates/lx/src/runtime/event_stream.rs` -- EventStream trait and entry types
- `crates/lx/src/runtime/jsonl_backend.rs` -- JSONL file-based EventStream implementation

## Files to Modify

- `crates/lx/src/runtime/mod.rs` -- add module declarations, add `event_stream` field to RuntimeCtx

## Step 1: Define EventStream trait and types

File: `crates/lx/src/runtime/event_stream.rs`

### Stream entry ID format

IDs follow the format `{unix_ms}-{seq}` where `unix_ms` is wall clock time in milliseconds and `seq` is a per-millisecond monotonic counter starting at 0. Example: `"1679083200123-0"`, `"1679083200123-1"`.

### Types

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::value::LxVal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEntry {
    pub id: String,
    pub kind: String,
    pub agent: String,
    pub ts: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SpanInfo>,
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    pub line: u32,
    pub col: u32,
}

pub struct IdGenerator {
    last_ms: AtomicU64,
    seq: AtomicU64,
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGenerator {
    pub fn new() -> Self {
        Self {
            last_ms: AtomicU64::new(0),
            seq: AtomicU64::new(0),
        }
    }

    pub fn next_id(&self) -> (String, u64) {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let prev_ms = self.last_ms.load(Ordering::SeqCst);
        if now_ms == prev_ms {
            let seq = self.seq.fetch_add(1, Ordering::SeqCst);
            (format!("{now_ms}-{seq}"), now_ms)
        } else {
            self.last_ms.store(now_ms, Ordering::SeqCst);
            self.seq.store(1, Ordering::SeqCst);
            (format!("{now_ms}-0"), now_ms)
        }
    }
}
```

### Trait definition

```rust
pub trait EventStream: Send + Sync {
    fn xadd(&self, entry: StreamEntry) -> Result<String, String>;

    fn xrange(
        &self,
        start: &str,
        end: &str,
        count: Option<usize>,
    ) -> Result<Vec<StreamEntry>, String>;

    fn xread(
        &self,
        last_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<Option<StreamEntry>, String>;

    fn xlen(&self) -> Result<usize, String>;

    fn xtrim(&self, maxlen: usize) -> Result<usize, String>;
}
```

### Helper to build entries

```rust
impl StreamEntry {
    pub fn new(kind: &str, agent: &str, id_gen: &IdGenerator) -> Self {
        let (id, ts) = id_gen.next_id();
        Self {
            id,
            kind: kind.to_string(),
            agent: agent.to_string(),
            ts,
            span: None,
            fields: HashMap::new(),
        }
    }

    pub fn with_span(mut self, span: Option<SpanInfo>) -> Self {
        self.span = span;
        self
    }

    pub fn with_field(mut self, key: &str, value: serde_json::Value) -> Self {
        self.fields.insert(key.to_string(), value);
        self
    }

    pub fn to_lx_val(&self) -> LxVal {
        let mut fields = IndexMap::new();
        fields.insert(crate::sym::intern("id"), LxVal::str(&self.id));
        fields.insert(crate::sym::intern("kind"), LxVal::str(&self.kind));
        fields.insert(crate::sym::intern("agent"), LxVal::str(&self.agent));
        fields.insert(crate::sym::intern("ts"), LxVal::int(self.ts as i64));
        if let Some(ref span) = self.span {
            let mut span_fields = IndexMap::new();
            span_fields.insert(crate::sym::intern("line"), LxVal::int(span.line as i64));
            span_fields.insert(crate::sym::intern("col"), LxVal::int(span.col as i64));
            fields.insert(crate::sym::intern("span"), LxVal::record(span_fields));
        }
        for (k, v) in &self.fields {
            fields.insert(crate::sym::intern(k), LxVal::from(v.clone()));
        }
        LxVal::record(fields)
    }
}
```

`LxVal::from(serde_json::Value)` exists in `crates/lx/src/value/serde_impl.rs`.

## Step 2: Implement JSONL backend

File: `crates/lx/src/runtime/jsonl_backend.rs`

```rust
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;

use super::event_stream::{EventStream, IdGenerator, StreamEntry};

pub struct JsonlBackend {
    path: String,
    file: Mutex<File>,
    id_gen: IdGenerator,
    entries: Mutex<Vec<StreamEntry>>,
}

impl JsonlBackend {
    pub fn new(path: &str) -> Result<Self, String> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("cannot open stream file '{path}': {e}"))?;

        let mut entries = Vec::new();
        if let Ok(read_file) = File::open(path) {
            let reader = BufReader::new(read_file);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Ok(entry) = serde_json::from_str::<StreamEntry>(&line) {
                        entries.push(entry);
                    }
                }
            }
        }

        Ok(Self {
            path: path.to_string(),
            file: Mutex::new(file),
            id_gen: IdGenerator::new(),
            entries: Mutex::new(entries),
        })
    }

    pub fn entries(&self) -> Vec<StreamEntry> {
        self.entries.lock().map(|e| e.clone()).unwrap_or_default()
    }
}
```

### EventStream implementation

```rust
impl EventStream for JsonlBackend {
    fn xadd(&self, mut entry: StreamEntry) -> Result<String, String> {
        if entry.id.is_empty() {
            let (id, ts) = self.id_gen.next_id();
            entry.id = id;
            entry.ts = ts;
        }

        let line = serde_json::to_string(&entry)
            .map_err(|e| format!("serialize: {e}"))?;

        {
            let mut file = self.file.lock().map_err(|e| format!("lock: {e}"))?;
            writeln!(file, "{line}").map_err(|e| format!("write: {e}"))?;
            file.flush().map_err(|e| format!("flush: {e}"))?;
        }

        let id = entry.id.clone();
        self.entries.lock().map_err(|e| format!("lock: {e}"))?.push(entry);
        Ok(id)
    }

    fn xrange(
        &self,
        start: &str,
        end: &str,
        count: Option<usize>,
    ) -> Result<Vec<StreamEntry>, String> {
        let entries = self.entries.lock().map_err(|e| format!("lock: {e}"))?;
        let mut result: Vec<StreamEntry> = entries
            .iter()
            .filter(|e| {
                let after_start = start == "-" || e.id.as_str() >= start;
                let before_end = end == "+" || e.id.as_str() <= end;
                after_start && before_end
            })
            .cloned()
            .collect();
        if let Some(max) = count {
            result.truncate(max);
        }
        Ok(result)
    }

    fn xread(
        &self,
        last_id: &str,
        timeout_ms: Option<u64>,
    ) -> Result<Option<StreamEntry>, String> {
        let deadline = timeout_ms.map(|ms| {
            std::time::Instant::now() + std::time::Duration::from_millis(ms)
        });

        loop {
            {
                let entries = self.entries.lock().map_err(|e| format!("lock: {e}"))?;
                let found = if last_id == "$" {
                    None
                } else {
                    entries
                        .iter()
                        .find(|e| e.id.as_str() > last_id)
                        .cloned()
                };
                if found.is_some() {
                    return Ok(found);
                }
            }

            if let Some(deadline) = deadline {
                if std::time::Instant::now() >= deadline {
                    return Ok(None);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn xlen(&self) -> Result<usize, String> {
        Ok(self.entries.lock().map_err(|e| format!("lock: {e}"))?.len())
    }

    fn xtrim(&self, maxlen: usize) -> Result<usize, String> {
        let mut entries = self.entries.lock().map_err(|e| format!("lock: {e}"))?;
        if entries.len() <= maxlen {
            return Ok(0);
        }
        let removed = entries.len() - maxlen;
        let _ = entries.drain(..removed);

        let mut file = self.file.lock().map_err(|e| format!("lock: {e}"))?;
        let new_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|e| format!("rewrite: {e}"))?;
        *file = new_file;
        for entry in entries.iter() {
            let line = serde_json::to_string(entry)
                .map_err(|e| format!("serialize: {e}"))?;
            writeln!(file, "{line}").map_err(|e| format!("write: {e}"))?;
        }
        file.flush().map_err(|e| format!("flush: {e}"))?;
        Ok(removed)
    }
}
```

Note on `xread`: The `EventStream` trait is `Send + Sync` (not async). The `xread` method uses `std::thread::sleep` for polling because it is a synchronous trait method. The JSONL backend is an in-process data structure, so the blocking sleep is brief (10ms) and acceptable. For production use with external backends, the trait would be made async.

## Step 3: Add event_stream field to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

### 3a: Add module declarations

After the existing module declarations (lines 1-3):
```rust
mod defaults;
mod noop;
mod restricted;
```

Add:
```rust
pub mod event_stream;
mod jsonl_backend;
```

### 3b: Add re-exports

After the existing re-exports (lines 5-7):
```rust
pub use defaults::*;
pub use noop::*;
pub use restricted::*;
```

Add:
```rust
pub use event_stream::{EventStream, IdGenerator, SpanInfo, StreamEntry};
pub use jsonl_backend::JsonlBackend;
```

### 3c: Add field to RuntimeCtx

In the `RuntimeCtx` struct (lines 20-39), add after the `test_runs` field (line 38):

```rust
pub event_stream: Option<Arc<dyn EventStream>>,
```

The `SmartDefault` derive defaults `Option<...>` to `None`, which is correct -- no event stream is active by default.

### 3d: Add helper method to RuntimeCtx

Add an impl block after the struct definition (or at the bottom of the file):

```rust
impl RuntimeCtx {
    pub fn xadd(&self, entry: StreamEntry) -> Option<String> {
        self.event_stream.as_ref().and_then(|s| s.xadd(entry).ok())
    }
}
```

This convenience method is used by the interpreter's auto-logging code (Unit 6) to conditionally log if a stream is active.

## Step 4: Verify file lengths

Ensure both new files are under 300 lines:
- `event_stream.rs`: ~120 lines (types + trait + helpers)
- `jsonl_backend.rs`: ~130 lines (struct + EventStream impl)

Both are well under the limit.

## Verification

1. Run `just diagnose` -- no compiler errors or clippy warnings
2. All existing tests pass unchanged
3. The new types are defined but not yet wired into the interpreter -- that happens in Unit 5
