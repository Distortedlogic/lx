use std::path::Path;
use std::process::Command;

fn main() {
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
  let root = Path::new(&manifest_dir).parent().expect("crates dir").parent().expect("repo root");
  let assets = Path::new(&manifest_dir).join("assets");

  let ts_packages = [("widget-bridge", &["src", "widgets"][..]), ("dx-charts", &["src"][..])];

  for (pkg, source_dirs) in &ts_packages {
    let pkg_dir = root.join(format!("ts/{pkg}"));
    for dir in *source_dirs {
      println!("cargo:rerun-if-changed={}", pkg_dir.join(dir).display());
    }
    if pkg_dir.join("package.json").exists() {
      let status = Command::new("pnpm").arg("build").current_dir(&pkg_dir).status();
      match status {
        Ok(s) if s.success() => {},
        Ok(s) => eprintln!("cargo:warning=pnpm build for {pkg} exited with {s}"),
        Err(e) => eprintln!("cargo:warning=pnpm build for {pkg} not available: {e}"),
      }
    }
  }

  let copies = [
    (root.join("ts/widget-bridge/dist/widget-bridge.js"), assets.join("widget-bridge.js")),
    (root.join("ts/dx-charts/dist/dx-charts.js"), assets.join("dx-charts.js")),
  ];

  for (src, dst) in &copies {
    if src.exists() {
      std::fs::copy(src, dst).unwrap_or_else(|e| panic!("failed to copy {}: {e}", src.display()));
    }
  }

  println!("cargo:rerun-if-changed=build.rs");
}
