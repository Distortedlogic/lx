mod agent_cmd;
mod check;
mod fmt;
mod init;
mod install;
mod install_ops;
mod listing;
mod lockfile;
mod manifest;
mod run;
mod testing;

use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use lx::prelude::RuntimeCtx;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lx", version, about = "The lx scripting language")]
struct Cli {
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  Run {
    file: String,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    control: Option<String>,
  },
  Test {
    #[arg()]
    dir: Option<String>,
    #[arg(short, long)]
    member: Option<String>,
  },
  Check {
    #[arg()]
    file: Option<String>,
    #[arg(short, long)]
    member: Option<String>,
    #[arg(long)]
    strict: bool,
    #[arg(long)]
    fix: bool,
  },
  Fmt {
    file: Option<String>,
    #[arg(long)]
    member: Option<String>,
    #[arg(long)]
    check: bool,
  },
  Agent {
    script: String,
  },
  Diagram {
    file: String,
    #[arg(short, long)]
    output: Option<String>,
  },
  List,
  Init {
    #[arg()]
    name: Option<String>,
    #[arg(long)]
    flow: bool,
  },
  Install {
    #[arg()]
    package: Option<String>,
  },
  Update {
    #[arg()]
    package: Option<String>,
  },
}

fn main() -> ExitCode {
  let cli = Cli::parse();
  match cli.command {
    Command::Run { file, json, control } => {
      let resolved = resolve_run_target(&file);
      run_file(&resolved, json, control.as_deref())
    },
    Command::Check { file, member, strict, fix } => {
      if let Some(file) = file {
        check::check_file(&file, strict, fix)
      } else {
        check::check_workspace(member.as_deref(), strict, fix)
      }
    },
    Command::Fmt { file, member, check } => {
      if let Some(path) = file {
        fmt::fmt_file(&path, check)
      } else {
        fmt::fmt_workspace(member.as_deref(), check)
      }
    },
    Command::Test { dir, member } => {
      if let Some(dir) = dir {
        testing::run_tests_dir(&dir)
      } else {
        testing::run_workspace_tests(member.as_deref())
      }
    },
    Command::Agent { script } => agent_cmd::run_agent(&script),
    Command::Diagram { file, output } => run_diagram(&file, output.as_deref()),
    Command::List => listing::list_workspace(),
    Command::Init { name, flow } => init::run_init(name.as_deref(), flow),
    Command::Install { package } => install::run_install(package.as_deref()),
    Command::Update { package } => install::run_update(package.as_deref()),
  }
}

fn resolve_run_target(target: &str) -> String {
  let path = Path::new(target);
  if path.exists() && path.is_file() {
    return target.to_string();
  }
  let Ok(cwd) = env::current_dir() else {
    return target.to_string();
  };
  let Some(root) = manifest::find_workspace_root(&cwd) else {
    return target.to_string();
  };
  let Ok(ws) = manifest::load_workspace(&root) else {
    return target.to_string();
  };
  for member in &ws.members {
    if member.pkg.name == target {
      let entry = member.pkg.entry.as_deref().unwrap_or("main.lx");
      return member.dir.join(entry).to_string_lossy().to_string();
    }
  }
  target.to_string()
}

