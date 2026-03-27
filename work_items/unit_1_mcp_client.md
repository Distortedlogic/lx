# Unit 1: MCP Client Library

Self-contained JSON-RPC 2.0 client for communicating with MCP servers over stdin/stdout. Built from scratch using `serde_json` + `tokio` (both already workspace dependencies).

## Prerequisites

None. This unit has no dependencies on other units.

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify
- Prefer established crates over custom code

## Files to Create

- `crates/lx/src/mcp/mod.rs`
- `crates/lx/src/mcp/jsonrpc.rs`
- `crates/lx/src/mcp/client.rs`
- `crates/lx/src/mcp/types.rs`

## Files to Modify

- `crates/lx/src/lib.rs` -- add `pub mod mcp;` after line 13 (after `pub mod linter;`), before `pub mod parser;`
- `Cargo.toml` (workspace root) -- add `process` and `io-util` features to tokio dependency on line 69

## Step 1: Add tokio features

File: `Cargo.toml` (workspace root), line 69.

Current:
```toml
tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread", "sync", "time"] }
```

Change to:
```toml
tokio = { version = "1.50.0", features = ["io-util", "macros", "process", "rt-multi-thread", "sync", "time"] }
```

The `process` feature is needed for `tokio::process::Command`. The `io-util` feature is needed for `AsyncBufReadExt` and `AsyncWriteExt`.

## Step 2: Create JSON-RPC types

File: `crates/lx/src/mcp/jsonrpc.rs`

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

## Step 3: Create MCP protocol types

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

## Step 4: Create MCP Client

File: `crates/lx/src/mcp/client.rs`

This is the core client that spawns a child process, connects via JSON-RPC, and provides an async API for tool calls.

```rust
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, oneshot, mpsc};

use super::jsonrpc::{Request, ServerMessage};
use super::types::{
    ClientInfo, InitializeParams, ToolCallParams, ToolInfo, ToolsListResult,
};

pub struct McpClient {
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<Result<serde_json::Value, String>>>>>,
    next_id: AtomicU64,
    child: Arc<Mutex<Child>>,
    tools: Vec<ToolInfo>,
    reader_handle: tokio::task::JoinHandle<()>,
    pub notification_rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<NotificationEvent>>>,
}

pub struct NotificationEvent {
    pub method: String,
    pub params: Option<serde_json::Value>,
}
```

### Public methods

```rust
impl McpClient {
    pub async fn spawn(command: &str) -> Result<Self, String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let (cmd, args) = parts.split_first().ok_or_else(|| "empty command".to_string())?;
        Self::spawn_with_args(cmd, args).await
    }

    pub async fn spawn_with_args(command: &str, args: &[&str]) -> Result<Self, String> {
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

        let (notif_tx, notif_rx) = mpsc::unbounded_channel();
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
                        ServerMessage::Notification(notif) => {
                            let _ = notif_tx.send(NotificationEvent {
                                method: notif.method,
                                params: notif.params,
                            });
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
            notification_rx: Arc::new(tokio::sync::Mutex::new(notif_rx)),
        };

        client.initialize().await?;
        client.discover_tools().await?;
        Ok(client)
    }

    pub async fn call_tool(
        &self,
        method: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let params = ToolCallParams {
            name: method.to_string(),
            arguments: args,
        };
        let params_json =
            serde_json::to_value(&params).map_err(|e| format!("serialize: {e}"))?;
        self.send_request("tools/call", Some(params_json)).await
    }

    pub fn available_tools(&self) -> &[ToolInfo] {
        &self.tools
    }

    pub async fn shutdown(&self) {
        let mut child = self.child.lock().await;
        let _ = child.kill().await;
        self.reader_handle.abort();
    }

    pub async fn is_alive(&self) -> bool {
        let mut child = self.child.lock().await;
        matches!(child.try_wait(), Ok(None))
    }
}
```

`JoinHandle::abort` takes `&self`, so `shutdown` works directly with `&self`.

### Private methods

```rust
impl McpClient {
    async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = Request::new(id, method, params);
        let mut line =
            serde_json::to_string(&request).map_err(|e| format!("serialize: {e}"))?;
        line.push('\n');

        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        {
            let mut stdin = self.stdin.lock().await;
            stdin
                .write_all(line.as_bytes())
                .await
                .map_err(|e| format!("write: {e}"))?;
            stdin.flush().await.map_err(|e| format!("flush: {e}"))?;
        }

        rx.await.map_err(|_| "channel closed".to_string())?
    }

    async fn initialize(&self) -> Result<(), String> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: serde_json::json!({}),
            client_info: ClientInfo {
                name: "lx".to_string(),
                version: "0.1.0".to_string(),
            },
        };
        let params_json =
            serde_json::to_value(&params).map_err(|e| format!("serialize: {e}"))?;
        self.send_request("initialize", Some(params_json)).await?;

        let notif = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let mut line =
            serde_json::to_string(&notif).map_err(|e| format!("serialize: {e}"))?;
        line.push('\n');
        let mut stdin = self.stdin.lock().await;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("write: {e}"))?;
        stdin.flush().await.map_err(|e| format!("flush: {e}"))?;
        Ok(())
    }

    async fn discover_tools(&mut self) -> Result<(), String> {
        let result = self.send_request("tools/list", None).await?;
        let tools_result: ToolsListResult =
            serde_json::from_value(result).map_err(|e| format!("tools/list parse: {e}"))?;
        self.tools = tools_result.tools;
        Ok(())
    }
}
```

## Step 5: Create module file

File: `crates/lx/src/mcp/mod.rs`

```rust
mod client;
mod jsonrpc;
mod types;

pub use client::{McpClient, NotificationEvent};
pub use types::ToolInfo;
```

## Step 6: Register module in lib.rs

File: `crates/lx/src/lib.rs`

Add after line 13 (`pub mod linter;`), before `pub mod parser;`:
```rust
pub mod mcp;
```

The full file becomes:
```rust
pub const PLUGIN_MANIFEST: &str = "plugin.toml";
pub const LX_MANIFEST: &str = "lx.toml";

pub mod ast;
pub mod builtins;
pub mod checker;
pub mod env;
pub mod error;
pub mod folder;
pub mod formatter;
pub mod interpreter;
pub mod lexer;
pub mod linter;
pub mod mcp;
pub mod parser;
pub mod runtime;
pub mod source;
pub mod stdlib;
pub mod sym;
pub mod value;
pub mod visitor;
```

## Verification

Run `just diagnose`. The new `mcp` module compiles with no errors or warnings. The `McpClient` is not wired into the interpreter -- that happens in Unit 3.
