use dioxus::prelude::*;

use crate::layout::shell::MobileShell;
use crate::pages::approvals::Approvals;
use crate::pages::events::Events;
use crate::pages::status::Status;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MobileShell)]
        #[route("/")]
        Status {},
        #[route("/events")]
        Events {},
        #[route("/approvals")]
        Approvals {},
}
