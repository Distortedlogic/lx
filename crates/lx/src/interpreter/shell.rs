use crate::ast::{ShellMode, StrPart};
use crate::error::LxError;
use crate::span::Span;
use crate::value::LxVal;

use super::Interpreter;

impl Interpreter {
    pub(super) async fn eval_shell(
        &mut self,
        mode: &ShellMode,
        parts: &[StrPart],
        span: Span,
    ) -> Result<LxVal, LxError> {
        let cmd_str = self.build_shell_string(parts).await?;
        let cmd_trimmed = cmd_str.trim();
        match mode {
            ShellMode::Normal | ShellMode::Block => self.ctx.shell.exec(cmd_trimmed, span),
            ShellMode::Propagate => self.ctx.shell.exec_capture(cmd_trimmed, span),
        }
    }

    async fn build_shell_string(&mut self, parts: &[StrPart]) -> Result<String, LxError> {
        let mut cmd = String::new();
        for part in parts {
            match part {
                StrPart::Text(s) => cmd.push_str(s),
                StrPart::Interp(expr) => {
                    let val = self.eval_expr(expr).await?;
                    cmd.push_str(&format!("{val}"));
                }
            }
        }
        Ok(cmd)
    }
}
