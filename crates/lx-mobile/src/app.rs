use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
