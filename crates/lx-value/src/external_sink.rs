pub trait ExternalStreamSink: Send + Sync {
  fn xadd(&self, entry_json: serde_json::Value);
  fn shutdown(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;
}
