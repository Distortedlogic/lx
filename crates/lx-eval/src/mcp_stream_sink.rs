use std::sync::Arc;

use crate::mcp_client::McpClient;

pub struct McpStreamSink {
  client: Arc<tokio::sync::Mutex<McpClient>>,
}

impl McpStreamSink {
  pub fn new(client: McpClient) -> Self {
    Self { client: Arc::new(tokio::sync::Mutex::new(client)) }
  }
}

impl lx_value::ExternalStreamSink for McpStreamSink {
  fn xadd(&self, entry_json: serde_json::Value) {
    let client = Arc::clone(&self.client);
    tokio::task::spawn(async move {
      if let Err(e) = client.lock().await.tools_call("xadd", entry_json).await {
        eprintln!("[stream:external] xadd failed: {e}");
      }
    });
  }

  fn shutdown(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
    Box::pin(async {
      self.client.lock().await.shutdown().await;
    })
  }
}
