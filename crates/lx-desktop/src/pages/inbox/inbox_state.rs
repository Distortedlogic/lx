#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InboxTab {
  Mine,
  Recent,
  All,
  Unread,
}

impl InboxTab {
  pub fn label(&self) -> &'static str {
    match self {
      Self::Mine => "Mine",
      Self::Recent => "Recent",
      Self::All => "All",
      Self::Unread => "Unread",
    }
  }

  pub fn all() -> &'static [InboxTab] {
    &[Self::Mine, Self::Recent, Self::All, Self::Unread]
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InboxCategoryFilter {
  Everything,
  IssuesTouched,
  JoinRequests,
  Approvals,
  FailedRuns,
  Alerts,
}

impl InboxCategoryFilter {
  pub fn label(&self) -> &'static str {
    match self {
      Self::Everything => "Everything",
      Self::IssuesTouched => "Issues I Touched",
      Self::JoinRequests => "Join Requests",
      Self::Approvals => "Approvals",
      Self::FailedRuns => "Failed Runs",
      Self::Alerts => "Alerts",
    }
  }

  pub fn all() -> &'static [InboxCategoryFilter] {
    &[Self::Everything, Self::IssuesTouched, Self::JoinRequests, Self::Approvals, Self::FailedRuns, Self::Alerts]
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApprovalStatus {
  Pending,
}

impl std::fmt::Display for ApprovalStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Pending => write!(f, "pending"),
    }
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxApprovalItem {
  pub id: String,
  pub approval_type: String,
  pub status: ApprovalStatus,
  pub requester_name: Option<String>,
  pub updated_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxFailedRun {
  pub id: String,
  pub agent_id: String,
  pub agent_name: Option<String>,
  pub error_message: String,
  pub status: String,
  pub created_at: String,
  pub issue_id: Option<String>,
  pub issue_title: Option<String>,
  pub issue_identifier: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxJoinRequest {
  pub id: String,
  pub request_type: String,
  pub agent_name: Option<String>,
  pub adapter_type: Option<String>,
  pub request_ip: String,
  pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboxIssueItem {
  pub id: String,
  pub identifier: Option<String>,
  pub title: String,
  pub status: String,
  pub is_unread: bool,
  pub updated_at: String,
}
