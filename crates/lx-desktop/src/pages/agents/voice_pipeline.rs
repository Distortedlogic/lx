use std::sync::LazyLock;

use super::voice_context::{PipelineStage, TranscriptEntry, VoiceContext, VoiceStatus};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use common_inference::InferenceClient as _;
use common_kokoro::SpeechRequest;
use common_whisper::TranscribeRequest;
use dioxus::logger::tracing::{error, info};
use dioxus::prelude::*;

pub static AUDIO_SINK: LazyLock<rodio::MixerDeviceSink> = LazyLock::new(|| {
  let mut sink = rodio::DeviceSinkBuilder::open_default_sink().unwrap_or_else(|e| panic!("audio device: {e}"));
  sink.log_on_drop(false);
  sink
});

pub async fn transcribe(pcm: &[u8]) -> anyhow::Result<String> {
  let wav = common_audio::wrap_pcm_as_wav(pcm, common_audio::SAMPLE_RATE, common_audio::CHANNELS, common_audio::BITS_PER_SAMPLE);
  let transcription = common_whisper::WHISPER.infer(&TranscribeRequest { audio_data: B64.encode(&wav), language: Some("en".into()) }).await?;
  Ok(transcription.text.trim().to_owned())
}

pub fn match_trigger(text: &str, triggers: &[String]) -> Option<String> {
  let text_words: Vec<&str> = text.split_whitespace().collect();
  let mut sorted: Vec<&String> = triggers.iter().collect();
  sorted.sort_by(|a, b| b.len().cmp(&a.len()));
  for trigger in sorted {
    let trigger_words: Vec<&str> = trigger.split_whitespace().collect();
    let n = trigger_words.len();
    if text_words.len() >= n {
      let matches = text_words[..n].iter().zip(trigger_words.iter()).all(|(a, b)| {
        let a_clean: String = a.chars().filter(|c| c.is_alphanumeric()).collect();
        let b_clean: String = b.chars().filter(|c| c.is_alphanumeric()).collect();
        a_clean.eq_ignore_ascii_case(&b_clean)
      });
      if matches {
        return Some(text_words[n..].join(" "));
      }
    }
  }
  None
}

pub fn generate_ack_tone() -> Vec<u8> {
  let sample_rate = 24000u32;
  let duration_samples = (sample_rate as f64 * 0.15) as usize;
  let fade_samples = (sample_rate as f64 * 0.01) as usize;
  let freq = 440.0f64;
  let mut pcm = Vec::with_capacity(duration_samples * 2);
  for i in 0..duration_samples {
    let t = i as f64 / sample_rate as f64;
    let mut sample = (t * freq * 2.0 * std::f64::consts::PI).sin() * 0.3;
    if i < fade_samples {
      sample *= i as f64 / fade_samples as f64;
    } else if i >= duration_samples - fade_samples {
      sample *= (duration_samples - 1 - i) as f64 / fade_samples as f64;
    }
    let s = (sample * 32767.0) as i16;
    pcm.extend_from_slice(&s.to_le_bytes());
  }
  common_audio::wrap_pcm_as_wav(&pcm, sample_rate, 1, 16)
}

pub fn play_wav(wav_bytes: Vec<u8>) -> tokio::task::JoinHandle<anyhow::Result<()>> {
  tokio::task::spawn_blocking(move || {
    let cursor = std::io::Cursor::new(wav_bytes);
    let player = rodio::play(AUDIO_SINK.mixer(), cursor).map_err(|e| {
      error!("voice: rodio::play failed: {e}");
      e
    })?;
    player.sleep_until_end();
    Ok(())
  })
}

async fn tts(text: &str) -> anyhow::Result<Vec<u8>> {
  let req = SpeechRequest { text: text.to_owned(), voice: "am_michael".into(), lang_code: "a".into(), speed: 1.2 };
  common_kokoro::KOKORO.infer(&req).await
}

pub async fn run_pipeline(
  text: &str,
  _voice_widget: dioxus_widget_bridge::TsWidgetHandle,
  agent_widget: dioxus_widget_bridge::TsWidgetHandle,
  mut ctx: VoiceContext,
) -> anyhow::Result<()> {
  ctx.transcript.write().push(TranscriptEntry { is_user: true, text: text.to_owned() });

  agent_widget.send_update(serde_json::json!({ "type": "user_display", "text": text }));

  ctx.pipeline_stage.set(PipelineStage::QueryingLlm);
  let response = crate::voice_backend::query_streaming(text, |chunk| {
    agent_widget.send_update(serde_json::json!({ "type": "assistant_chunk", "text": chunk }));
  })
  .await?;
  agent_widget.send_update(serde_json::json!({ "type": "assistant_done" }));

  if response.is_empty() {
    ctx.pipeline_stage.set(PipelineStage::Idle);
    return Ok(());
  }

  ctx.pipeline_stage.set(PipelineStage::SynthesizingSpeech);
  let wav_bytes = tts(&response).await?;

  ctx.status.set(VoiceStatus::Speaking);

  let wav_len = wav_bytes.len();
  info!("voice: TTS returned {wav_len} bytes, starting playback");
  let play_result = play_wav(wav_bytes).await;
  match &play_result {
    Ok(Ok(())) => info!("voice: playback finished"),
    Ok(Err(e)) => error!("voice: playback error: {e}"),
    Err(e) => error!("voice: spawn_blocking panicked: {e}"),
  }
  play_result??;

  let transcript_entry = response;
  let mut t = ctx.transcript.write();
  match t.last_mut() {
    Some(entry) if !entry.is_user => entry.text.push_str(&format!(" {transcript_entry}")),
    _ => t.push(TranscriptEntry { is_user: false, text: transcript_entry }),
  }

  ctx.pipeline_stage.set(PipelineStage::Idle);
  Ok(())
}
