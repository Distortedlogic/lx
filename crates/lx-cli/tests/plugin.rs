use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn lx_bin() -> PathBuf {
  PathBuf::from(env!("CARGO_BIN_EXE_lx"))
}

fn make_temp_dir(label: &str) -> PathBuf {
  let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).expect("system time").as_nanos();
  let dir = PathBuf::from(format!("/tmp/lx_plugin_test_{}_{}_{}", label, std::process::id(), nanos));
  let _ = fs::remove_dir_all(&dir);
  fs::create_dir_all(&dir).expect("create temp dir");
  dir
}

fn run_plugin(args: &[&str], home_override: &str, cwd: Option<&str>) -> (bool, String, String) {
  let mut all_args = vec!["plugin"];
  all_args.extend_from_slice(args);
  let mut cmd = Command::new(lx_bin());
  cmd.args(&all_args).env("HOME", home_override);
  if let Some(dir) = cwd {
    cmd.current_dir(dir);
  }
  let output = cmd.output().expect("failed to run lx plugin");
  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();
  (output.status.success(), stdout, stderr)
}

fn make_fake_plugin(dir: &std::path::Path, name: &str, version: &str) {
  fs::create_dir_all(dir).expect("create plugin dir");
  fs::write(dir.join("fake.wasm"), b"fake wasm").expect("write wasm");
  let manifest = format!(
    "[plugin]\nname = \"{name}\"\nversion = \"{version}\"\ndescription = \"A test plugin\"\nwasm = \"fake.wasm\"\n\n[exports]\nhello = {{ arity = 1 }}\n"
  );
  fs::write(dir.join("plugin.toml"), manifest).expect("write manifest");
}

fn cleanup(dir: &std::path::Path) {
  let _ = fs::remove_dir_all(dir);
}

#[test]
fn install_list_remove_cycle() {
  let tmp = make_temp_dir("cycle");
  let fake_home = tmp.join("home");
  fs::create_dir_all(&fake_home).expect("create fake home");
  let plugin_src = tmp.join("my-plugin-src");
  make_fake_plugin(&plugin_src, "test-plugin", "0.1.0");

  let (ok, stdout, _) = run_plugin(&["install", plugin_src.to_str().unwrap()], fake_home.to_str().unwrap(), None);
  assert!(ok, "install should succeed");
  assert!(stdout.contains("installed test-plugin 0.1.0"));

  let installed_dir = fake_home.join(".lx").join("plugins").join("test-plugin");
  assert!(installed_dir.exists());
  assert!(installed_dir.join("plugin.toml").exists());
  assert!(installed_dir.join("fake.wasm").exists());

  let (ok, stdout, _) = run_plugin(&["list"], fake_home.to_str().unwrap(), None);
  assert!(ok, "list should succeed");
  assert!(stdout.contains("test-plugin"));
  assert!(stdout.contains("0.1.0"));
  assert!(stdout.contains("global"));

  let (ok, stdout, _) = run_plugin(&["remove", "test-plugin"], fake_home.to_str().unwrap(), None);
  assert!(ok, "remove should succeed");
  assert!(stdout.contains("removed test-plugin"));
  assert!(!installed_dir.exists());

  let (ok, stdout, _) = run_plugin(&["list"], fake_home.to_str().unwrap(), None);
  assert!(ok, "list should succeed when empty");
  assert!(stdout.contains("no plugins installed"));

  cleanup(&tmp);
}

#[test]
fn install_update_overwrites() {
  let tmp = make_temp_dir("update");
  let fake_home = tmp.join("home");
  fs::create_dir_all(&fake_home).expect("create fake home");

  let v1_src = tmp.join("v1");
  make_fake_plugin(&v1_src, "upd-plugin", "1.0.0");
  let (ok, _, _) = run_plugin(&["install", v1_src.to_str().unwrap()], fake_home.to_str().unwrap(), None);
  assert!(ok);

  let v2_src = tmp.join("v2");
  make_fake_plugin(&v2_src, "upd-plugin", "2.0.0");
  let (ok, stdout, stderr) = run_plugin(&["install", v2_src.to_str().unwrap()], fake_home.to_str().unwrap(), None);
  assert!(ok, "update install should succeed");
  assert!(stderr.contains("updating upd-plugin 1.0.0 → 2.0.0"));
  assert!(stdout.contains("installed upd-plugin 2.0.0"));

  cleanup(&tmp);
}

#[test]
fn remove_nonexistent_errors() {
  let tmp = make_temp_dir("rm_missing");
  let fake_home = tmp.join("home");
  fs::create_dir_all(&fake_home).expect("create fake home");
  let (ok, _, stderr) = run_plugin(&["remove", "nonexistent"], fake_home.to_str().unwrap(), None);
  assert!(!ok, "remove nonexistent should fail");
  assert!(stderr.contains("not found"));
  cleanup(&tmp);
}

