use std::ops::ControlFlow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitAction {
  Descend,
  Skip,
  Stop,
}

impl VisitAction {
  pub fn is_stop(self) -> bool {
    self == VisitAction::Stop
  }

  pub fn to_control_flow(self) -> ControlFlow<()> {
    match self {
      VisitAction::Stop => ControlFlow::Break(()),
      _ => ControlFlow::Continue(()),
    }
  }
}
