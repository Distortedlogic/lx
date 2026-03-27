# Unit 2: MCP Client Library

## Goal

Implement a JSON-RPC 2.0 client that communicates with MCP (Model Context Protocol) servers over stdin/stdout. This is the Rust-side transport layer that Tool Modules (Unit 3) will use to dispatch method calls to external processes.

## Preconditions

- The workspace Cargo.toml (`Cargo.toml:27-73`) manages dependencies
- `serde_json` and `tokio` are already workspace dependencies
- The `lx` crate (`crates/lx/Cargo.toml`) uses `serde_json.workspace = true` and `tokio.workspace = true`
- `tokio` features include `macros`, `rt-multi-thread`, `sync`, `time`
- No existing MCP client code exists in the codebase

## Step 1: Add module structure

Create new module directory: `crates/lx/src/mcp/`

Files to create:
- `crates/lx/src/mcp/mod.rs` — public module, re-exports
- `crates/lx/src/mcp/jsonrpc.rs` — JSON-RPC 2.0 message types
- `crates/lx/src/mcp/client.rs` — MCP client that spawns process + communicates
- `crates/lx/src/mcp/types.rs` — MCP protocol types (tool definitions, etc.)

Register the module in `crates/lx/src/lib.rs` — add `pub mod mcp;`

## Step 2: JSON-RPC 2.0 message types

File: `crates/lx/src/mcp/jsonrpc.rs`

Define the wire-format types for JSON-RPC 2.0:

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Request {
  pub jsonrpc: &'static str,
  pub id: u64,
  pub method: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub params: Option<serde_json::Value>,
}

impl Request {
  pub fn new(id: u64, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
    Self { jsonrpc: "2.0", id, method: method.into(), params }
  }
}

#[derive(Deserialize)]
pub struct Response {
  pub id: Option<u64>,
  pub result: Option<serde_json::Value>,
  pub error: Option<RpcError>,
}

