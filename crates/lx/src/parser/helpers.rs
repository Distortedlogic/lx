use crate::ast::{Expr, SExpr};
use crate::token::TokenKind;

pub(super) fn looks_like_record(p: &super::Parser) -> bool {
    let (cur, next) = (
        p.tokens.get(p.pos).map(|t| &t.kind),
        p.tokens.get(p.pos + 1).map(|t| &t.kind),
    );
    if matches!(
        (cur, next),
        (Some(TokenKind::Ident(_)), Some(TokenKind::Colon)) | (Some(TokenKind::DotDot), _)
    ) {
        return true;
    }
    if matches!(cur, Some(TokenKind::Ident(_))) {
        let mut j = p.pos;
        let mut ident_count = 0u32;
        loop {
            match p.tokens.get(j).map(|t| &t.kind) {
                Some(TokenKind::Ident(_)) => {
                    ident_count += 1;
                    j += 1;
                }
                Some(TokenKind::Semi) => {
                    j += 1;
                }
                Some(TokenKind::RBrace) => return ident_count >= 2,
                _ => return false,
            }
        }
    }
    false
}

pub(super) fn infix_bp(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::Question => Some((3, 4)),
        TokenKind::Amp => Some((7, 8)),
        TokenKind::QQ => Some((11, 12)),
        TokenKind::Or => Some((13, 14)),
        TokenKind::And => Some((15, 16)),
        TokenKind::Eq
        | TokenKind::NotEq
        | TokenKind::Lt
        | TokenKind::Gt
        | TokenKind::LtEq
        | TokenKind::GtEq => Some((17, 18)),
        TokenKind::Pipe => Some((19, 20)),
        TokenKind::PlusPlus | TokenKind::TildeArrow | TokenKind::TildeArrowQ => Some((21, 22)),
        TokenKind::DotDot | TokenKind::DotDotEq => Some((23, 24)),
        TokenKind::Plus | TokenKind::Minus => Some((25, 26)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent | TokenKind::IntDiv => {
            Some((27, 28))
        }
        TokenKind::Dot => Some((33, 34)),
        _ => None,
    }
}

pub(super) fn postfix_bp(kind: &TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Caret => Some(10),
        _ => None,
    }
}

impl super::Parser {
    pub(crate) fn is_application_candidate(&self, left: &SExpr, min_bp: u8) -> bool {
        if min_bp > 31 || self.no_juxtapose {
            return false;
        }
        let callable = if self.collection_depth > 0 {
            matches!(left.node, Expr::TypeConstructor(_))
                && !matches!(self.peek(), TokenKind::TypeName(_))
        } else {
            matches!(
                left.node,
                Expr::Ident(_)
                    | Expr::TypeConstructor(_)
                    | Expr::Apply { .. }
                    | Expr::FieldAccess { .. }
                    | Expr::Section(_)
                    | Expr::Func { .. }
            )
        };
        if !callable {
            return false;
        }
        matches!(
            self.peek(),
            TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::StrStart
                | TokenKind::RawStr(_)
                | TokenKind::Regex(_)
                | TokenKind::Ident(_)
                | TokenKind::TypeName(_)
                | TokenKind::LParen
                | TokenKind::LBracket
                | TokenKind::LBrace
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Unit
                | TokenKind::PercentLBrace
        )
    }
}
