use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ClientToTerminal {
  Input(Vec<u8>),
  Resize { cols: u16, rows: u16 },
  Close,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerminalToClient {
  Output(Vec<u8>),
  SessionReady { cols: u16, rows: u16 },
  Closed,
  Error(String),
}
