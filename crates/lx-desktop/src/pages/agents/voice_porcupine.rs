use std::sync::{LazyLock, Mutex};

use dioxus::logger::tracing::{info, warn};
use rustpotter::{AudioFmt, Endianness, Rustpotter, RustpotterConfig, SampleFormat};

struct DetectorState {
  detector: Rustpotter,
  frame_size: usize,
  buffer: Vec<i16>,
}

static ENGINE: LazyLock<Option<Mutex<DetectorState>>> = LazyLock::new(|| {
  let wakeword_path = std::env::var("RUSTPOTTER_WAKEWORD_PATH").ok()?;
  if !std::path::Path::new(&wakeword_path).exists() {
    warn!("rustpotter: wakeword file not found: {wakeword_path}");
    return None;
  }
  let config = RustpotterConfig {
    fmt: AudioFmt { sample_rate: 16000, sample_format: SampleFormat::I16, channels: 1, endianness: Endianness::Little },
    detector: rustpotter::DetectorConfig { threshold: 0.5, avg_threshold: 0.2, min_scores: 5, ..Default::default() },
    filters: rustpotter::FiltersConfig { gain_normalizer: rustpotter::GainNormalizationConfig { enabled: true, ..Default::default() }, ..Default::default() },
  };
  let mut detector = match Rustpotter::new(&config) {
    Ok(d) => d,
    Err(e) => {
      warn!("rustpotter: failed to create detector: {e}");
      return None;
    },
  };
  if let Err(e) = detector.add_wakeword_from_file("wakeword", &wakeword_path) {
    warn!("rustpotter: failed to load wakeword from {wakeword_path}: {e}");
    return None;
  }
  let frame_size = detector.get_samples_per_frame();
  info!("rustpotter: initialized, frame_size={frame_size}, wakeword={wakeword_path}");
  Some(Mutex::new(DetectorState { detector, frame_size, buffer: Vec::new() }))
});

pub fn is_available() -> bool {
  ENGINE.is_some()
}

pub fn feed_samples(samples: &[i16]) -> bool {
  let Some(engine) = ENGINE.as_ref() else { return false };
  let mut state = engine.lock().expect("lock poisoned");
  state.buffer.extend_from_slice(samples);
  let frame_size = state.frame_size;
  let mut detected = false;
  while state.buffer.len() >= frame_size {
    let frame: Vec<i16> = state.buffer.drain(..frame_size).collect();
    if state.detector.process_samples(frame).is_some() {
      detected = true;
    }
  }
  detected
}

pub fn reset_buffer() {
  if let Some(engine) = ENGINE.as_ref() {
    let mut state = engine.lock().expect("lock poisoned");
    state.buffer.clear();
    state.detector.reset();
  }
}
