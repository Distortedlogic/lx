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
  if std::env::var("RUST_LOG").is_err() {
    std::env::set_var("RUST_LOG", "info,chromiumoxide=error,chromiumoxide::conn=error");
  }
  let builder = dioxus::LaunchBuilder::new();
  #[cfg(feature = "desktop")]
  let builder = builder.with_cfg(
    dioxus::desktop::Config::new().with_window(dioxus::desktop::WindowBuilder::new().with_decorations(false).with_resizable(true).with_title("lx desktop")),
  );
  builder.launch(lx_desktop::app::App);
}
