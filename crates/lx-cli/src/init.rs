use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

pub fn run_init(name: Option<&str>, flow: bool) -> ExitCode {
  let project_dir = match name {
    Some(n) => {
      let dir = Path::new(n);
      if let Err(e) = fs::create_dir_all(dir) {
        eprintln!("error: cannot create directory {n}: {e}");
        return ExitCode::from(1);
      }
      dir.to_path_buf()
    },
    None => match env::current_dir() {
      Ok(d) => d,
      Err(e) => {
        eprintln!("error: cannot determine cwd: {e}");
        return ExitCode::from(1);
      },
    },
  };

  let project_name = name.unwrap_or_else(|| project_dir.file_name().and_then(|n| n.to_str()).unwrap_or("my-project"));

  let manifest = if flow {
    format!("[package]\nname = \"{project_name}\"\nversion = \"0.1.0\"\nentry = \"src/main.lx\"\n\n[test]\nthreshold = 0.75\nruns = 1\n")
  } else {
    format!("[package]\nname = \"{project_name}\"\nversion = \"0.1.0\"\nentry = \"src/main.lx\"\n")
  };

  let manifest_path = project_dir.join(lx_span::LX_MANIFEST);
  if manifest_path.exists() {
    eprintln!("error: lx.toml already exists");
    return ExitCode::from(1);
  }

  if let Err(e) = fs::write(&manifest_path, &manifest) {
    eprintln!("error: cannot write lx.toml: {e}");
    return ExitCode::from(1);
  }

  let src_dir = project_dir.join("src");
  if let Err(e) = fs::create_dir_all(&src_dir) {
    eprintln!("error: cannot create src/: {e}");
    return ExitCode::from(1);
  }

  let main_lx = src_dir.join("main.lx");
  if let Err(e) = fs::write(&main_lx, "emit \"hello from lx\"\n") {
    eprintln!("error: cannot write src/main.lx: {e}");
    return ExitCode::from(1);
  }

  let test_dir = project_dir.join("test");
  if let Err(e) = fs::create_dir_all(&test_dir) {
    eprintln!("error: cannot create test/: {e}");
    return ExitCode::from(1);
  }

  let test_file = test_dir.join("main_test.lx");
  if let Err(e) = fs::write(&test_file, "assert true\n") {
    eprintln!("error: cannot write test/main_test.lx: {e}");
    return ExitCode::from(1);
  }

  if flow {
    let agents_dir = src_dir.join("agents");
    if let Err(e) = fs::create_dir_all(&agents_dir) {
      eprintln!("error: cannot create src/agents/: {e}");
      return ExitCode::from(1);
    }

    let scenarios_dir = test_dir.join("scenarios");
    if let Err(e) = fs::create_dir_all(&scenarios_dir) {
      eprintln!("error: cannot create test/scenarios/: {e}");
      return ExitCode::from(1);
    }
  }

  println!("Created {project_name}/");
  println!("  lx.toml");
  println!("  src/main.lx");
  println!("  test/main_test.lx");
  if flow {
    println!("  src/agents/");
    println!("  test/scenarios/");
  }

  ExitCode::SUCCESS
}
