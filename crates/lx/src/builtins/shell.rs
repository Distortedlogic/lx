use std::process::Command;
use std::sync::Arc;

use crate::BuiltinCtx;
use crate::env::Env;
use crate::error::LxError;
use crate::record;
use crate::value::LxVal;
use miette::SourceSpan;

fn bi_bash(args: &[LxVal], span: SourceSpan, _ctx: &Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  let cmd = args[0].require_str("bash", span)?;
  match Command::new("bash").arg("-c").arg(cmd).output() {
    Ok(output) => {
      let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
      let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
      let code = output.status.code().unwrap_or(-1) as i64;
      Ok(LxVal::ok(record! {
        "stdout" => LxVal::str(stdout),
        "stderr" => LxVal::str(stderr),
        "code" => LxVal::int(code),
      }))
    },
    Err(e) => Ok(LxVal::err_str(e.to_string())),
  }
}

pub fn register(env: &Env) {
  super::register_builtins!(env, {
    "bash"/1 => bi_bash,
  });
}
