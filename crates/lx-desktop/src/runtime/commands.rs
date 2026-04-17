use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopRuntimeCommand {
  Prompt { message: String },
  Steer { message: String },
  FollowUp { message: String },
  Abort,
  Pause,
  Resume,
  RefreshState,
}

impl DesktopRuntimeCommand {
  pub fn label(&self) -> &'static str {
    match self {
      Self::Prompt { .. } => "prompt",
      Self::Steer { .. } => "steer",
      Self::FollowUp { .. } => "follow_up",
      Self::Abort => "abort",
      Self::Pause => "pause",
      Self::Resume => "resume",
      Self::RefreshState => "get_state",
    }
  }
}

pub fn command_message(command: &DesktopRuntimeCommand) -> Option<&str> {
  match command {
    DesktopRuntimeCommand::Prompt { message } | DesktopRuntimeCommand::Steer { message } | DesktopRuntimeCommand::FollowUp { message } => Some(message),
    DesktopRuntimeCommand::Abort | DesktopRuntimeCommand::Pause | DesktopRuntimeCommand::Resume | DesktopRuntimeCommand::RefreshState => None,
  }
}
