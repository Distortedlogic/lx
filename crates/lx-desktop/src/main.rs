mod app;
mod layout;
mod pages;
mod routes;
mod server;

fn main() {
    dioxus::launch(app::App);
}
