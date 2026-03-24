use std::process::Command;

#[test]
fn lx_test_suite() {
  let output = Command::new(env!("CARGO_BIN_EXE_lx"))
    .args(["test", "tests"])
    .current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/../..")
    .output()
    .expect("failed to execute lx test");

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  let has_pass = stdout.contains("passed");
  let zero_passed = stdout.contains("0 passed");

  if !has_pass || zero_passed {
    panic!("lx test suite found no passing tests (exit code {:?}):\nstdout:\n{}\nstderr:\n{}", output.status.code(), stdout, stderr);
  }
}
