use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
  AudioChunk { data: String, seq: u64 },
  StartStandby,
  StopStandby,
  Cancel,
  PlaybackComplete,
  Ping,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
  AudioResponse { data: String, seq: u64 },
  TextTranscript { text: String },
  AgentResponse { text: String },
  Activated,
  StandbyResumed,
  Pong,
  Error { message: String },
}

pub enum SessionState {
  Idle,
  Standby,
  Activated,
  Processing,
  Speaking,
}

pub struct VoiceSession {
  pub id: Uuid,
  pub state: SessionState,
}
