use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunStatusResponse {
    pub status: String,
    pub source_path: Option<String>,
    pub elapsed_ms: Option<u64>,
    pub cost: Option<f64>,
    pub error: Option<String>,
}

pub struct LxClient {
    base_url: String,
    http: Client,
}

impl LxClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: Client::new(),
        }
    }

    pub async fn fetch_run_status(&self) -> Result<RunStatusResponse, String> {
        let url = format!("{}/api/run/status", self.base_url);
        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<RunStatusResponse>()
            .await
            .map_err(|e| e.to_string())
    }

    pub fn base_url_for_spawn(&self) -> String {
        self.base_url.clone()
    }

    pub async fn fetch_pending_prompts(&self) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/api/run/prompts", self.base_url);
        self.http
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<serde_json::Value>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn post_user_response(
        &self,
        prompt_id: u64,
        response: serde_json::Value,
    ) -> Result<(), String> {
        let url = format!("{}/api/run/respond", self.base_url);
        let body = serde_json::json!({
            "prompt_id": prompt_id,
            "response": response,
        });
        self.http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl PartialEq for LxClient {
    fn eq(&self, other: &Self) -> bool {
        self.base_url == other.base_url
    }
}
