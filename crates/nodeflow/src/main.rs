#[cfg(feature = "server")]
fn main() {
  dioxus::launch(nodeflow::app::App);
}

#[cfg(not(feature = "server"))]
fn main() {
  let builder = dioxus::LaunchBuilder::new();
  #[cfg(feature = "desktop")]
  let builder = builder.with_cfg(
    dioxus::desktop::Config::new().with_window(dioxus::desktop::WindowBuilder::new().with_decorations(false).with_resizable(true).with_title("nodeflow")),
  );
  builder.launch(nodeflow::app::App);
}
