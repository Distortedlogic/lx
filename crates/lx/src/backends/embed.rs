use std::sync::Arc;

use reqwest::Client;
use serde_json::json;

use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::{EmbedBackend, EmbedOpts};

pub struct VoyageEmbedBackend;

impl EmbedBackend for VoyageEmbedBackend {
    fn embed(&self, texts: &[String], opts: &EmbedOpts, span: Span) -> Result<LxVal, LxError> {
        let api_key = match std::env::var("VOYAGE_API_KEY") {
            Ok(k) if !k.is_empty() => k,
            _ => {
                return Ok(LxVal::Err(Box::new(LxVal::Str(Arc::from(
                    "VOYAGE_API_KEY not set — get one at https://dash.voyageai.com/",
                )))));
            }
        };

        let model = opts.model.as_deref().unwrap_or("voyage-3-lite");

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let client = Client::builder()
                    .build()
                    .map_err(|e| LxError::runtime(format!("embed: client: {e}"), span))?;

                let mut body = json!({
                    "input": texts,
                    "model": model,
                });
                if let (Some(dim), Some(obj)) = (opts.dimensions, body.as_object_mut()) {
                    obj.insert("output_dimension".into(), json!(dim));
                }

                let resp = client
                    .post("https://api.voyageai.com/v1/embeddings")
                    .header("Authorization", format!("Bearer {api_key}"))
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await;

                let resp = match resp {
                    Ok(r) => r,
                    Err(e) => {
                        return Ok(LxVal::Err(Box::new(LxVal::Str(Arc::from(format!(
                            "embed: request failed: {e}"
                        ))))));
                    }
                };

                let status = resp.status().as_u16();
                let body_text = resp
                    .text()
                    .await
                    .map_err(|e| LxError::runtime(format!("embed: body read: {e}"), span))?;

                if status != 200 {
                    return Ok(LxVal::Err(Box::new(LxVal::Str(Arc::from(format!(
                        "embed: API error {status}: {body_text}"
                    ))))));
                }

                let jv: serde_json::Value = serde_json::from_str(&body_text)
                    .map_err(|e| LxError::runtime(format!("embed: JSON parse: {e}"), span))?;

                let data = jv.get("data").and_then(|d| d.as_array()).ok_or_else(|| {
                    LxError::runtime("embed: missing 'data' array in response", span)
                })?;

                let vectors: Vec<LxVal> = data
                    .iter()
                    .filter_map(|item| item.get("embedding").and_then(|e| e.as_array()))
                    .map(|arr| {
                        let floats: Vec<LxVal> = arr
                            .iter()
                            .filter_map(|f| f.as_f64())
                            .map(LxVal::Float)
                            .collect();
                        LxVal::List(Arc::new(floats))
                    })
                    .collect();

                Ok(LxVal::Ok(Box::new(LxVal::List(Arc::new(vectors)))))
            })
        })
    }
}
