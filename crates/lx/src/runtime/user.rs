use std::io::Write;

use crate::value::LxVal;

use super::UserBackend;

pub struct NoopUserBackend;

impl UserBackend for NoopUserBackend {
  fn confirm(&self, _message: &str) -> Result<bool, String> {
    Ok(true)
  }

  fn choose(&self, _message: &str, _options: &[String]) -> Result<usize, String> {
    Ok(0)
  }

  fn ask(&self, _message: &str, default: Option<&str>) -> Result<String, String> {
    Ok(default.unwrap_or("").to_string())
  }

  fn progress(&self, _current: usize, _total: usize, _message: &str) {}

  fn progress_pct(&self, _pct: f64, _message: &str) {}

  fn status(&self, _level: &str, _message: &str) {}

  fn table(&self, _headers: &[String], _rows: &[Vec<String>]) {}

  fn check_signal(&self) -> Option<LxVal> {
    None
  }
}

pub struct StdinStdoutUserBackend;

impl UserBackend for StdinStdoutUserBackend {
  fn confirm(&self, message: &str) -> Result<bool, String> {
    eprint!("{message} [y/N]: ");
    std::io::stderr().flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
    let answer = line.trim().to_lowercase();
    Ok(answer == "y" || answer == "yes")
  }

  fn choose(&self, message: &str, options: &[String]) -> Result<usize, String> {
    eprintln!("{message}");
    for (i, opt) in options.iter().enumerate() {
      eprintln!("  {}: {opt}", i + 1);
    }
    eprint!("choice [1-{}]: ", options.len());
    std::io::stderr().flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
    let n: usize = line.trim().parse().map_err(|_| format!("invalid choice: {}", line.trim()))?;
    if n < 1 || n > options.len() {
      return Err(format!("choice out of range: {n}"));
    }
    Ok(n - 1)
  }

  fn ask(&self, message: &str, default: Option<&str>) -> Result<String, String> {
    match default {
      Some(d) => eprint!("{message} [{d}]: "),
      None => eprint!("{message} "),
    }
    std::io::stderr().flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
    if trimmed.is_empty() { Ok(default.unwrap_or("").to_string()) } else { Ok(trimmed.to_string()) }
  }

  fn progress(&self, current: usize, total: usize, message: &str) {
    eprint!("\r[{current}/{total}] {message}");
    if let Err(e) = std::io::stderr().flush() {
      eprintln!("progress: stderr flush failed: {e}");
    }
    if current >= total {
      eprintln!();
    }
  }

  fn progress_pct(&self, pct: f64, message: &str) {
    let pct_display = (pct * 100.0).min(100.0);
    eprint!("\r[{pct_display:.0}%] {message}");
    if let Err(e) = std::io::stderr().flush() {
      eprintln!("progress_pct: stderr flush failed: {e}");
    }
    if pct >= 1.0 {
      eprintln!();
    }
  }

  fn status(&self, level: &str, message: &str) {
    let tag = match level.trim_start_matches(':') {
      "info" => "INFO",
      "warn" => "WARN",
      "error" => "ERROR",
      "success" => "OK",
      other => other,
    };
    eprintln!("[{tag}] {message}");
  }

  fn table(&self, headers: &[String], rows: &[Vec<String>]) {
    let col_count = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
      for (i, cell) in row.iter().enumerate() {
        if i < col_count && cell.len() > widths[i] {
          widths[i] = cell.len();
        }
      }
    }
    let header_line: Vec<String> = headers.iter().enumerate().map(|(i, h)| format!("{:width$}", h, width = widths[i])).collect();
    eprintln!("{}", header_line.join("  "));
    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    eprintln!("{}", sep.join("  "));
    for row in rows {
      let cells: Vec<String> = (0..col_count)
        .map(|i| {
          let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
          format!("{:width$}", cell, width = widths[i])
        })
        .collect();
      eprintln!("{}", cells.join("  "));
    }
  }

  fn check_signal(&self) -> Option<LxVal> {
    let pid = std::process::id();
    let path = format!(".lx/signals/{pid}.json");
    let content = std::fs::read_to_string(&path).ok()?;
    if let Err(e) = std::fs::remove_file(&path) {
      eprintln!("check_signal: failed to remove signal file {path}: {e}");
    }
    let jv: serde_json::Value = serde_json::from_str(content.trim()).ok()?;
    Some(LxVal::from(jv))
  }
}
