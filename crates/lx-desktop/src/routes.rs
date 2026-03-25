use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::accounts::Accounts;
use crate::pages::activity::Activity;
use crate::pages::agents::Agents;
use crate::pages::settings::Settings;
use crate::pages::tools::Tools;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Agents {},
        #[route("/activity")]
        Activity {},
        #[route("/tools")]
        Tools {},
        #[route("/settings")]
        Settings {},
        #[route("/accounts")]
        Accounts {},
}
