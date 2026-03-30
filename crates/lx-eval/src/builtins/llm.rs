use lx_value::{BuiltinCtx, LxError, LxVal};
use miette::SourceSpan;

pub fn bi_prompt(_args: &[LxVal], span: SourceSpan, _ctx: &std::sync::Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Err(LxError::runtime("llm.prompt removed: use `use tool \"claude-mcp\" as llm` and call llm.prompt instead", span))
}

pub fn bi_prompt_with(_args: &[LxVal], span: SourceSpan, _ctx: &std::sync::Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Err(LxError::runtime("llm.prompt_with removed: use `use tool \"claude-mcp\" as llm` instead", span))
}

pub fn bi_prompt_structured(_args: &[LxVal], span: SourceSpan, _ctx: &std::sync::Arc<dyn BuiltinCtx>) -> Result<LxVal, LxError> {
  Err(LxError::runtime("llm.prompt_structured removed: use `use tool \"claude-mcp\" as llm` instead", span))
}
