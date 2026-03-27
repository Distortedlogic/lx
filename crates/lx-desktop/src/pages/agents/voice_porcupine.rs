#![allow(unsafe_code)]

use std::sync::{LazyLock, Mutex};

use dioxus::logger::tracing::{info, warn};
use sherpa_onnx::{KeywordSpotter, KeywordSpotterConfig};

struct KwsEngine {
  spotter: KeywordSpotter,
  stream: sherpa_onnx::OnlineStream,
}

unsafe impl Send for KwsEngine {}

static ENGINE: LazyLock<Option<Mutex<KwsEngine>>> = LazyLock::new(|| {
  let model_dir = std::env::var("SHERPA_KWS_MODEL_DIR").ok()?;
  let dir = std::path::Path::new(&model_dir);
  let encoder = dir.join("encoder.onnx");
  let decoder = dir.join("decoder.onnx");
  let joiner = dir.join("joiner.onnx");
  let tokens = dir.join("tokens.txt");
  let keywords = dir.join("keywords.txt");
  for path in [&encoder, &decoder, &joiner, &tokens, &keywords] {
    if !path.exists() {
      warn!("kws: missing file {}", path.display());
      return None;
    }
  }
  let mut config = KeywordSpotterConfig::default();
  config.model_config.transducer.encoder = Some(encoder.to_string_lossy().into_owned());
  config.model_config.transducer.decoder = Some(decoder.to_string_lossy().into_owned());
  config.model_config.transducer.joiner = Some(joiner.to_string_lossy().into_owned());
  config.model_config.tokens = Some(tokens.to_string_lossy().into_owned());
  config.keywords_file = Some(keywords.to_string_lossy().into_owned());
  config.model_config.num_threads = 1;
  let Some(spotter) = KeywordSpotter::create(&config) else {
    warn!("kws: failed to create keyword spotter");
    return None;
  };
  let stream = spotter.create_stream();
  info!("kws: initialized from {model_dir}");
  Some(Mutex::new(KwsEngine { spotter, stream }))
});

pub fn is_available() -> bool {
  ENGINE.is_some()
}

pub fn feed_samples(samples: &[i16]) -> bool {
  let Some(engine_mutex) = ENGINE.as_ref() else { return false };
  let engine = engine_mutex.lock().expect("lock poisoned");
  let f32_samples: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();
  engine.stream.accept_waveform(16000, &f32_samples);
  while engine.spotter.is_ready(&engine.stream) {
    engine.spotter.decode(&engine.stream);
  }
  if let Some(result) = engine.spotter.get_result(&engine.stream)
    && !result.keyword.is_empty()
  {
    return true;
  }
  false
}

pub fn reset_buffer() {
  if let Some(engine_mutex) = ENGINE.as_ref() {
    let engine = engine_mutex.lock().expect("lock poisoned");
    engine.spotter.reset(&engine.stream);
  }
}
