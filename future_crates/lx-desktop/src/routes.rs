use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::events::Events;
use crate::pages::run::Run;
use crate::pages::terminals::Terminals;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Run {},
        #[route("/terminals")]
        Terminals {},
        #[route("/events")]
        Events {},
}
