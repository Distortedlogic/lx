use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use axum::extract::ws::{Message as WsMessage, WebSocket};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use futures::{FutureExt, SinkExt, StreamExt};
use kokoro_client::{InferenceClient, SpeechRequest};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use whisper_client::TranscribeRequest;

use crate::backend::AgentBackend;
use crate::detector::{self, TriggerDetector, strip_triggers};
use crate::types::{ClientMessage, ServerMessage, SessionState, VoiceSession};

const MAX_COMMAND_BYTES: usize = 16000 * 2 * 60;

enum ChunkAction {
    Continue,
    ChannelClosed,
    ProcessCommand(Vec<u8>),
}

pub async fn handle_session(ws: WebSocket, agent: Arc<dyn AgentBackend>) {
    let (mut sender, mut receiver) = ws.split();
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(32);

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if let Err(e) = sender.send(WsMessage::Text(json.into())).await {
                        tracing::warn!("ws send error: {e}");
                        break;
                    }
                }
                Err(e) => tracing::warn!("json serialize error: {e}"),
            }
        }
    });

    let mut session =
        VoiceSession { id: uuid::Uuid::new_v4(), state: SessionState::Idle };
    let mut detector = TriggerDetector::new();
    let mut command_buffer: Vec<u8> = Vec::new();
    tracing::info!("voice session started: {}", session.id);

    match whisper_client::WHISPER.health().await {
        Ok(true) => {}
        Ok(false) => {
            let _ = tx
                .send(ServerMessage::Error {
                    message: "Whisper service is not reachable".into(),
                })
                .await;
        }
        Err(e) => {
            let _ = tx
                .send(ServerMessage::Error {
                    message: format!("Whisper health check failed: {e}"),
                })
                .await;
        }
    }
    match kokoro_client::KOKORO.health().await {
        Ok(true) => {}
        Ok(false) => {
            let _ = tx
                .send(ServerMessage::Error {
                    message: "Kokoro TTS service is not reachable".into(),
                })
                .await;
        }
        Err(e) => {
            let _ = tx
                .send(ServerMessage::Error {
                    message: format!("Kokoro health check failed: {e}"),
                })
                .await;
        }
    }

    let (cmd_result_tx, mut cmd_result_rx) = mpsc::channel::<anyhow::Result<()>>(1);
    let mut command_cancel: Option<CancellationToken> = None;

    loop {
        tokio::select! {
            ws_msg = receiver.next() => {
                let Some(ws_msg) = ws_msg else { break };
                let ws_msg = match ws_msg {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("ws receive error: {e}");
                        break;
                    }
                };

                match ws_msg {
                    WsMessage::Text(text) => {
                        let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) else {
                            continue;
                        };

                        match client_msg {
                            ClientMessage::StartStandby => {
                                session.state = SessionState::Standby;
                                detector.reset();
                                command_buffer.clear();
                            }
                            ClientMessage::StopStandby => {
                                session.state = SessionState::Idle;
                                detector.reset();
                                command_buffer.clear();
                            }
                            ClientMessage::AudioChunk { data, .. } => {
                                match STANDARD.decode(&data) {
                                    Ok(bytes) => {
                                        match handle_audio_chunk(
                                            &bytes,
                                            &mut session,
                                            &mut detector,
                                            &mut command_buffer,
                                            &tx,
                                        ).await {
                                            ChunkAction::Continue => {}
                                            ChunkAction::ChannelClosed => break,
                                            ChunkAction::ProcessCommand(audio) => {
                                                let cmd_cancel = CancellationToken::new();
                                                command_cancel = Some(cmd_cancel.clone());
                                                let a = Arc::clone(&agent);
                                                let t = tx.clone();
                                                let crt = cmd_result_tx.clone();
                                                tokio::spawn(async move {
                                                    let result = AssertUnwindSafe(
                                                        process_command(audio, a, t, cmd_cancel),
                                                    )
                                                    .catch_unwind()
                                                    .await
                                                    .unwrap_or_else(|_| {
                                                        Err(anyhow::anyhow!("command processing panicked"))
                                                    });
                                                    let _ = crt.send(result).await;
                                                });
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        if tx
                                            .send(ServerMessage::Error {
                                                message: format!("base64 decode error: {e}"),
                                            })
                                            .await
                                            .is_err()
                                        {
                                            break;
                                        }
                                    }
                                }
                            }
                            ClientMessage::Cancel => {
                                if let Some(cc) = command_cancel.take() {
                                    cc.cancel();
                                }
                                command_buffer.clear();
                                detector.reset();
                                session.state = SessionState::Idle;
                            }
                            ClientMessage::PlaybackComplete => {
                                if matches!(session.state, SessionState::Speaking) {
                                    session.state = SessionState::Standby;
                                    detector.reset();
                                    command_buffer.clear();
                                }
                            }
                            ClientMessage::Ping => {
                                if tx.send(ServerMessage::Pong).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    WsMessage::Close(_) => break,
                    WsMessage::Binary(_) | WsMessage::Ping(_) | WsMessage::Pong(_) => {}
                }
            }
            Some(result) = cmd_result_rx.recv() => {
                command_cancel = None;
                match result {
                    Ok(()) => {
                        session.state = SessionState::Speaking;
                        if tx.send(ServerMessage::StandbyResumed).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx
                            .send(ServerMessage::Error { message: format!("{e}") })
                            .await;
                        session.state = SessionState::Standby;
                    }
                }
            }
        }
    }

    if let Some(cc) = command_cancel.take() {
        cc.cancel();
    }
    drop(cmd_result_tx);
}

async fn handle_audio_chunk(
    pcm: &[u8],
    session: &mut VoiceSession,
    detector: &mut TriggerDetector,
    command_buffer: &mut Vec<u8>,
    tx: &mpsc::Sender<ServerMessage>,
) -> ChunkAction {
    match session.state {
        SessionState::Standby => {
            detector.feed(pcm);
            if detector.should_check() {
                match detector
                    .check_trigger(detector::TRIGGER_ACTIVATE, &whisper_client::WHISPER)
                    .await
                {
                    Ok(true) => {
                        session.state = SessionState::Activated;
                        *command_buffer = detector.take_buffer();
                        if tx.send(ServerMessage::Activated).await.is_err() {
                            return ChunkAction::ChannelClosed;
                        }
                    }
                    Ok(false) => {}
                    Err(e) => tracing::warn!("trigger check error: {e}"),
                }
            }
        }
        SessionState::Activated => {
            command_buffer.extend_from_slice(pcm);
            detector.feed(pcm);

            let trigger_fired = if command_buffer.len() >= MAX_COMMAND_BYTES {
                true
            } else if detector.should_check() {
                detector
                    .check_trigger(detector::TRIGGER_RESPOND, &whisper_client::WHISPER)
                    .await
                    .unwrap_or(false)
            } else {
                false
            };

            if trigger_fired {
                session.state = SessionState::Processing;
                let audio = std::mem::take(command_buffer);
                detector.reset();
                return ChunkAction::ProcessCommand(audio);
            }
        }
        _ => {}
    }
    ChunkAction::Continue
}

async fn process_command(
    audio: Vec<u8>,
    agent: Arc<dyn AgentBackend>,
    tx: mpsc::Sender<ServerMessage>,
    cancel: CancellationToken,
) -> anyhow::Result<()> {
    let wav = audio_core::wrap_pcm_as_wav(&audio, 16000, 1, 16);
    let audio_data = STANDARD.encode(&wav);
    let req = TranscribeRequest { audio_data, language: None };
    let transcription = whisper_client::WHISPER.infer(&req).await?;
    if cancel.is_cancelled() {
        return Ok(());
    }

    let clean_text = strip_triggers(&transcription.text);
    tx.send(ServerMessage::TextTranscript { text: clean_text.clone() })
        .await
        .map_err(|e| anyhow::anyhow!("channel closed: {e}"))?;

    if clean_text.is_empty() {
        return Ok(());
    }

    let response = tokio::time::timeout(
        tokio::time::Duration::from_secs(120),
        agent.query(&clean_text),
    )
    .await
    .map_err(|_| anyhow::anyhow!("voice query timed out after 120s"))??;
    if cancel.is_cancelled() {
        return Ok(());
    }

    tx.send(ServerMessage::AgentResponse { text: response.clone() })
        .await
        .map_err(|e| anyhow::anyhow!("channel closed: {e}"))?;

    let speech_req = SpeechRequest {
        text: response,
        voice: "af_heart".into(),
        lang_code: "a".into(),
        speed: 1.0,
    };
    let wav = kokoro_client::KOKORO.infer(&speech_req).await?;
    if cancel.is_cancelled() {
        return Ok(());
    }

    let chunks = audio_core::chunk_wav(&wav, 32768);
    for (seq, chunk) in chunks.into_iter().enumerate() {
        let data = STANDARD.encode(&chunk);
        tx.send(ServerMessage::AudioResponse { data, seq: seq as u64 })
            .await
            .map_err(|e| anyhow::anyhow!("channel closed: {e}"))?;
    }
    Ok(())
}
