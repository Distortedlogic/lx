use dioxus::prelude::*;

use crate::layout::shell::Shell;
use crate::pages::terminals::Terminals;

#[derive(Clone, Routable, Debug, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Terminals {},
}
