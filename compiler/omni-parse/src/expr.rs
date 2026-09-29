//! Expression-parser scaffolding.
//!
//! # Precedence
//!
//! This module used to carry its own `binding_power` table, which duplicated
//! `infix` in `parser.rs` and disagreed with both the EBNF and the other copy:
//! it bound `=` left-associatively and omitted `&&`, `||`, `??`, `%`, `^`,
//! `<<`, `>>`, `as`, and every compound assignment. Two tables that can
//! disagree mean a parser obeys whichever one it happens to call, and the other
//! silently rots.
//!
//! [`binding_power`] is now a re-export of [`crate::precedence::binding_power`],
//! the single authority transcribed from `spec/grammar/omni-edition1.ebnf`. This
//! re-export is retained so existing callers keep compiling.

use omni_lex::TokenKind;
use omni_syntax::SyntaxKind;

pub use crate::precedence::{binding_power, is_infix_operator, NO_BINDING};

/// A sink the event-driven walker drives.
///
/// This trait exists for consumers that want to walk an expression with their
/// own token source.
///
/// The authoritative Edition 1 parse is the recursive-descent + Pratt parser in
/// [`crate::parser`]; this is a convenience view over the same precedence table.
pub trait ExprParser {
    fn peek(&self) -> Option<TokenKind>;
    fn advance(&mut self);
    fn start_node(&mut self, kind: SyntaxKind) -> usize;
    fn finish_node(&mut self);
    fn error(&mut self, msg: String);
}
pub fn parse_expr_bp<P: ExprParser>(p: &mut P, min_bp: u8) {
    let Some(kind) = p.peek() else {
        p.error("Unexpected EOF while parsing expression".into());
        return;
    };
    let mark = p.start_node(match kind {
        TokenKind::Ident => SyntaxKind::NameRef,
        TokenKind::Int
        | TokenKind::Float
        | TokenKind::Char
        | TokenKind::Byte
        | TokenKind::String
        | TokenKind::RawString
        | TokenKind::InterpolatedString
        | TokenKind::Keyword(_) => SyntaxKind::LiteralExpr,
        TokenKind::Punct(
            omni_lex::token::Punct::Minus
            | omni_lex::token::Punct::Bang
            | omni_lex::token::Punct::Amp,
        ) => SyntaxKind::UnaryExpr,
        _ => SyntaxKind::ErrorNode,
    });
    let _ = mark;
    p.advance();
    p.finish_node();
    while let Some(op) = p.peek() {
        let (l, r) = binding_power(op);
        if l < min_bp || l == NO_BINDING.0 {
            break;
        }
        p.advance();
        parse_expr_bp(p, r);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_lex::token::Punct;

    /// The re-export must be the *same* function as the shared table, not a
    /// look-alike. If someone reintroduces a local table, this fails.
    #[test]
    fn binding_power_is_the_shared_table() {
        assert!(
            std::ptr::eq(
                binding_power as *const () as *const u8,
                crate::precedence::binding_power as *const () as *const u8
            ),
            "binding_power must be the shared table, not a local copy"
        );
    }

    /// The operators the old local table omitted must now be bound here too.
    #[test]
    fn previously_missing_operators_are_now_bound() {
        for p in [
            Punct::AmpAmp,
            Punct::PipePipe,
            Punct::QuestionQuestion,
            Punct::Percent,
            Punct::Caret,
            Punct::Shl,
            Punct::Shr,
            Punct::StarEq,
            Punct::PlusEq,
        ] {
            assert!(is_infix_operator(TokenKind::Punct(p)), "{p:?} must be bound here too");
        }
        // Assignment is right-associative in the shared table; it was not before.
        assert_eq!(binding_power(TokenKind::Punct(Punct::Eq)), (2, 1));
    }
}
