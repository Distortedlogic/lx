#[cfg(feature = "server")]
fn main() {
  use dioxus::prelude::DioxusRouterExt;
  dioxus::serve(|| async {
    let cfg = dioxus::server::ServeConfig::builder();
    Ok(axum::Router::new().serve_dioxus_application(cfg, lx_desktop::app::App))
  });
}

#[cfg(not(feature = "server"))]
fn main() {
  let builder = dioxus::LaunchBuilder::new();
  #[cfg(feature = "desktop")]
  let builder = builder.with_cfg(
    dioxus::desktop::Config::new().with_window(dioxus::desktop::WindowBuilder::new().with_decorations(false).with_resizable(true).with_title("lx desktop")),
  );
  builder.launch(lx_desktop::app::App);
}
