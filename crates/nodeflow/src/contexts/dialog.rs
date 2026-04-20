use dioxus::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct NewIssueDefaults {
  pub status: Option<String>,
  pub priority: Option<String>,
  pub project_id: Option<String>,
  pub assignee_agent_id: Option<String>,
  pub title: Option<String>,
  pub description: Option<String>,
}

#[derive(Clone, Copy)]
pub struct DialogState {
  pub new_issue_open: Signal<bool>,
  pub new_issue_defaults: Signal<NewIssueDefaults>,
  pub new_project_open: Signal<bool>,
  pub new_agent_open: Signal<bool>,
  pub onboarding_open: Signal<bool>,
}

impl DialogState {
  pub fn provide() -> Self {
    let state = Self {
      new_issue_open: Signal::new(false),
      new_issue_defaults: Signal::new(NewIssueDefaults::default()),
      new_project_open: Signal::new(false),
      new_agent_open: Signal::new(false),
      onboarding_open: Signal::new(false),
    };
    use_context_provider(|| state);
    state
  }

  pub fn open_new_issue(&self, defaults: NewIssueDefaults) {
    let mut d = self.new_issue_defaults;
    d.set(defaults);
    let mut o = self.new_issue_open;
    o.set(true);
  }

  pub fn close_new_issue(&self) {
    let mut o = self.new_issue_open;
    o.set(false);
    let mut d = self.new_issue_defaults;
    d.set(NewIssueDefaults::default());
  }

  pub fn open_new_project(&self) {
    let mut o = self.new_project_open;
    o.set(true);
  }

  pub fn close_new_project(&self) {
    let mut o = self.new_project_open;
    o.set(false);
  }

  pub fn open_new_agent(&self) {
    let mut o = self.new_agent_open;
    o.set(true);
  }

  pub fn close_new_agent(&self) {
    let mut o = self.new_agent_open;
    o.set(false);
  }

  pub fn open_onboarding(&self) {
    let mut o = self.onboarding_open;
    o.set(true);
  }

  pub fn close_onboarding(&self) {
    let mut o = self.onboarding_open;
    o.set(false);
  }
}