fn run_file(path: &str, _json: bool, control_spec: Option<&str>) -> ExitCode {
  let source = match fs::read_to_string(path) {
    Ok(s) => s,
    Err(e) => {
      eprintln!("error: cannot read {path}: {e}");
      return ExitCode::from(1);
    },
  };
  let ws_members = manifest::try_load_workspace_members();
  let dep_dirs = manifest::try_load_dep_dirs_no_dev();
  let mut ctx_val = if std::io::stdin().is_terminal() { RuntimeCtx { ..RuntimeCtx::default() } } else { RuntimeCtx::default() };
  ctx_val.workspace_members = ws_members;
  ctx_val.dep_dirs = dep_dirs;
  apply_manifest_tools(&mut ctx_val, path);
  apply_manifest_backends(&mut ctx_val, path);
  if let Some(spec) = control_spec
    && spec == "stdin"
  {
    let (inject_tx, inject_rx) = tokio::sync::mpsc::channel::<lx_value::LxVal>(1);
    ctx_val.inject_tx = Some(inject_tx);
    ctx_val.yield_ = Arc::new(lx_eval::runtime::ControlYieldBackend { inject_rx: Arc::new(tokio::sync::Mutex::new(inject_rx)) });
  }
  let ctx = Arc::new(ctx_val);
  ctx.tokio_runtime.block_on(setup_external_stream(&ctx, path));
  match run::run(&source, path, &ctx, control_spec) {
    Ok(()) => ExitCode::SUCCESS,
    Err(errors) => {
      let named = miette::NamedSource::new(path, source.clone());
      for err in errors {
        if let lx_value::error::LxError::Sourced { source_name, source_text, inner } = err {
          let src = miette::NamedSource::new(source_name, source_text.to_string());
          let report = miette::Report::new(*inner).with_source_code(src);
          eprintln!("{report:?}");
        } else {
          let report = miette::Report::new(err).with_source_code(named.clone());
          eprintln!("{report:?}");
        }
      }
      ExitCode::from(1)
    },
  }
}

fn apply_manifest_tools(ctx: &mut RuntimeCtx, file_path: &str) {
  let file_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
  let Some(root) = manifest::find_manifest_root(file_dir) else {
    return;
  };
  let Ok(m) = manifest::load_manifest(&root) else {
    return;
  };
  let Some(tools) = m.tools else {
    return;
  };
  for (name, spec) in tools {
    let decl = match spec {
      manifest::ToolSpec::Lx { path } => lx::prelude::ToolDecl::Lx { path: root.join(path) },
      manifest::ToolSpec::Mcp { command } => lx::prelude::ToolDecl::Mcp { command },
    };
    ctx.tools.insert(name, decl);
  }
}

fn apply_manifest_backends(_ctx: &mut RuntimeCtx, file_path: &str) {
  let file_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
  let Some(root) = manifest::find_manifest_root(file_dir) else {
    return;
  };
  let Ok(m) = manifest::load_manifest(&root) else {
    return;
  };
  if let Some(ref backends) = m.backends
    && let Some(ref backend) = backends.yield_backend
  {
    match backend {
      manifest::YieldBackend::StdinStdout => {},
    }
  }
}

async fn setup_external_stream(ctx: &Arc<RuntimeCtx>, file_path: &str) {
  let file_dir = Path::new(file_path).parent().unwrap_or(Path::new("."));
  let Some(root) = manifest::find_manifest_root(file_dir) else {
    return;
  };
  let Ok(m) = manifest::load_manifest(&root) else {
    return;
  };
  let Some(stream_config) = m.stream else {
    return;
  };
  let command = stream_config.command;
  match lx_eval::mcp_client::McpClient::spawn(&command).await {
    Ok(client) => {
      let sink = Arc::new(lx_eval::mcp_stream_sink::McpStreamSink::new(client));
      ctx.event_stream.set_external_client(sink);
    },
    Err(e) => {
      eprintln!("[stream:external] failed to connect to '{command}': {e}");
    },
  }
}

fn run_diagram(path: &str, output: Option<&str>) -> ExitCode {
  let (_source, program) = match run::read_and_parse(path) {
    Ok(sp) => sp,
    Err(code) => return code,
  };
  let mermaid = lx_eval::stdlib::diag::extract_mermaid(&program);
  match output {
    Some(out_path) => {
      if let Err(e) = fs::write(out_path, &mermaid) {
        eprintln!("error: cannot write {out_path}: {e}");
        return ExitCode::from(1);
      }
      println!("wrote diagram to {out_path}");
    },
    None => print!("{mermaid}"),
  }
  ExitCode::SUCCESS
}