#[test]
fn install_missing_manifest_errors() {
  let tmp = make_temp_dir("no_manifest");
  let fake_home = tmp.join("home");
  fs::create_dir_all(&fake_home).expect("create fake home");
  let empty_dir = tmp.join("empty-plugin");
  fs::create_dir_all(&empty_dir).expect("create empty dir");
  let (ok, _, stderr) = run_plugin(&["install", empty_dir.to_str().unwrap()], fake_home.to_str().unwrap(), None);
  assert!(!ok, "install without manifest should fail");
  assert!(stderr.contains("plugin.toml"));
  cleanup(&tmp);
}

#[test]
fn install_missing_wasm_errors() {
  let tmp = make_temp_dir("no_wasm");
  let fake_home = tmp.join("home");
  fs::create_dir_all(&fake_home).expect("create fake home");
  let bad_dir = tmp.join("bad-plugin");
  fs::create_dir_all(&bad_dir).expect("create bad dir");
  let manifest = "[plugin]\nname = \"bad\"\nversion = \"0.1.0\"\nwasm = \"missing.wasm\"\n";
  fs::write(bad_dir.join("plugin.toml"), manifest).expect("write manifest");
  let (ok, _, stderr) = run_plugin(&["install", bad_dir.to_str().unwrap()], fake_home.to_str().unwrap(), None);
  assert!(!ok, "install with missing wasm should fail");
  assert!(stderr.contains("wasm file"));
  cleanup(&tmp);
}

#[test]
fn new_creates_scaffold() {
  let tmp = make_temp_dir("new_scaffold");
  let project_name = "my-test-plugin";
  let (ok, stdout, _) = run_plugin(&["new", project_name], tmp.to_str().unwrap(), Some(tmp.to_str().unwrap()));
  assert!(ok, "new should succeed");
  assert!(stdout.contains("Created plugin project 'my-test-plugin'"));
  assert!(stdout.contains("cargo build --release"));
  assert!(stdout.contains("lx plugin install ./my-test-plugin"));

  let project_dir = tmp.join(project_name);
  assert!(project_dir.join("Cargo.toml").exists());
  assert!(project_dir.join("src").join("lib.rs").exists());
  assert!(project_dir.join("plugin.toml").exists());
  assert!(project_dir.join(".cargo").join("config.toml").exists());

  let cargo = fs::read_to_string(project_dir.join("Cargo.toml")).unwrap();
  assert!(cargo.contains("name = \"my-test-plugin\""));
  assert!(cargo.contains("cdylib"));
  assert!(cargo.contains("extism-pdk"));

  let lib = fs::read_to_string(project_dir.join("src").join("lib.rs")).unwrap();
  assert!(lib.contains("Hello from my-test-plugin"));

  let plugin_toml = fs::read_to_string(project_dir.join("plugin.toml")).unwrap();
  assert!(plugin_toml.contains("name = \"my-test-plugin\""));
  assert!(plugin_toml.contains("my_test_plugin.wasm"));

  let cargo_config = fs::read_to_string(project_dir.join(".cargo").join("config.toml")).unwrap();
  assert!(cargo_config.contains("wasm32-unknown-unknown"));

  cleanup(&tmp);
}

#[test]
fn new_rejects_existing_directory() {
  let tmp = make_temp_dir("new_exists");
  let project_name = "existing-dir";
  fs::create_dir_all(tmp.join(project_name)).expect("create existing dir");
  let (ok, _, stderr) = run_plugin(&["new", project_name], tmp.to_str().unwrap(), Some(tmp.to_str().unwrap()));
  assert!(!ok, "new should fail if directory exists");
  assert!(stderr.contains("already exists"));
  cleanup(&tmp);
}

#[test]
fn new_rejects_invalid_names() {
  let tmp = make_temp_dir("new_invalid");
  for bad_name in &["bad/name", "bad\\name", "bad..name", "bad name"] {
    let (ok, _, stderr) = run_plugin(&["new", bad_name], tmp.to_str().unwrap(), Some(tmp.to_str().unwrap()));
    assert!(!ok, "new should reject invalid name '{bad_name}'");
    assert!(stderr.contains("invalid plugin name"), "bad name '{bad_name}' should get validation error");
  }
  cleanup(&tmp);
}

#[test]
fn list_shows_local_plugins() {
  let tmp = make_temp_dir("local_list");
  let fake_home = tmp.join("home");
  fs::create_dir_all(&fake_home).expect("create fake home");

  let project_dir = tmp.join("project");
  let local_plugins = project_dir.join(".lx").join("plugins").join("local-plug");
  make_fake_plugin(&local_plugins, "local-plug", "0.3.0");

  let (ok, stdout, _) = run_plugin(&["list"], fake_home.to_str().unwrap(), Some(project_dir.to_str().unwrap()));
  assert!(ok, "list should succeed");
  assert!(stdout.contains("local-plug"));
  assert!(stdout.contains("0.3.0"));
  assert!(stdout.contains("local"));

  cleanup(&tmp);
}
