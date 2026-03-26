use super::Lexer;
use super::token::{Token, TokenKind};
use crate::error::LxError;
use miette::SourceSpan;
use num_bigint::BigInt;

impl<'src> Lexer<'src> {
  pub(super) fn emit_int(&mut self, slice: &str, prefix_len: usize, radix: u32, span: SourceSpan) -> Result<(), LxError> {
    let d = Self::strip_underscores(&slice[prefix_len..]);
    let v = BigInt::parse_bytes(d.as_bytes(), radix).ok_or_else(|| LxError::parse("invalid integer literal", span, None))?;
    self.emit(Token::new(TokenKind::Int(v), span));
    Ok(())
  }
}

pub(super) fn ident_or_keyword(text: &str) -> TokenKind {
  match text {
    "true" => TokenKind::True,
    "false" => TokenKind::False,
    "use" => TokenKind::Use,
    "loop" => TokenKind::Loop,
    "break" => TokenKind::Break,
    "par" => TokenKind::Par,
    "sel" => TokenKind::Sel,
    "assert" => TokenKind::Assert,
    "emit" => TokenKind::Emit,
    "yield" => TokenKind::Yield,
    "with" => TokenKind::With,
    "timeout" => TokenKind::Timeout,
    "as" => TokenKind::As,
    _ => TokenKind::Ident(crate::sym::intern(text)),
  }
}

pub(super) fn type_name_or_keyword(text: &str) -> TokenKind {
  match text {
    "Trait" => TokenKind::Trait,
    "Class" => TokenKind::ClassKw,
    "Agent" => TokenKind::AgentKw,
    "Tool" => TokenKind::ToolKw,
    "Prompt" => TokenKind::PromptKw,
    "Store" => TokenKind::StoreKw,
    "Session" => TokenKind::SessionKw,
    "Guard" => TokenKind::GuardKw,
    "Workflow" => TokenKind::WorkflowKw,
    "Schema" => TokenKind::SchemaKw,
    "MCP" => TokenKind::McpKw,
    "CLI" => TokenKind::CliKw,
    "HTTP" => TokenKind::HttpKw,
    _ => TokenKind::TypeName(crate::sym::intern(text)),
  }
}
