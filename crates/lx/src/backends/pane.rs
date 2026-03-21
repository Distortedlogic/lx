use std::io::{BufRead, Write};
use std::sync::Arc;

use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::LxVal;

use super::PaneBackend;

pub struct YieldPaneBackend;

impl PaneBackend for YieldPaneBackend {
    fn open(&self, kind: &str, config: &LxVal, span: Span) -> Result<LxVal, LxError> {
        let config_json = serde_json::Value::from(config);
        let msg = serde_json::json!({
            "__pane": {"action": "open", "kind": kind, "config": config_json}
        });
        println!("{msg}");
        std::io::stdout()
            .flush()
            .map_err(|e| LxError::runtime(format!("pane: stdout: {e}"), span))?;
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| LxError::runtime(format!("pane: stdin: {e}"), span))?;
        if line.trim().is_empty() {
            return Err(LxError::runtime("pane: orchestrator closed stdin", span));
        }
        let response: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| LxError::runtime(format!("pane: JSON parse: {e}"), span))?;
        let pane_id = response
            .get("pane_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LxError::runtime("pane: response missing 'pane_id'", span))?;
        Ok(record! {
            "__pane_id" => LxVal::Str(Arc::from(pane_id)),
            "kind" => LxVal::Str(Arc::from(kind)),
        })
    }

    fn update(&self, pane_id: &str, content: &LxVal, span: Span) -> Result<(), LxError> {
        let content_json = serde_json::Value::from(content);
        let msg = serde_json::json!({
            "__pane": {"action": "update", "pane_id": pane_id, "content": content_json}
        });
        println!("{msg}");
        std::io::stdout()
            .flush()
            .map_err(|e| LxError::runtime(format!("pane: stdout: {e}"), span))?;
        Ok(())
    }

    fn close(&self, pane_id: &str, _span: Span) -> Result<(), LxError> {
        let msg = serde_json::json!({
            "__pane": {"action": "close", "pane_id": pane_id}
        });
        println!("{msg}");
        std::io::stdout()
            .flush()
            .map_err(|e| LxError::runtime(format!("pane: stdout: {e}"), _span))?;
        Ok(())
    }

    fn list(&self, span: Span) -> Result<LxVal, LxError> {
        let msg = serde_json::json!({"__pane": {"action": "list"}});
        println!("{msg}");
        std::io::stdout()
            .flush()
            .map_err(|e| LxError::runtime(format!("pane: stdout: {e}"), span))?;
        let mut line = String::new();
        std::io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|e| LxError::runtime(format!("pane: stdin: {e}"), span))?;
        if line.trim().is_empty() {
            return Err(LxError::runtime("pane: orchestrator closed stdin", span));
        }
        let response: serde_json::Value = serde_json::from_str(line.trim())
            .map_err(|e| LxError::runtime(format!("pane: JSON parse: {e}"), span))?;
        Ok(LxVal::from(response))
    }
}
