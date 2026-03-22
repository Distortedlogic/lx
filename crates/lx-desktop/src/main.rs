#[cfg(feature = "server")]
fn main() {
  dioxus::serve(|| async {
    let dioxus_router = dioxus::server::router(lx_desktop::app::App);
    let app_router = lx_desktop::server::router();
    Ok(app_router.merge(dioxus_router))
  });
}

#[cfg(not(feature = "server"))]
fn main() {
  dioxus::launch(lx_desktop::app::App);
}
