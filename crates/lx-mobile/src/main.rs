mod app;
mod layout;
mod pages;
mod routes;

fn main() {
  dioxus::fullstack::set_server_url("http://127.0.0.1:8080");
  dioxus::launch(app::App);
}
