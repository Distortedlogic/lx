# Unit 03: Runtime Error Surface Cleanup

## Goal

Remove the verified runtime-side swallowed-error patterns in transport, tool execution, event streaming, and checkpoint persistence. Runtime failures should either return an error, emit a concrete log message, or both. They must not silently degrade into empty JSON payloads, `Null`, or “missing checkpoint” behavior.

## Preconditions

- Unit 01 should be complete first.
- Unit 02 may land before or after this unit; there is no semantic dependency, but take the latest `BuiltinCtx` signature shape if both units touch `checkpoint.rs`.

## Verified Findings

- `crates/lx-eval/src/mcp_client.rs`
  - collapses process spawn failures to `command '{command}' not found`
  - ignores `shutdown` request failure
  - ignores forced-kill failure after timeout
- `crates/lx-eval/src/tool_module.rs`
  - falls back to `{}` when `serde_json::to_value(...)` fails for tool arguments
- `crates/lx-eval/src/runtime/control_stdin.rs`
  - uses `serde_json::to_string(&resp).unwrap_or_default()`
- `crates/lx-eval/src/runtime/control_tcp.rs`
  - uses `serde_json::to_string(&resp).unwrap_or_default()`
  - ignores `writer.write_all(...)` errors
- `crates/lx-eval/src/runtime/control_ws.rs`
  - uses `serde_json::to_string(&resp).unwrap_or_default()`
  - ignores `write.send(...)` errors
- `crates/lx-value/src/event_stream.rs`
  - ignores `create_dir_all(...)` failures in `new(...)` and `enable_jsonl(...)`
  - silently disables JSONL writing if the file cannot be opened
  - coerces external-sink serialization failure to `serde_json::Value::Null`
- `crates/lx-eval/src/stdlib/checkpoint.rs`
  - `read_checkpoint(...)` returns `None` for both “file missing” and “file exists but read/JSON parse failed”, which masks corrupted checkpoint state as a cache miss

## Files to Modify

- `crates/lx-eval/src/mcp_client.rs`
- `crates/lx-eval/src/tool_module.rs`
- `crates/lx-eval/src/runtime/control_stdin.rs`
- `crates/lx-eval/src/runtime/control_tcp.rs`
- `crates/lx-eval/src/runtime/control_ws.rs`
- `crates/lx-eval/src/stdlib/checkpoint.rs`
- `crates/lx-value/src/event_stream.rs`

## Steps

### Step 1: Preserve real MCP process failures

In `crates/lx-eval/src/mcp_client.rs`:

- Replace the blanket `map_err(|_| format!("command '{command}' not found"))` on process spawn with an error that preserves the underlying OS failure text.
- Keep the human-readable command in the message, but do not claim “not found” unless that is actually the error.
- Make `shutdown()` report meaningful failures internally:
  - log when the shutdown RPC fails
  - log when the forced kill fails after timeout

Do not change the public return type of `spawn(...)` just to hide errors differently. The goal is clearer error propagation, not a new wrapper type for its own sake.

### Step 2: Stop converting tool-argument serialization failures into empty objects

In `crates/lx-eval/src/tool_module.rs`, remove the `unwrap_or(json!({}))` fallbacks around `serde_json::to_value(...)`.

Required behavior:

- if argument serialization fails, return an `LxError`
- emit the corresponding `tool/error` event with the actual serialization failure message
- do not call `tools_call(...)` with a fabricated empty object

Keep the special `LxVal::Str` and `LxVal::Unit` mapping behavior unchanged.

### Step 3: Make control-channel responses fail loudly instead of silently

In:

- `crates/lx-eval/src/runtime/control_stdin.rs`
- `crates/lx-eval/src/runtime/control_tcp.rs`
- `crates/lx-eval/src/runtime/control_ws.rs`

introduce one shared local pattern per file for “serialize response or fall back to an explicit error string,” and stop using `unwrap_or_default()`.

For TCP and WebSocket:

- if response serialization fails, send an explicit serialization-error response if possible
- if the send/write itself fails, log the transport error and break the loop instead of ignoring it

For stdin:

- print a concrete serialization failure line instead of printing an empty string

Do not add a new shared crate for this. Keep the fix local to the control transport modules.

### Step 4: Surface JSONL writer setup and sink serialization failures

In `crates/lx-value/src/event_stream.rs`:

- stop silently discarding `create_dir_all(...)` failures in `new(...)` and `enable_jsonl(...)`
- stop silently treating “failed to open JSONL file” as “JSONL disabled”
- when the external sink payload cannot be serialized, log that failure and skip the sink dispatch rather than sending `serde_json::Value::Null`

Keep the in-memory event stream operational even when JSONL persistence fails. The fix is not to make `EventStream::new(...)` infallible; it is to stop hiding why persistence is unavailable.

### Step 5: Distinguish missing checkpoints from corrupted checkpoints

In `crates/lx-eval/src/stdlib/checkpoint.rs`:

- change `read_checkpoint(...)` so it does not collapse:
  - missing file
  - file read failure
  - JSON parse failure
  into the same `None` result
- use the checkpoint step/scope call path to surface real read/parse failures as `LxError`
- preserve the current “cache miss means execute the body” behavior only for the genuine missing-file case

Do not leave corrupted checkpoint files behaving like a cache miss. That is the audit violation this unit is fixing.

## Verification

1. Run `just test`.
2. Run `just rust-diagnose`.
3. Run `rg -n 'unwrap_or_default\\(|let _ = .*send\\(|let _ = .*write_all\\(|serde_json::Value::Null' crates/lx-eval/src/mcp_client.rs crates/lx-eval/src/tool_module.rs crates/lx-eval/src/runtime/control_stdin.rs crates/lx-eval/src/runtime/control_tcp.rs crates/lx-eval/src/runtime/control_ws.rs crates/lx-eval/src/stdlib/checkpoint.rs crates/lx-value/src/event_stream.rs`.
4. Confirm the remaining matches, if any, are explicitly justified by the final implementation and are not swallowing `Result` failures in the targeted runtime paths.
5. Manually exercise one MCP startup failure path and one control-channel invalid-command path to confirm the user-facing output now includes the real error instead of an empty/default payload.
