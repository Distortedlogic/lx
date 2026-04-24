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
    rerun_if_tree_changed(&widget_bridge_dir.join(dir));
  }
  let widget_bridge_pkg = widget_bridge_dir.join("package.json");
  if widget_bridge_pkg.exists() {
    println!("cargo:rerun-if-changed={}", widget_bridge_pkg.display());
  }
  for pkg in &["audio-playback", "audio-capture"] {
    let pkg_src = dioxus_common.join(format!("ts/{pkg}/src"));
    rerun_if_tree_changed(&pkg_src);
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
  ];

  for (src, dst) in &copies {
    if src.exists() {
      fs::copy(src, dst).unwrap_or_else(|e| panic!("failed to copy {}: {e}", src.display()));
    }
  }

  println!("cargo:rerun-if-changed=build.rs");
}

fn rerun_if_tree_changed(dir: &Path) {
  if !dir.exists() {
    return;
  }

  let Ok(entries) = fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      rerun_if_tree_changed(&path);
    } else {
      println!("cargo:rerun-if-changed={}", path.display());
    }
  }
}
