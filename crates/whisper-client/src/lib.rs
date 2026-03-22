use std::env;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

pub use inference_client::InferenceClient;

#[derive(Serialize, Deserialize)]
pub struct TranscribeRequest {
    pub audio_data: String,
    pub language: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct TranscribeResponse {
    pub text: String,
    pub language: String,
}

pub type WhisperClient = inference_client::JsonInferenceClient<TranscribeRequest, TranscribeResponse>;

pub static WHISPER_URL: LazyLock<String> =
    LazyLock::new(|| env::var("WHISPER_URL").unwrap_or_else(|_| "http://localhost:8095".to_owned()));

pub static WHISPER: LazyLock<WhisperClient> =
    LazyLock::new(|| WhisperClient::new(&WHISPER_URL, 30).expect("failed to initialize whisper client"));
