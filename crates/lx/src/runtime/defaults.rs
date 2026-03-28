use std::io::Write;

use crate::error::LxError;
use crate::value::LxVal;
use miette::SourceSpan;

use super::YieldBackend;

pub struct StdinStdoutYieldBackend;

impl YieldBackend for StdinStdoutYieldBackend {
  fn yield_value(&self, value: LxVal, span: SourceSpan) -> Result<LxVal, LxError> {
    use std::io::BufRead;
    let json = serde_json::Value::from(&value);
    let msg = serde_json::json!({"__yield": json});
    println!("{msg}");
    std::io::stdout().flush().map_err(|e| LxError::runtime(format!("yield: stdout: {e}"), span))?;
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).map_err(|e| LxError::runtime(format!("yield: stdin: {e}"), span))?;
    if line.trim().is_empty() {
      return Err(LxError::runtime("yield: orchestrator closed stdin", span));
    }
    let response: serde_json::Value = serde_json::from_str(line.trim()).map_err(|e| LxError::runtime(format!("yield: JSON parse: {e}"), span))?;
    Ok(LxVal::from(response))
  }
}
