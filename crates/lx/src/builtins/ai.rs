use std::sync::Arc;

use crate::error::LxError;
use crate::runtime::{AiOpts, RuntimeCtx};
use crate::sym::intern;
use crate::value::LxVal;
use miette::SourceSpan;

pub fn bi_prompt(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let text = args[0].require_str("ai.prompt", span)?;
  ctx.ai.prompt(text, span)
}

pub fn bi_prompt_with(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let LxVal::Record(rec) = &args[0] else {
    return Err(LxError::type_err("ai.prompt_with: expected Record", span, None));
  };
  let prompt =
    rec.get(&intern("prompt")).and_then(|v| v.as_str()).ok_or_else(|| LxError::type_err("ai.prompt_with: 'prompt' field required", span, None))?.to_string();
  let tools =
    rec.get(&intern("tools")).and_then(|v| v.as_list()).map(|list| list.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default();
  let max_turns = rec.get(&intern("max_turns")).and_then(|v| v.as_int()).and_then(|n| u32::try_from(n).ok());
  let json_schema = rec.get(&intern("json_schema")).and_then(|v| v.as_str()).map(String::from);

  let opts = AiOpts { prompt, tools, max_turns, json_schema };
  ctx.ai.prompt_with(&opts, span)
}

pub fn bi_prompt_structured(args: &[LxVal], span: SourceSpan, ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let schema = args[0].require_str("ai.prompt_structured", span)?.to_string();
  let prompt = args[1].require_str("ai.prompt_structured", span)?.to_string();
  let opts = AiOpts { prompt, json_schema: Some(schema), ..Default::default() };
  ctx.ai.prompt_with(&opts, span)
}
