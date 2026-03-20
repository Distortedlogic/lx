use std::time::Duration;

use futures::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub struct EventWsClient {
    url: String,
}

impl EventWsClient {
    pub fn new(base_url: &str) -> Self {
        let ws_url = base_url.replacen("http", "ws", 1);
        Self {
            url: format!("{ws_url}/ws/events"),
        }
    }

    pub async fn connect_and_stream(&self, tx: tokio::sync::mpsc::Sender<serde_json::Value>) {
        let mut backoff = Duration::from_secs(1);
        loop {
            if let Ok((ws_stream, _)) = connect_async(&self.url).await {
                backoff = Duration::from_secs(1);
                let (_sink, mut stream) = ws_stream.split();
                while let Some(Ok(msg)) = stream.next().await {
                    if let Message::Text(text) = msg
                        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text)
                        && tx.send(val).await.is_err()
                    {
                        return;
                    }
                }
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(30));
        }
    }
}
