mod api_client;
mod app;
mod components;
mod layout;
mod pages;
mod routes;
mod ws_client;

fn main() {
    dioxus::launch(app::App);
}
