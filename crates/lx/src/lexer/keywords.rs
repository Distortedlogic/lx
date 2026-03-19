use crate::token::TokenKind;

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
        "refine" => TokenKind::Refine,
        "receive" => TokenKind::Receive,
        _ => TokenKind::Ident(text.to_string()),
    }
}

pub(super) fn type_name_or_keyword(text: &str) -> TokenKind {
    match text {
        "Protocol" => TokenKind::Protocol,
        "MCP" => TokenKind::Mcp,
        "Trait" => TokenKind::Trait,
        "Agent" => TokenKind::AgentKw,
        "Class" => TokenKind::ClassKw,
        _ => TokenKind::TypeName(text.to_string()),
    }
}
