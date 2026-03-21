use crate::event::{RuntimeEvent, SpanInfo};

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const DIM: &str = "\x1b[2m";
pub const BOLD: &str = "\x1b[1m";
pub const RESET: &str = "\x1b[0m";

pub fn format_ai_start(model: &str, prompt: &str) -> String {
  let truncated = if prompt.len() > 200 { &prompt[..200] } else { prompt };
  format!("{BLUE}{BOLD}[AI] {model}{RESET}\n{DIM}{truncated}{RESET}")
}

pub fn format_ai_complete(response: &str, model: &str, cost: Option<f64>, duration_ms: u64) -> String {
  let cost_str = cost.map_or(String::new(), |c| format!(", ${c:.4}"));
  format!("{response}\n{DIM}({model}{cost_str}, {duration_ms}ms){RESET}")
}

pub fn format_ai_error(error: &str) -> String {
  format!("{RED}[AI ERROR] {error}{RESET}")
}

pub fn format_log(level: &str, msg: &str) -> String {
  let color = match level {
    "info" => BLUE,
    "warn" => YELLOW,
    "err" => RED,
    "debug" => DIM,
    _ => RESET,
  };
  format!("{color}[{level}]{RESET} {msg}")
}

pub fn format_emit(value: &str) -> String {
  value.to_string()
}

pub fn format_shell_exec(cmd: &str) -> String {
  format!("{DIM}$ {cmd}{RESET}")
}

pub fn format_shell_result(exit_code: i32, stdout: &str, stderr: &str) -> String {
  let mut out = String::new();
  if !stdout.is_empty() {
    out.push_str(stdout);
  }
  if !stderr.is_empty() {
    if !out.is_empty() {
      out.push('\n');
    }
    out.push_str(&format!("{RED}{stderr}{RESET}"));
  }
  if exit_code != 0 {
    if !out.is_empty() {
      out.push('\n');
    }
    out.push_str(&format!("{RED}exit {exit_code}{RESET}"));
  }
  out
}

pub fn format_error(error: &str, span_info: Option<&SpanInfo>) -> String {
  match span_info {
    Some(si) => format!("{RED}{BOLD}error:{RESET} {RED}{error}{RESET} {DIM}({}:{}){RESET}", si.start_line, si.start_col),
    None => format!("{RED}{BOLD}error:{RESET} {RED}{error}{RESET}"),
  }
}

pub fn format_progress(current: usize, total: usize, message: &str) -> String {
  let pct = if total == 0 { 0 } else { (current * 100) / total };
  let filled = pct / 5;
  let empty = 20 - filled;
  format!("\r[{}>{}] {pct}% {message}", "=".repeat(filled), " ".repeat(empty),)
}

pub fn format_agent_spawned(agent_id: &str, name: &str) -> String {
  format!("{GREEN}[SPAWN] {name} ({agent_id}){RESET}")
}

pub fn format_agent_killed(agent_id: &str) -> String {
  format!("{DIM}[EXIT] {agent_id}{RESET}")
}

pub fn format_program_started(path: &str) -> String {
  format!("{BOLD}[START] {path}{RESET}")
}

pub fn format_program_finished(result: &Result<String, String>, duration_ms: u64) -> String {
  match result {
    Ok(val) => format!("{GREEN}[OK]{RESET} {val} {DIM}({duration_ms}ms){RESET}"),
    Err(e) => format!("{RED}[FAIL]{RESET} {e} {DIM}({duration_ms}ms){RESET}"),
  }
}

pub fn format_event(event: &RuntimeEvent) -> String {
  match event {
    RuntimeEvent::AgentSpawned { agent_id, name, .. } => format_agent_spawned(agent_id, name),
    RuntimeEvent::AgentKilled { agent_id, .. } => format_agent_killed(agent_id),
    RuntimeEvent::AiCallStart { model, prompt, .. } => format_ai_start(model.as_deref().unwrap_or("unknown"), prompt),
    RuntimeEvent::AiCallComplete { response, model, cost_usd, duration_ms, .. } => format_ai_complete(response, model, *cost_usd, *duration_ms),
    RuntimeEvent::AiCallError { error, .. } => format_ai_error(error),
    RuntimeEvent::Emit { value, .. } => format_emit(value),
    RuntimeEvent::Log { level, msg, .. } => format_log(level, msg),
    RuntimeEvent::ShellExec { cmd, .. } => format_shell_exec(cmd),
    RuntimeEvent::ShellResult { exit_code, stdout, stderr, .. } => format_shell_result(*exit_code, stdout, stderr),
    RuntimeEvent::Error { error, span_info, .. } => format_error(error, span_info.as_ref()),
    RuntimeEvent::Progress { current, total, message, .. } => format_progress(*current, *total, message),
    RuntimeEvent::ProgramStarted { source_path, .. } => format_program_started(source_path),
    RuntimeEvent::ProgramFinished { result, duration_ms, .. } => format_program_finished(result, *duration_ms),
    RuntimeEvent::MessageSend { from_agent, to_agent, msg, .. } => format!("{CYAN}[MSG] {from_agent} -> {to_agent}:{RESET} {msg}"),
    RuntimeEvent::MessageAsk { from_agent, to_agent, msg, .. } => format!("{CYAN}[ASK] {from_agent} -> {to_agent}:{RESET} {msg}"),
    RuntimeEvent::MessageResponse { from_agent, to_agent, response, duration_ms, .. } => {
      format!("{CYAN}[REPLY] {from_agent} -> {to_agent}:{RESET} {response} {DIM}({duration_ms}ms){RESET}")
    },
    RuntimeEvent::UserPrompt { kind, .. } => {
      format!("{MAGENTA}[PROMPT]{RESET} {kind:?}")
    },
    RuntimeEvent::UserResponse { response, .. } => {
      format!("{MAGENTA}[RESPONSE]{RESET} {response}")
    },
    RuntimeEvent::TraceSpanRecorded { name, input, output, .. } => format!("{DIM}[SPAN] {name}: {input} -> {output}{RESET}"),
  }
}
