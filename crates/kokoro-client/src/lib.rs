use std::env;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

pub use inference_client::InferenceClient;

#[derive(Serialize, Deserialize)]
pub struct SpeechRequest {
  pub text: String,
  pub voice: String,
  pub lang_code: String,
  pub speed: f32,
}

pub type KokoroClient = inference_client::BinaryInferenceClient<SpeechRequest>;

pub static KOKORO_URL: LazyLock<String> = LazyLock::new(|| env::var("KOKORO_URL").unwrap_or_else(|_| "http://localhost:8094".to_owned()));

pub static KOKORO: LazyLock<KokoroClient> = LazyLock::new(|| KokoroClient::new(&KOKORO_URL, 60).expect("failed to initialize kokoro client"));
