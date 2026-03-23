use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::activity::Activity;
use crate::pages::agents::Agents;
use crate::pages::dashboard::Dashboard;
use crate::pages::files::Files;
use crate::pages::repos::Repos;
use crate::pages::search::Search;
use crate::pages::settings::Settings;
use crate::pages::tasks::Tasks;
use crate::pages::terminals::Terminals;
use crate::pages::voice::Voice;
use crate::pages::workspaces::Workspaces;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Terminals {},
        #[route("/agents")]
        Agents {},
        #[route("/activity")]
        Activity {},
        #[route("/tasks")]
        Tasks {},
        #[route("/workspaces")]
        Workspaces {},
        #[route("/voice")]
        Voice {},
        #[route("/dashboard")]
        Dashboard {},
        #[route("/repos")]
        Repos {},
        #[route("/search")]
        Search {},
        #[route("/files")]
        Files {},
        #[route("/settings")]
        Settings {},
}
