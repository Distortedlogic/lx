use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::Utc;
use reqwest::Client;
use serde_json::{Value, json};
use uuid::Uuid;

const DEFAULT_BASE_URL: &str = "https://cloud.langfuse.com";

pub struct LangfuseClient {
    base_url: String,
    auth_header: Option<String>,
    http: Client,
    enabled: bool,
}

impl PartialEq for LangfuseClient {
    fn eq(&self, other: &Self) -> bool {
        self.base_url == other.base_url && self.enabled == other.enabled
    }
}

impl LangfuseClient {
    pub fn from_env() -> Self {
        let public_key = std::env::var("LANGFUSE_PUBLIC_KEY").ok();
        let secret_key = std::env::var("LANGFUSE_SECRET_KEY").ok();
        let base_url =
            std::env::var("LANGFUSE_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let (auth_header, enabled) = match (public_key, secret_key) {
            (Some(pk), Some(sk)) => {
                let creds = format!("{pk}:{sk}");
                let encoded = BASE64.encode(creds.as_bytes());
                (Some(format!("Basic {encoded}")), true)
            }
            _ => (None, false),
        };

        Self {
            base_url,
            auth_header,
            http: Client::new(),
            enabled,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn create_trace<'a>(&'a self, name: &str, metadata: Value) -> LangfuseTrace<'a> {
        let trace_id = Uuid::new_v4().to_string();
        if self.enabled {
            let body = json!({
                "id": trace_id,
                "name": name,
                "metadata": metadata,
                "timestamp": Utc::now().to_rfc3339(),
            });
            self.fire_and_forget("traces", body);
        }
        LangfuseTrace {
            id: trace_id,
            client: self,
        }
    }

    pub fn create_generation<'a>(
        &'a self,
        trace: &LangfuseTrace<'a>,
        name: &str,
        model: &str,
        input: &str,
    ) -> LangfuseGeneration<'a> {
        let gen_id = Uuid::new_v4().to_string();
        if self.enabled {
            let body = json!({
                "id": gen_id,
                "traceId": trace.id,
                "name": name,
                "model": model,
                "input": input,
                "startTime": Utc::now().to_rfc3339(),
            });
            self.fire_and_forget("generations", body);
        }
        LangfuseGeneration {
            id: gen_id,
            trace_id: trace.id.clone(),
            client: self,
        }
    }

    pub fn create_span<'a>(
        &'a self,
        trace: &LangfuseTrace<'a>,
        name: &str,
        input: &str,
    ) -> LangfuseSpan<'a> {
        let span_id = Uuid::new_v4().to_string();
        if self.enabled {
            let body = json!({
                "id": span_id,
                "traceId": trace.id,
                "name": name,
                "input": input,
                "startTime": Utc::now().to_rfc3339(),
            });
            self.fire_and_forget("spans", body);
        }
        LangfuseSpan {
            id: span_id,
            trace_id: trace.id.clone(),
            client: self,
        }
    }

    pub fn log_event(&self, trace_id: &str, level: &str, msg: &str) {
        if !self.enabled {
            return;
        }
        let body = json!({
            "id": Uuid::new_v4().to_string(),
            "traceId": trace_id,
            "name": format!("log.{level}"),
            "body": msg,
            "timestamp": Utc::now().to_rfc3339(),
        });
        self.fire_and_forget("events", body);
    }

    fn fire_and_forget(&self, endpoint: &str, body: Value) {
        let Some(ref auth) = self.auth_header else {
            return;
        };
        let url = format!("{}/api/public/{endpoint}", self.base_url);
        let req = self
            .http
            .post(url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(&body);
        tokio::spawn(async move {
            let _ = req.send().await;
        });
    }
}

pub struct LangfuseTrace<'a> {
    pub id: String,
    client: &'a LangfuseClient,
}

impl<'a> LangfuseTrace<'a> {
    pub fn end(&self, success: bool) {
        if !self.client.enabled {
            return;
        }
        let body = json!({
            "status": if success { "ok" } else { "error" },
            "endTime": Utc::now().to_rfc3339(),
        });
        let Some(ref auth) = self.client.auth_header else {
            return;
        };
        let url = format!("{}/api/public/traces/{}", self.client.base_url, self.id);
        let req = self
            .client
            .http
            .patch(url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(&body);
        tokio::spawn(async move {
            let _ = req.send().await;
        });
    }
}

pub struct LangfuseGeneration<'a> {
    pub id: String,
    pub trace_id: String,
    client: &'a LangfuseClient,
}

impl<'a> LangfuseGeneration<'a> {
    pub fn end_success(&self, output: &str, duration_ms: u64, model: &str) {
        self.end_with(json!({
            "output": output,
            "endTime": Utc::now().to_rfc3339(),
            "completionStartTime": Utc::now().to_rfc3339(),
            "model": model,
            "metadata": { "duration_ms": duration_ms },
        }));
    }

    pub fn end_error(&self, error: &str) {
        self.end_with(json!({
            "statusMessage": error,
            "level": "ERROR",
            "endTime": Utc::now().to_rfc3339(),
        }));
    }

    fn end_with(&self, body: Value) {
        if !self.client.enabled {
            return;
        }
        let Some(ref auth) = self.client.auth_header else {
            return;
        };
        let url = format!(
            "{}/api/public/generations/{}",
            self.client.base_url, self.id
        );
        let req = self
            .client
            .http
            .patch(url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(&body);
        tokio::spawn(async move {
            let _ = req.send().await;
        });
    }
}

pub struct LangfuseSpan<'a> {
    pub id: String,
    pub trace_id: String,
    client: &'a LangfuseClient,
}

impl<'a> LangfuseSpan<'a> {
    pub fn end(&self, output: &str) {
        if !self.client.enabled {
            return;
        }
        let Some(ref auth) = self.client.auth_header else {
            return;
        };
        let url = format!("{}/api/public/spans/{}", self.client.base_url, self.id);
        let body = json!({
            "output": output,
            "endTime": Utc::now().to_rfc3339(),
        });
        let req = self
            .client
            .http
            .patch(url)
            .header("Authorization", auth)
            .header("Content-Type", "application/json")
            .json(&body);
        tokio::spawn(async move {
            let _ = req.send().await;
        });
    }
}
