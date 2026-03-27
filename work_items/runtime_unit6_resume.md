# Unit 6: Resume/Replay Cache

## Goal

Implement the resume system: on program restart with an existing event stream, replay cached tool call results to skip completed work. Uses content-addressed hashing for cache keys to prevent returning wrong cached results when control flow changes between runs.

## Preconditions

- Unit 3 complete: Tool module dispatch works, tool calls go through a single code path
- Unit 4 complete: `StreamBackend` trait with `load_all()` method exists
- Unit 5 complete: Event stream is wired into interpreter, `tool.call`/`tool.result` entries are auto-logged with `call_id`, `tool`, `method`, `args` fields
- `RuntimeCtx` has `event_stream: Option<Arc<dyn StreamBackend>>` and `tool_call_counter: AtomicU64`

## Step 1: Define replay cache type

File: `crates/lx/src/event_stream/replay.rs` (new file)

```rust
use std::collections::HashMap;
use serde_json::Value as JsonValue;

pub struct ReplayCache {
  entries: HashMap<ReplayCacheKey, JsonValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ReplayCacheKey {
  tool: String,
  method: String,
  args_hash: u64,
  ordinal: u64,
}
```

The key is content-addressed: `(tool_name, method_name, hash_of_args, call_ordinal)`. The ordinal is the position of this specific (tool, method, args_hash) combination in the execution sequence. This prevents wrong cache hits when branches change — a different method at the same ordinal position won't match.

### Build cache from stream history

```rust
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

fn hash_json(val: &JsonValue) -> u64 {
  let canonical = serde_json::to_string(val).unwrap_or_default();
  let mut hasher = DefaultHasher::new();
  canonical.hash(&mut hasher);
  hasher.finish()
}

impl ReplayCache {
  pub fn from_entries(entries: &[super::StreamEntry]) -> Self {
    let mut cache = HashMap::new();
    let mut ordinal_counters: HashMap<(String, String, u64), u64> = HashMap::new();

    // Pair tool.call with tool.result by call_id
    let mut calls: HashMap<u64, (&super::StreamEntry, Option<&super::StreamEntry>)> = HashMap::new();

    for entry in entries {
      match entry.kind.as_str() {
        "tool.call" => {
          if let Some(call_id) = entry.fields.get("call_id").and_then(|v| v.as_u64()) {
            calls.entry(call_id).or_insert((entry, None)).0 = entry;
          }
        },
        "tool.result" => {
          if let Some(call_id) = entry.fields.get("call_id").and_then(|v| v.as_u64()) {
            calls.entry(call_id).and_modify(|e| e.1 = Some(entry));
          }
        },
        _ => {},
      }
    }

    // Sort paired entries by call_id to ensure deterministic ordinal assignment
    let mut sorted_calls: Vec<_> = calls.into_iter().collect();
    sorted_calls.sort_by_key(|(call_id, _)| *call_id);

    for (_call_id, (call_entry, result_entry)) in &sorted_calls {
      let Some(result) = result_entry else { continue };
      let tool = call_entry.fields.get("tool").and_then(|v| v.as_str()).unwrap_or("");
      let method = call_entry.fields.get("method").and_then(|v| v.as_str()).unwrap_or("");
      let args = call_entry.fields.get("args").cloned().unwrap_or(JsonValue::Null);
      let args_hash = hash_json(&args);

      let key_prefix = (tool.to_string(), method.to_string(), args_hash);
      let ordinal = ordinal_counters.entry(key_prefix.clone()).or_insert(0);
      *ordinal += 1;

      let cache_key = ReplayCacheKey {
        tool: key_prefix.0,
        method: key_prefix.1,
        args_hash,
        ordinal: *ordinal,
      };

      if let Some(result_val) = result.fields.get("result") {
        cache.insert(cache_key, result_val.clone());
      }
    }

    Self { entries: cache }
  }

  pub fn get(&mut self, tool: &str, method: &str, args: &JsonValue, ordinal: u64) -> Option<JsonValue> {
    let args_hash = hash_json(args);
    let key = ReplayCacheKey {
      tool: tool.to_string(),
      method: method.to_string(),
      args_hash,
      ordinal,
    };
    self.entries.remove(&key)
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }
}
```

## Step 2: Add replay cache to RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

```rust
pub replay_cache: parking_lot::Mutex<Option<crate::event_stream::replay::ReplayCache>>,
```

Default: `Mutex::new(None)`.

## Step 3: Load replay cache on stream initialization

