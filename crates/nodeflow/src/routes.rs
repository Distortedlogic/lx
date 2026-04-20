use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::flows::{FlowDetail, Flows};
use crate::pages::not_found::NotFound;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Flows {},
        #[route("/flows/:flow_id")]
        FlowDetail { flow_id: String },
        #[route("/:..segments")]
        NotFound { segments: Vec<String> },
}
