use std::path::Path;
use std::process::Command;

fn main() {
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
  let root = Path::new(&manifest_dir).parent().expect("crates dir").parent().expect("repo root");
  let assets = Path::new(&manifest_dir).join("assets");

  let copies = [
    (root.join("ts/widget-bridge/dist/widget-bridge.js"), assets.join("widget-bridge.js")),
    (root.join("ts/dx-charts/dist/dx-charts.js"), assets.join("dx-charts.js")),
  ];

  for (src, dst) in &copies {
    if src.exists() {
      std::fs::copy(src, dst).unwrap_or_else(|e| panic!("failed to copy {}: {e}", src.display()));
    }
    println!("cargo:rerun-if-changed={}", src.display());
  }

  let input_css = Path::new(&manifest_dir).join("src/tailwind.css");
  let output_css = assets.join("tailwind.css");
  let status = Command::new("pnpm").args(["exec", "tailwindcss", "--input"]).arg(&input_css).arg("--output").arg(&output_css).current_dir(root).status();
  match status {
    Ok(s) if s.success() => {},
    Ok(s) => eprintln!("cargo:warning=tailwindcss exited with {s}"),
    Err(e) => eprintln!("cargo:warning=tailwindcss not available: {e}"),
  }

  println!("cargo:rerun-if-changed=src/tailwind.css");
  println!("cargo:rerun-if-changed=build.rs");
}
