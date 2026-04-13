use reqwest::Client;
use serde::de::DeserializeOwned;

pub struct ApiClient {
  client: Client,
  base_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error("HTTP {status}: {message}")]
  Http { status: u16, message: String, body: Option<serde_json::Value>, raw_body: Option<String> },
  #[error("Request failed: {0}")]
  Request(#[from] reqwest::Error),
  #[error("JSON parse error: {0}")]
  Json(#[from] serde_json::Error),
}

impl ApiClient {
  pub fn new(base_url: String) -> Self {
    Self { client: Client::new(), base_url }
  }

  pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self.client.get(&url).send().await?;
    self.handle_response(resp).await
  }

  pub async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self.client.post(&url).json(body).send().await?;
    self.handle_response(resp).await
  }

  pub async fn put<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self.client.put(&url).json(body).send().await?;
    self.handle_response(resp).await
  }

  pub async fn patch<T: DeserializeOwned>(&self, path: &str, body: &impl serde::Serialize) -> Result<T, ApiError> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self.client.patch(&url).json(body).send().await?;
    self.handle_response(resp).await
  }

  pub async fn delete<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
    let url = format!("{}{}", self.base_url, path);
    let resp = self.client.delete(&url).send().await?;
    self.handle_response(resp).await
  }

  async fn handle_response<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T, ApiError> {
    let status = resp.status();
    if !status.is_success() {
      let text = resp.text().await?;
      if let Ok(body) = serde_json::from_str::<serde_json::Value>(&text) {
        let message = body.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()).unwrap_or_else(|| body.to_string());
        return Err(ApiError::Http { status: status.as_u16(), message, body: Some(body), raw_body: Some(text) });
      }
      let message = if text.trim().is_empty() { "empty error response".to_string() } else { text.clone() };
      return Err(ApiError::Http { status: status.as_u16(), message, body: None, raw_body: Some(text) });
    }
    if status == reqwest::StatusCode::NO_CONTENT {
      let val: T = serde_json::from_value(serde_json::Value::Null)?;
      return Ok(val);
    }
    let text = resp.text().await?;
    let parsed: T = serde_json::from_str(&text)?;
    Ok(parsed)
  }
}

#[cfg(test)]
mod tests {
  use tokio::io::{AsyncReadExt, AsyncWriteExt};
  use tokio::net::TcpListener;

  use super::*;

  #[tokio::test]
  async fn preserves_plain_text_http_errors() {
    let listener = match TcpListener::bind("127.0.0.1:0").await {
      Ok(listener) => listener,
      Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return,
      Err(e) => panic!("bind test listener: {e}"),
    };
    let addr = listener.local_addr().expect("listener addr");

    tokio::spawn(async move {
      let (mut stream, _) = listener.accept().await.expect("accept connection");
      let mut buf = [0_u8; 1024];
      let _ = stream.read(&mut buf).await;
      stream
        .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nContent-Length: 15\r\nConnection: close\r\n\r\nplain text fail")
        .await
        .expect("write response");
    });

    let client = ApiClient::new(format!("http://{addr}"));
    let err = client.get::<serde_json::Value>("/broken").await.expect_err("request should fail");
    match err {
      ApiError::Http { status, message, body, raw_body } => {
        assert_eq!(status, 500);
        assert_eq!(message, "plain text fail");
        assert!(body.is_none());
        assert_eq!(raw_body.as_deref(), Some("plain text fail"));
      },
      other => panic!("expected HTTP error, got {other:?}"),
    }
  }
}
