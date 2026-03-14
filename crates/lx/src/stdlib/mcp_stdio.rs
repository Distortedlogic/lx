use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use crate::error::LxError;
use crate::span::Span;

pub(super) struct StdioTransport {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl StdioTransport {
    pub(super) fn spawn(cmd: &str, args: &[String], span: Span) -> Result<Self, LxError> {
        let mut child = Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| LxError::runtime(format!("mcp stdio: spawn: {e}"), span))?;
        let stdin = BufWriter::new(
            child
                .stdin
                .take()
                .ok_or_else(|| LxError::runtime("mcp stdio: no stdin", span))?,
        );
        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| LxError::runtime("mcp stdio: no stdout", span))?,
        );
        Ok(StdioTransport {
            child,
            stdin,
            stdout,
        })
    }

    pub(super) fn send(
        &mut self,
        req: &serde_json::Value,
        span: Span,
    ) -> Result<serde_json::Value, LxError> {
        let s = serde_json::to_string(req)
            .map_err(|e| LxError::runtime(format!("mcp stdio: encode: {e}"), span))?;
        writeln!(self.stdin, "{s}")
            .map_err(|e| LxError::runtime(format!("mcp stdio: write: {e}"), span))?;
        self.stdin
            .flush()
            .map_err(|e| LxError::runtime(format!("mcp stdio: flush: {e}"), span))?;
        loop {
            let mut line = String::new();
            self.stdout
                .read_line(&mut line)
                .map_err(|e| LxError::runtime(format!("mcp stdio: read: {e}"), span))?;
            if line.is_empty() {
                return Err(LxError::runtime("mcp stdio: server disconnected", span));
            }
            let jv: serde_json::Value = serde_json::from_str(line.trim())
                .map_err(|e| LxError::runtime(format!("mcp stdio: decode: {e}"), span))?;
            if jv.get("id").is_some() {
                return Ok(jv);
            }
        }
    }

    pub(super) fn send_notify(
        &mut self,
        req: &serde_json::Value,
        span: Span,
    ) -> Result<(), LxError> {
        let s = serde_json::to_string(req)
            .map_err(|e| LxError::runtime(format!("mcp stdio: encode: {e}"), span))?;
        writeln!(self.stdin, "{s}")
            .map_err(|e| LxError::runtime(format!("mcp stdio: write: {e}"), span))?;
        self.stdin
            .flush()
            .map_err(|e| LxError::runtime(format!("mcp stdio: flush: {e}"), span))
    }

    pub(super) fn shutdown(mut self, span: Span) -> Result<(), LxError> {
        drop(self.stdin);
        drop(self.stdout);
        match self.child.try_wait() {
            Ok(Some(_)) => Ok(()),
            _ => {
                self.child
                    .kill()
                    .map_err(|e| LxError::runtime(format!("mcp stdio: kill: {e}"), span))?;
                self.child
                    .wait()
                    .map_err(|e| LxError::runtime(format!("mcp stdio: wait: {e}"), span))?;
                Ok(())
            }
        }
    }
}
