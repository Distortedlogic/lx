use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::accounts::Accounts;
use crate::pages::activity::Activity;
use crate::pages::agents::Agents;
use crate::pages::repos::Repos;
use crate::pages::settings::Settings;
use crate::pages::terminals::Terminals;

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
        #[route("/repos")]
        Repos {},
        #[route("/settings")]
        Settings {},
        #[route("/accounts")]
        Accounts {},
}