File: `crates/lx/src/interpreter/modules.rs` (in the `UseKind::Stream` handler from Unit 5)

After creating the stream backend, load existing entries and build the replay cache:

```rust
// After creating the stream backend:
let existing = stream.load_all().unwrap_or_default();
if !existing.is_empty() {
  let cache = crate::event_stream::replay::ReplayCache::from_entries(&existing);
  if !cache.is_empty() {
    *self.ctx.replay_cache.lock() = Some(cache);
  }
}
```

Reset the tool call counter to 0 so ordinals match:
```rust
self.ctx.tool_call_counter.store(0, Ordering::Relaxed);
```

## Step 4: Check replay cache before tool calls

In the tool call DynAsync closure (from Unit 3's Step 2, in `crates/lx/src/interpreter/modules.rs`), add replay cache lookup BEFORE making the actual MCP call. Insert this code at the beginning of the closure body, after computing `call_id` and `args_json` (from Unit 5's Step 4d):

```rust
// Inside the DynAsync closure for each tool method:
let call_id = ctx.tool_call_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
let args_json = serde_json::Value::from(&args[0]);

// Content-addressed ordinal for replay cache lookup
let ordinal = next_ordinal(&ctx, &tool_module_name, &method_name, &args_json);

// Check replay cache before making the MCP call
{
  let mut cache = ctx.replay_cache.lock();
  if let Some(ref mut replay) = *cache {
    if let Some(cached_result) = replay.get(&tool_module_name, &method_name, &args_json, ordinal) {
      // Auto-log the replayed call
      let mut call_fields = serde_json::Map::new();
      call_fields.insert("call_id".into(), serde_json::json!(call_id));
      call_fields.insert("tool".into(), serde_json::json!(tool_module_name));
      call_fields.insert("method".into(), serde_json::json!(method_name));
      call_fields.insert("args".into(), args_json);
      crate::event_stream::auto_log(&ctx, "tool.call", call_fields, Some(span));

      let mut result_fields = serde_json::Map::new();
      result_fields.insert("call_id".into(), serde_json::json!(call_id));
      result_fields.insert("tool".into(), serde_json::json!(tool_module_name));
      result_fields.insert("method".into(), serde_json::json!(method_name));
      result_fields.insert("result".into(), cached_result.clone());
      crate::event_stream::auto_log(&ctx, "tool.result", result_fields, Some(span));

      return Ok(LxVal::ok(LxVal::from(cached_result)));
    }
  }
}

// Cache miss — execute live (rest of Unit 3 + Unit 5 Step 4d code follows)
```

The `next_ordinal` function and `replay_ordinals` field are defined in Step 5.

## Step 5: Per-method ordinal tracking

Add ordinal tracking to RuntimeCtx at `crates/lx/src/runtime/mod.rs`:

```rust
pub replay_ordinals: parking_lot::Mutex<std::collections::HashMap<(String, String, u64), u64>>,
```

Default: `parking_lot::Mutex::new(std::collections::HashMap::new())`.

Add this helper function in `crates/lx/src/event_stream/replay.rs`:

```rust
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn hash_json(val: &serde_json::Value) -> u64 {
  let canonical = serde_json::to_string(val).unwrap_or_default();
  let mut hasher = DefaultHasher::new();
  canonical.hash(&mut hasher);
  hasher.finish()
}

pub fn next_ordinal(ctx: &crate::runtime::RuntimeCtx, tool: &str, method: &str, args_json: &serde_json::Value) -> u64 {
  let args_hash = hash_json(args_json);
  let mut ordinals = ctx.replay_ordinals.lock();
  let key = (tool.to_string(), method.to_string(), args_hash);
  let ord = ordinals.entry(key).or_insert(0);
  *ord += 1;
  *ord
}
```

Note: `DefaultHasher` uses `RandomState` which produces different hashes per process. This is fine because both `from_entries` (which builds the cache) and `next_ordinal` (which queries it) run in the same process instance. The hashes only need to be consistent within a single run, not across runs.

## Step 6: Update module exports

File: `crates/lx/src/event_stream/mod.rs`

Add `pub mod replay;` and re-export `ReplayCache`.

## Verification

1. Run `just diagnose`
2. Test scenario:
   - Run a program that makes 3 tool calls, with event stream enabled
   - Check that `.lx/events.jsonl` has tool.call + tool.result entries
   - Run the same program again
   - Verify the first 3 tool calls are replayed from cache (no MCP calls made)
   - Verify new tool calls after the cache is exhausted execute live
