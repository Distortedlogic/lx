use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::accounts::Accounts;
use crate::pages::activity::Activity;
use crate::pages::agent_detail::AgentDetail;
use crate::pages::agents::Agents;
use crate::pages::approvals::{ApprovalDetail, Approvals};
use crate::pages::companies::Companies;
use crate::pages::company_export::CompanyExport;
use crate::pages::company_import::CompanyImport;
use crate::pages::company_skills::CompanySkills;
use crate::pages::costs::Costs;
use crate::pages::dashboard::{Dashboard, DashboardAlt};
use crate::pages::flows::{FlowDetail, Flows};
use crate::pages::goals::{GoalDetail, Goals};
use crate::pages::inbox::Inbox;
use crate::pages::issues::{IssueDetail, Issues};
use crate::pages::not_found::NotFound;
use crate::pages::onboarding::Onboarding;
use crate::pages::org::OrgChart;
use crate::pages::plugins::{PluginManager, PluginPage, PluginSettingsPage};
use crate::pages::projects::{ProjectDetail, Projects};
use crate::pages::routines::{RoutineDetail, Routines};
use crate::pages::settings::{CompanySettings, InstanceHeartbeats as InstanceSettings, Settings};
use crate::pages::tools::{PiAgentPage, PiPage, Tools};

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Dashboard {},
        #[route("/dashboard")]
        DashboardAlt {},
        #[route("/agents")]
        Agents {},
        #[route("/agents/:agent_id")]
        AgentDetail { agent_id: String },
        #[route("/projects")]
        Projects {},
        #[route("/projects/:project_id")]
        ProjectDetail { project_id: String },
        #[route("/issues")]
        Issues {},
        #[route("/issues/:issue_id")]
        IssueDetail { issue_id: String },
        #[route("/goals")]
        Goals {},
        #[route("/goals/:goal_id")]
        GoalDetail { goal_id: String },
        #[route("/approvals")]
        Approvals {},
        #[route("/approvals/:approval_id")]
        ApprovalDetail { approval_id: String },
        #[route("/routines")]
        Routines {},
        #[route("/routines/:routine_id")]
        RoutineDetail { routine_id: String },
        #[route("/costs")]
        Costs {},
        #[route("/activity")]
        Activity {},
        #[route("/inbox")]
        Inbox {},
        #[route("/flows")]
        Flows {},
        #[route("/flows/:flow_id")]
        FlowDetail { flow_id: String },
        #[route("/org")]
        OrgChart {},
        #[route("/tools")]
        Tools {},
        #[route("/tools/pi")]
        PiPage {},
        #[route("/tools/pi/:agent_id")]
        PiAgentPage { agent_id: String },
        #[route("/settings")]
        Settings {},
        #[route("/accounts")]
        Accounts {},
        #[route("/company/settings")]
        CompanySettings {},
        #[route("/instance/settings")]
        InstanceSettings {},
        #[route("/companies")]
        Companies {},
        #[route("/company/export")]
        CompanyExport {},
        #[route("/company/import")]
        CompanyImport {},
        #[route("/skills")]
        CompanySkills {},
        #[route("/onboarding")]
        Onboarding {},
        #[route("/plugins")]
        PluginManager {},
        #[route("/plugins/:plugin_id")]
        PluginPage { plugin_id: String },
        #[route("/plugins/:plugin_id/settings")]
        PluginSettingsPage { plugin_id: String },
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}
