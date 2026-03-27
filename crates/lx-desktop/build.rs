use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
  let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
  let root = Path::new(&manifest_dir).parent().expect("crates dir").parent().expect("repo root");
  let dioxus_common = root.join("../dioxus-common");
  let assets = Path::new(&manifest_dir).join("assets");

  let widget_bridge_dir = dioxus_common.join("ts/widget-bridge");
  for dir in &["src", "widgets"] {
    let d = widget_bridge_dir.join(dir);
    if d.exists()
      && let Ok(entries) = fs::read_dir(&d)
    {
      for entry in entries.flatten() {
        if entry.path().extension().is_some_and(|e| e == "ts") {
          println!("cargo:rerun-if-changed={}", entry.path().display());
        }
      }
    }
  }
  for pkg in &["audio-playback", "audio-capture"] {
    let pkg_src = dioxus_common.join(format!("ts/{pkg}/src"));
    if pkg_src.exists()
      && let Ok(entries) = fs::read_dir(&pkg_src)
    {
      for entry in entries.flatten() {
        if entry.path().extension().is_some_and(|e| e == "ts") {
          println!("cargo:rerun-if-changed={}", entry.path().display());
        }
      }
    }
  }
  if widget_bridge_dir.join("package.json").exists() {
    let status = Command::new("pnpm").arg("build").current_dir(&widget_bridge_dir).status();
    match status {
      Ok(s) if s.success() => {},
      Ok(s) => eprintln!("cargo:warning=pnpm build for widget-bridge exited with {s}"),
      Err(e) => eprintln!("cargo:warning=pnpm build for widget-bridge not available: {e}"),
    }
  }

  let copies = [
    (widget_bridge_dir.join("dist/widget-bridge.js"), assets.join("widget-bridge.js")),
    (dioxus_common.join("crates/common-charts/assets/charts.js"), assets.join("charts.js")),
  ];

  for (src, dst) in &copies {
    if src.exists() {
      fs::copy(src, dst).unwrap_or_else(|e| panic!("failed to copy {}: {e}", src.display()));
    }
  }

  println!("cargo:rerun-if-changed=build.rs");
}
