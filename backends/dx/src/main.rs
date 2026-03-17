mod app;
mod backends;
mod components;
mod event;
mod langfuse;
mod runner;

use app::App;

fn main() {
    dioxus::launch(App);
}
