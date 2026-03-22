use std::marker::PhantomData;
use std::time::Duration;

use anyhow::{Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

#[async_trait::async_trait]
pub trait InferenceClient {
    type Request: serde::Serialize + Send + Sync;
    type Response: serde::de::DeserializeOwned;

    async fn infer(&self, req: &Self::Request) -> Result<Self::Response>;
    async fn health(&self) -> Result<bool>;
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GenerationResponse {
    pub data: String,
    pub format: String,
    pub metadata: serde_json::Value,
}

const HEALTH_TIMEOUT_SECS: u64 = 10;

#[derive(Clone)]
pub struct BinaryInferenceClient<R> {
    request_client: reqwest::Client,
    health_client: reqwest::Client,
    infer_url: String,
    health_url: String,
    _phantom: PhantomData<R>,
}

impl<R: serde::Serialize + Send + Sync> BinaryInferenceClient<R> {
    pub fn new(base_url: &str, timeout_secs: u64) -> Result<Self> {
        let base = url::Url::parse(base_url)?;
        let infer_url = base.join("/infer")?.to_string();
        let health_url = base.join("/health")?.to_string();
        let request_client =
            reqwest::Client::builder().timeout(Duration::from_secs(timeout_secs)).build()?;
        let health_client =
            reqwest::Client::builder().timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS)).build()?;
        Ok(Self { request_client, health_client, infer_url, health_url, _phantom: PhantomData })
    }
}

#[async_trait::async_trait]
impl<R: serde::Serialize + Send + Sync> InferenceClient for BinaryInferenceClient<R> {
    type Request = R;
    type Response = Vec<u8>;

    async fn infer(&self, req: &Self::Request) -> Result<Self::Response> {
        let resp = self.request_client.post(&self.infer_url).json(req).send().await?;
        if !resp.status().is_success() {
            bail!(
                "inference failed with status {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            );
        }
        let generation: GenerationResponse = resp.json().await?;
        let bytes = STANDARD.decode(&generation.data)?;
        Ok(bytes)
    }

    async fn health(&self) -> Result<bool> {
        match self.health_client.get(&self.health_url).send().await {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Clone)]
pub struct JsonInferenceClient<Req, Resp> {
    request_client: reqwest::Client,
    health_client: reqwest::Client,
    infer_url: String,
    health_url: String,
    _phantom: PhantomData<(Req, Resp)>,
}

impl<Req, Resp> JsonInferenceClient<Req, Resp>
where
    Req: serde::Serialize + Send + Sync,
    Resp: serde::de::DeserializeOwned,
{
    pub fn new(base_url: &str, timeout_secs: u64) -> Result<Self> {
        let base = url::Url::parse(base_url)?;
        let infer_url = base.join("/infer")?.to_string();
        let health_url = base.join("/health")?.to_string();
        let request_client =
            reqwest::Client::builder().timeout(Duration::from_secs(timeout_secs)).build()?;
        let health_client =
            reqwest::Client::builder().timeout(Duration::from_secs(HEALTH_TIMEOUT_SECS)).build()?;
        Ok(Self { request_client, health_client, infer_url, health_url, _phantom: PhantomData })
    }
}

#[async_trait::async_trait]
impl<Req, Resp> InferenceClient for JsonInferenceClient<Req, Resp>
where
    Req: serde::Serialize + Send + Sync,
    Resp: serde::de::DeserializeOwned + Send + Sync,
{
    type Request = Req;
    type Response = Resp;

    async fn infer(&self, req: &Self::Request) -> Result<Self::Response> {
        let resp = self.request_client.post(&self.infer_url).json(req).send().await?;
        if !resp.status().is_success() {
            bail!(
                "inference failed with status {}: {}",
                resp.status(),
                resp.text().await.unwrap_or_default()
            );
        }
        let result: Resp = resp.json().await?;
        Ok(result)
    }

    async fn health(&self) -> Result<bool> {
        match self.health_client.get(&self.health_url).send().await {
            Ok(r) => Ok(r.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
