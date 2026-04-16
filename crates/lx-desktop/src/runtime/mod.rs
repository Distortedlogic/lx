mod controller;
mod pi_backend;
mod pi_event_mapper;
mod registry;
pub mod types;

pub use controller::{DesktopRuntimeController, DesktopRuntimeProvider, use_desktop_runtime};
pub use registry::{DesktopRuntimeRegistry, status_label, tool_status_label};
pub use types::DesktopAgentLaunchSpec;
