mod app;
mod layout;
mod pages;
mod routes;
mod server;
mod terminal;
mod ts_widget;

fn main() {
  dioxus::launch(app::App);
}
