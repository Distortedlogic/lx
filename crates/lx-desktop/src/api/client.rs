use reqwest::Client;
use serde::de::DeserializeOwned;

pub struct ApiClient {
  client: Client,
  base_url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
  #[error("HTTP {status}: {message}")]
  Http { status: u16, message: String, body: Option<serde_json::Value> },
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
      let body: Option<serde_json::Value> = resp.json().await.ok();
      let message = body.as_ref().and_then(|b| b.get("error")).and_then(|e| e.as_str()).unwrap_or("Unknown error").to_string();
      return Err(ApiError::Http { status: status.as_u16(), message, body });
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