#[derive(Deserialize)]
pub struct RpcError {
  pub code: i64,
  pub message: String,
  pub data: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct Notification {
  pub method: String,
  pub params: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ServerMessage {
  Response(Response),
  Notification(Notification),
}
```

The `#[serde(untagged)]` on `ServerMessage` allows deserializing either a response (has `id`) or a notification (has `method`, no `id`) from the same stream.

## Step 3: MCP protocol types

File: `crates/lx/src/mcp/types.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ToolInfo {
  pub name: String,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default)]
  pub input_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolsListResult {
  pub tools: Vec<ToolInfo>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallParams {
  pub name: String,
  pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolCallResult {
  #[serde(default)]
  pub content: Vec<ContentBlock>,
  #[serde(default)]
  pub is_error: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlock {
  #[serde(rename = "type")]
  pub content_type: String,
  #[serde(default)]
  pub text: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InitializeParams {
  pub protocol_version: String,
  pub capabilities: serde_json::Value,
  pub client_info: ClientInfo,
}

#[derive(Debug, Serialize)]
pub struct ClientInfo {
  pub name: String,
  pub version: String,
}
```

## Step 4: MCP Client implementation

File: `crates/lx/src/mcp/client.rs`

```rust
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, oneshot};

use super::jsonrpc::{Request, ServerMessage};
use super::types::{ToolInfo, ToolCallParams, ToolCallResult, InitializeParams, ClientInfo, ToolsListResult};
```

### Struct definition

```rust
pub struct McpClient {
  stdin: Arc<Mutex<tokio::process::ChildStdin>>,
  pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>>,
  next_id: AtomicU64,
  child: Arc<Mutex<Child>>,
  tools: Vec<ToolInfo>,
  reader_handle: tokio::task::JoinHandle<()>,
}
```

### spawn method

```rust
impl McpClient {
  pub async fn spawn(command: &str, args: &[&str]) -> Result<Self, String> {
    let mut child = Command::new(command)
      .args(args)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|e| format!("command '{command}' not found: {e}"))?;

    let stdin = child.stdin.take().ok_or("failed to capture stdin")?;
    let stdout = child.stdout.take().ok_or("failed to capture stdout")?;

    let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>> =
      Arc::new(Mutex::new(HashMap::new()));

    let pending_clone = Arc::clone(&pending);
    let reader_handle = tokio::spawn(async move {
      let reader = BufReader::new(stdout);
      let mut lines = reader.lines();
      while let Ok(Some(line)) = lines.next_line().await {
        if let Ok(msg) = serde_json::from_str::<ServerMessage>(&line) {
          match msg {
            ServerMessage::Response(resp) => {
              if let Some(id) = resp.id {
                let mut map = pending_clone.lock().await;
                if let Some(tx) = map.remove(&id) {
                  let result = if let Some(err) = resp.error {
                    Err(err.message)
                  } else {
                    Ok(resp.result.unwrap_or(serde_json::Value::Null))
                  };
                  let _ = tx.send(result);
                }
              }
            },
            ServerMessage::Notification(_notif) => {
              // notifications/message handling: log to event stream
              // (wired up in Unit 5)
            },
          }
        }
      }
    });

    let mut client = Self {
      stdin: Arc::new(Mutex::new(stdin)),
      pending,
      next_id: AtomicU64::new(1),
      child: Arc::new(Mutex::new(child)),
      tools: Vec::new(),
      reader_handle,
    };

    client.initialize().await?;
    client.discover_tools().await?;

    Ok(client)
  }
}
```

### send_request method

```rust
async fn send_request(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
  let id = self.next_id.fetch_add(1, Ordering::Relaxed);
  let request = Request::new(id, method, params);
  let mut line = serde_json::to_string(&request).map_err(|e| format!("serialize: {e}"))?;
  line.push('\n');

  let (tx, rx) = oneshot::channel();
  self.pending.lock().await.insert(id, tx);

  {
    let mut stdin = self.stdin.lock().await;
    stdin.write_all(line.as_bytes()).await.map_err(|e| format!("write: {e}"))?;
    stdin.flush().await.map_err(|e| format!("flush: {e}"))?;
  }

  rx.await.map_err(|_| "channel closed".to_string())?
}
```

### initialize method

Sends the MCP `initialize` handshake:

```rust
async fn initialize(&self) -> Result<(), String> {
  let params = InitializeParams {
    protocol_version: "2024-11-05".to_string(),
    capabilities: serde_json::json!({}),
    client_info: ClientInfo {
      name: "lx".to_string(),
      version: "0.1.0".to_string(),
    },
  };
  let params_json = serde_json::to_value(&params).map_err(|e| format!("serialize: {e}"))?;
  self.send_request("initialize", Some(params_json)).await?;

  // Send initialized notification (no id, no response expected)
  let notif = serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"});
  let mut line = serde_json::to_string(&notif).map_err(|e| format!("serialize: {e}"))?;
  line.push('\n');
  let mut stdin = self.stdin.lock().await;
  stdin.write_all(line.as_bytes()).await.map_err(|e| format!("write: {e}"))?;
  stdin.flush().await.map_err(|e| format!("flush: {e}"))?;

  Ok(())
}
```

### discover_tools method

```rust
async fn discover_tools(&mut self) -> Result<(), String> {
  let result = self.send_request("tools/list", None).await?;
  let tools_result: ToolsListResult = serde_json::from_value(result)
    .map_err(|e| format!("tools/list parse: {e}"))?;
  self.tools = tools_result.tools;
  Ok(())
}
```

### call_tool method (public API used by Unit 3)

```rust
pub async fn call_tool(&self, method: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
  if !self.tools.iter().any(|t| t.name == method) {
    return Err(format!("unknown method '{method}'"));
  }
  let params = ToolCallParams { name: method.to_string(), arguments: args };
  let params_json = serde_json::to_value(&params).map_err(|e| format!("serialize: {e}"))?;
  let raw = self.send_request("tools/call", Some(params_json)).await?;

  // Deserialize the MCP tools/call response
  let call_result: ToolCallResult = serde_json::from_value(raw)
    .map_err(|e| format!("tools/call response parse: {e}"))?;
  if call_result.is_error {
    let msg = call_result.content.iter()
      .filter_map(|c| c.text.as_deref())
      .collect::<Vec<_>>()
      .join("\n");
    return Err(msg);
  }
  // Extract text content blocks into a single string or structured value
  let texts: Vec<&str> = call_result.content.iter()
    .filter_map(|c| c.text.as_deref())
    .collect();
  if texts.len() == 1 {
    // Try to parse as JSON first
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(texts[0]) {
      Ok(json)
    } else {
      Ok(serde_json::Value::String(texts[0].to_string()))
    }
  } else {
    Ok(serde_json::Value::String(texts.join("\n")))
  }
}

pub fn available_tools(&self) -> &[ToolInfo] {
  &self.tools
}

pub fn has_tool(&self, name: &str) -> bool {
  self.tools.iter().any(|t| t.name == name)
}
```

### shutdown method

Per the architecture doc: send MCP shutdown, wait briefly, then kill if necessary.

```rust
pub async fn shutdown(&mut self) {
  // Send MCP shutdown request (best-effort)
  if let Ok(()) = self.send_request("shutdown", None).await.map(|_| ()) {
    // Wait briefly for process to exit
    let child = Arc::clone(&self.child);
    let wait = async {
      let mut c = child.lock().await;
      c.wait().await
    };
    if tokio::time::timeout(std::time::Duration::from_secs(2), wait).await.is_err() {
      let mut c = self.child.lock().await;
      // Process didn't exit gracefully — kill it
      if let Err(e) = c.kill().await {
        eprintln!("[mcp] kill failed: {e}");
      }
    }
  } else {
    // Shutdown request failed — force kill
    let mut c = self.child.lock().await;
    if let Err(e) = c.kill().await {
      eprintln!("[mcp] kill failed: {e}");
    }
  }
  self.reader_handle.abort();
}
```

## Step 5: Module re-exports

File: `crates/lx/src/mcp/mod.rs`

```rust
mod client;
mod jsonrpc;
mod types;

pub use client::McpClient;
pub use types::ToolInfo;
```

## Step 6: Serde attributes justification

The `#[serde(default)]`, `#[serde(skip_serializing_if)]`, and `#[serde(rename)]` attributes in the types above are required for the external MCP/JSON-RPC wire format. We do NOT control this format — it's defined by the MCP protocol specification. These are NOT backwards-compatibility code. Do not remove them.

## Step 7: Verify tokio features

The MCP client uses `tokio::process` which requires the `process` feature. Check if `Cargo.toml` has it.

File: `Cargo.toml` (workspace root), line 69:
```toml
tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread", "sync", "time"] }
```

The `process` feature is missing. Add it:
```toml
tokio = { version = "1.50.0", features = ["macros", "process", "rt-multi-thread", "sync", "time", "io-util"] }
```

Also add `io-util` for `AsyncBufReadExt` and `AsyncWriteExt`.

## Step 8: Handle notifications (collect, don't drop)

In the `spawn` method's reader task, the `ServerMessage::Notification` arm currently does nothing. Replace it with collection into a shared vec for later use by the event stream (Unit 5):

```rust
ServerMessage::Notification(notif) => {
  if notif.method == "notifications/message" {
    if let Some(params) = notif.params {
      // Store for event stream auto-logging
      // For now, log to stderr as a visible signal
      if let Some(msg) = params.get("message").and_then(|m| m.as_str()) {
        eprintln!("[mcp:notification] {msg}");
      }
    }
  }
},
```

## Verification

Run `just diagnose`. The new module should compile with no errors or warnings. The McpClient is not wired into the interpreter yet — that happens in Unit 3.
