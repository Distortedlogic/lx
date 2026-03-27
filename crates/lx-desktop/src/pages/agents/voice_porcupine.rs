#![allow(unsafe_code)]

use std::path::Path;
use std::sync::{LazyLock, Mutex};

use dioxus::logger::tracing::{info, warn};
use sherpa_onnx::{KeywordSpotter, KeywordSpotterConfig};

struct KwsEngine {
  spotter: KeywordSpotter,
  stream: sherpa_onnx::OnlineStream,
}

unsafe impl Send for KwsEngine {}

fn find_onnx(dir: &Path, prefix: &str) -> Option<String> {
  let int8 = std::fs::read_dir(dir)
    .ok()?
    .flatten()
    .find(|e| {
      let name = e.file_name();
      let n = name.to_string_lossy();
      n.starts_with(prefix) && n.ends_with(".int8.onnx")
    })
    .map(|e| e.path().to_string_lossy().into_owned());
  if int8.is_some() {
    return int8;
  }
  std::fs::read_dir(dir)
    .ok()?
    .flatten()
    .find(|e| {
      let name = e.file_name();
      let n = name.to_string_lossy();
      n.starts_with(prefix) && n.ends_with(".onnx")
    })
    .map(|e| e.path().to_string_lossy().into_owned())
}

static ENGINE: LazyLock<Option<Mutex<KwsEngine>>> = LazyLock::new(|| {
  let model_dir = std::env::var("SHERPA_KWS_MODEL_DIR").ok()?;
  let dir = Path::new(&model_dir);
  if !dir.is_dir() {
    warn!("kws: SHERPA_KWS_MODEL_DIR={model_dir} is not a directory");
    return None;
  }
  let encoder = find_onnx(dir, "encoder")?;
  let decoder = find_onnx(dir, "decoder")?;
  let joiner = find_onnx(dir, "joiner")?;
  let tokens = dir.join("tokens.txt");
  let keywords = dir.join("keywords.txt");
  if !tokens.exists() || !keywords.exists() {
    warn!("kws: missing tokens.txt or keywords.txt in {model_dir}");
    return None;
  }
  let mut config = KeywordSpotterConfig::default();
  config.model_config.transducer.encoder = Some(encoder);
  config.model_config.transducer.decoder = Some(decoder);
  config.model_config.transducer.joiner = Some(joiner);
  config.model_config.tokens = Some(tokens.to_string_lossy().into_owned());
  config.keywords_file = Some(keywords.to_string_lossy().into_owned());
  config.model_config.num_threads = 1;
  let Some(spotter) = KeywordSpotter::create(&config) else {
    warn!("kws: failed to create keyword spotter from {model_dir}");
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
