//! Token-level helpers and the authoritative Edition 1 precedence table.
//!
//! # Precedence is defined exactly once
//!
//! [`binding_power`] is the single authority for Edition 1 operator
//! precedence. Before 0.0.2.3 there were two independent tables -- `infix` in
//! `parser.rs` and `binding_power` in `expr.rs` -- that disagreed with each
//! other and with the EBNF. Two tables is one too many: a parser can only obey
//! one of them, and the other silently rots. Both now read from this one.
//!
//! The binding powers are transcribed from the "Precedence" section of
//! `spec/grammar/omni-edition1.ebnf` (lowest to highest), and
//! `precedence_matches_the_normative_grammar` pins the ordering so it cannot
//! drift.

use omni_lex::token::{Kw, Punct};
use omni_lex::TokenKind;

/// Left and right binding powers for an infix operator.
///
/// Higher binds tighter. `(l, r)` where `r > l` makes the operator
/// right-associative; `r == l` makes it left-associative.
pub type BindingPower = (u8, u8);

/// No binding power: this token cannot continue an expression.
pub const NO_BINDING: BindingPower = (0, 0);

/// Binding power for unary/prefix operators. This is above cast precedence.
pub const UNARY_BINDING_POWER: u8 = 28;

/// Binding power for postfix call/navigation operators. Generic parsing remains
/// a later wave; this constant only prevents call parsing from weakening binary
/// precedence in the current parser subset.
pub const POSTFIX_BINDING_POWER: u8 = 30;

/// The Edition 1 infix precedence table, transcribed from the normative EBNF
/// precedence section.
pub const fn binding_power(kind: TokenKind) -> BindingPower {
    match kind {
        // 1. assignment -- right-associative.
        TokenKind::Punct(
            Punct::Eq
            | Punct::PlusEq
            | Punct::MinusEq
            | Punct::StarEq
            | Punct::SlashEq
            | Punct::PercentEq
            | Punct::AmpEq
            | Punct::PipeEq
            | Punct::CaretEq
            | Punct::ShlEq
            | Punct::ShrEq,
        ) => (2, 1),

        // 2. ranges.
        TokenKind::Punct(Punct::DotDot | Punct::DotDotEq) => (4, 5),

        // 3. logical OR.
        TokenKind::Punct(Punct::PipePipe) => (6, 7),
        // 4. logical AND.
        TokenKind::Punct(Punct::AmpAmp) => (8, 9),
        // 5. null coalescing.
        TokenKind::Punct(Punct::QuestionQuestion) => (10, 11),
        // 6. comparison.
        TokenKind::Punct(
            Punct::EqEq | Punct::NotEq | Punct::Lt | Punct::Le | Punct::Gt | Punct::Ge,
        ) => (12, 13),
        // 7. bitwise OR.
        TokenKind::Punct(Punct::Pipe) => (14, 15),
        // 8. bitwise XOR.
        TokenKind::Punct(Punct::Caret) => (16, 17),
        // 9. bitwise AND.
        TokenKind::Punct(Punct::Amp) => (18, 19),
        // 10. shifts.
        TokenKind::Punct(Punct::Shl | Punct::Shr) => (20, 21),
        // 11. additive.
        TokenKind::Punct(Punct::Plus | Punct::Minus) => (22, 23),
        // 12. multiplicative.
        TokenKind::Punct(Punct::Star | Punct::Slash | Punct::Percent) => (24, 25),
        // 13. casts. `as` is a keyword token.
        TokenKind::Keyword(Kw::As) => (26, 27),
        _ => NO_BINDING,
    }
}

/// Whether `kind` is an infix/binary operator at all.
pub fn is_infix_operator(kind: TokenKind) -> bool {
    binding_power(kind) != NO_BINDING
}

/// Whether `kind` is one of the comparison operators.
///
/// Edition 1 forbids chaining them (`a < b < c`), so the parser has to
/// recognize them as a group rather than treat each as an ordinary binary
/// operator that happens to left-associate.
pub fn is_comparison_operator(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Punct(
            Punct::EqEq | Punct::NotEq | Punct::Lt | Punct::Le | Punct::Gt | Punct::Ge
        )
    )
}

/// Whether `kind` is an assignment operator.

/// A logical token part produced when an operator token is consumed as one or
/// more generic closers. The part refers to a byte range inside the original
/// lexer token; it never owns or duplicates source bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenericCloserPart {
    pub kind: Punct,
    pub byte_offset: u8,
    pub byte_len: u8,
}

/// Split `>`, `>>`, or `>>=` into the logical punctuation needed by nested
/// generic closers. The ranges are contiguous and cover the original spelling.
pub const fn split_generic_closer(kind: Punct) -> Option<[Option<GenericCloserPart>; 3]> {
    match kind {
        Punct::Gt => Some([
            Some(GenericCloserPart { kind: Punct::Gt, byte_offset: 0, byte_len: 1 }),
            None,
            None,
        ]),
        Punct::Shr => Some([
            Some(GenericCloserPart {
                kind: Punct::Gt,
                byte_offset: 0,
                byte_len: 1,
            }),
            Some(GenericCloserPart {
                kind: Punct::Gt,
                byte_offset: 1,
                byte_len: 1,
            }),
            None,
        ]),
        Punct::ShrEq => Some([
            Some(GenericCloserPart {
                kind: Punct::Gt,
                byte_offset: 0,
                byte_len: 1,
            }),
            Some(GenericCloserPart {
                kind: Punct::Gt,
                byte_offset: 1,
                byte_len: 1,
            }),
            Some(GenericCloserPart {
                kind: Punct::Eq,
                byte_offset: 2,
                byte_len: 1,
            }),
        ]),
        _ => None,
    }
}

pub fn is_assignment_operator(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Punct(
            Punct::Eq
                | Punct::PlusEq
                | Punct::MinusEq
                | Punct::StarEq
                | Punct::SlashEq
                | Punct::PercentEq
                | Punct::AmpEq
                | Punct::PipeEq
                | Punct::CaretEq
                | Punct::ShlEq
                | Punct::ShrEq
        )
    )
}

/// The delimiter matching an opening delimiter.
pub fn matching_close(open: Punct) -> Option<Punct> {
    match open {
        Punct::LParen => Some(Punct::RParen),
        Punct::LBracket => Some(Punct::RBracket),
        Punct::LBrace => Some(Punct::RBrace),
        _ => None,
    }
}

/// The opening delimiter matching a closing delimiter.
pub fn matching_open(close: Punct) -> Option<Punct> {
    match close {
        Punct::RParen => Some(Punct::LParen),
        Punct::RBracket => Some(Punct::LBracket),
        Punct::RBrace => Some(Punct::LBrace),
        _ => None,
    }
}

/// The keywords that can begin a module item, for lookahead.
///
/// This is a *classification* helper, not a parser entry point: it answers "is
/// the current token one of the words that can start an item?" so statement
/// parsing can tell an `item_decl_stmt` from an expression without committing
/// to a parse.
pub const ITEM_STARTERS: [Kw; 14] = [
    Kw::Fn,
    Kw::Struct,
    Kw::Enum,
    Kw::Trait,
    Kw::Impl,
    Kw::Const,
    Kw::Static,
    Kw::Use,
    Kw::Mod,
    Kw::Extern,
    Kw::Type,
    Kw::Pub,
    Kw::Unsafe,
    Kw::Async,
];

/// Whether `kw` can begin a module item (before attributes and visibility).
pub fn is_item_starter(kw: Kw) -> bool {
    ITEM_STARTERS.contains(&kw)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The binding powers must match the normative EBNF precedence ordering,
    /// lowest to highest. This asserts the *relative* order the specification
    /// states, not merely that the numbers are distinct.
    #[test]
    fn precedence_matches_the_normative_grammar() {
        // (token, label) in the exact order of the EBNF precedence section.
        let ladder: [(TokenKind, &str); 13] = [
            (TokenKind::Punct(Punct::Eq), "1. assignment"),
            (TokenKind::Punct(Punct::DotDot), "2. range"),
            (TokenKind::Punct(Punct::PipePipe), "3. logical or"),
            (TokenKind::Punct(Punct::AmpAmp), "4. logical and"),
            (TokenKind::Punct(Punct::QuestionQuestion), "5. null coalescing"),
            (TokenKind::Punct(Punct::EqEq), "6. comparison"),
            (TokenKind::Punct(Punct::Pipe), "7. bitwise or"),
            (TokenKind::Punct(Punct::Caret), "8. bitwise xor"),
            (TokenKind::Punct(Punct::Amp), "9. bitwise and"),
            (TokenKind::Punct(Punct::Shl), "10. shift"),
            (TokenKind::Punct(Punct::Plus), "11. additive"),
            (TokenKind::Punct(Punct::Star), "12. multiplicative"),
            (TokenKind::Keyword(Kw::As), "13. cast"),
        ];
        let mut prev = 0u8;
        for (kind, label) in ladder {
            let (l, _r) = binding_power(kind);
            assert_ne!(l, NO_BINDING.0, "{label} must have a binding power");
            assert!(l > prev, "{label} must bind tighter than the level below it ({l} <= {prev})");
            prev = l;
        }
    }

    /// Every operator the EBNF lists must be present in the table. An operator
    /// silently missing from the table would parse as two separate expressions
    /// instead of one binary expression.
    #[test]
    fn every_edition1_operator_has_a_binding_power() {
        for p in [
            Punct::Plus,
            Punct::Minus,
            Punct::Star,
            Punct::Slash,
            Punct::Percent,
            Punct::Amp,
            Punct::Pipe,
            Punct::Caret,
            Punct::Shl,
            Punct::Shr,
            Punct::PlusEq,
            Punct::MinusEq,
            Punct::StarEq,
            Punct::SlashEq,
            Punct::PercentEq,
            Punct::AmpEq,
            Punct::PipeEq,
            Punct::CaretEq,
            Punct::ShlEq,
            Punct::ShrEq,
            Punct::EqEq,
            Punct::NotEq,
            Punct::Lt,
            Punct::Le,
            Punct::Gt,
            Punct::Ge,
            Punct::AmpAmp,
            Punct::PipePipe,
            Punct::QuestionQuestion,
            Punct::DotDot,
            Punct::DotDotEq,
        ] {
            assert!(is_infix_operator(TokenKind::Punct(p)), "{p:?} must have a binding power");
        }
        assert!(is_infix_operator(TokenKind::Keyword(Kw::As)));
        // These are prefix/postfix-only and must NOT be binary operators.
        for p in [Punct::Bang, Punct::Tilde, Punct::Arrow, Punct::Dot, Punct::LParen] {
            assert!(!is_infix_operator(TokenKind::Punct(p)), "{p:?} is not a binary operator");
        }
    }

    /// Assignment is right-associative; every other binary level is
    /// left-associative. Getting this backwards silently reassociates `a = b = c`.
    #[test]
    fn associativity_matches_the_grammar() {
        assert_eq!(
            binding_power(TokenKind::Punct(Punct::Eq)),
            (2, 1),
            "assignment must be right-associative"
        );
        for p in [Punct::Plus, Punct::Star, Punct::AmpAmp, Punct::Pipe, Punct::EqEq] {
            let (l, r) = binding_power(TokenKind::Punct(p));
            assert_eq!(l + 1, r, "{p:?} must be left-associative");
        }
    }

    /// Comparison chaining must be recognizable as a group so the parser can
    /// reject `a < b < c` instead of silently accepting it.
    #[test]
    fn comparison_operators_form_a_distinguishable_group() {
        for p in [Punct::EqEq, Punct::NotEq, Punct::Lt, Punct::Le, Punct::Gt, Punct::Ge] {
            assert!(is_comparison_operator(TokenKind::Punct(p)), "{p:?} is a comparison");
            assert!(!is_assignment_operator(TokenKind::Punct(p)));
        }
        for p in [Punct::Plus, Punct::Star, Punct::Amp] {
            assert!(!is_comparison_operator(TokenKind::Punct(p)), "{p:?} is not a comparison");
        }
    }

    /// `|>` is Candidate 2's pipeline operator and must never acquire a binding
    /// power, or the parser would silently implement semantics that are not in
    /// force.
    #[test]
    fn candidate_two_operators_stay_unbound() {
        for p in [Punct::PipeArrow, Punct::QuestionDot] {
            assert!(
                !is_infix_operator(TokenKind::Punct(p)),
                "{p:?} is Candidate 2 and must stay unbound while that gate is disabled"
            );
            assert!(!is_comparison_operator(TokenKind::Punct(p)));
            assert!(!is_assignment_operator(TokenKind::Punct(p)));
        }
    }

    #[test]
    fn generic_closer_splits_cover_original_bytes_exactly() {
        for (punct, source) in [(Punct::Gt, ">"), (Punct::Shr, ">>"), (Punct::ShrEq, ">>=")] {
            let parts = split_generic_closer(punct).expect("generic closer");
            let mut cursor = 0usize;
            let mut rebuilt = String::new();
            for part in parts.into_iter().flatten() {
                assert_eq!(part.byte_offset as usize, cursor);
                let end = cursor + part.byte_len as usize;
                rebuilt.push_str(&source[cursor..end]);
                cursor = end;
            }
            assert_eq!(cursor, source.len());
            assert_eq!(rebuilt, source);
        }
    }

    #[test]
    fn generic_closer_does_not_reinterpret_ge() {
        assert_eq!(split_generic_closer(Punct::Ge), None);
    }

    #[test]
    fn delimiter_pairs_are_symmetric() {
        for open in [Punct::LParen, Punct::LBracket, Punct::LBrace] {
            let close = matching_close(open).expect("must have a match");
            assert_eq!(matching_open(close), Some(open));
        }
        for p in [Punct::Eq, Punct::Dot, Punct::Semicolon] {
            assert_eq!(matching_close(p), None);
            assert_eq!(matching_open(p), None);
        }
    }
}
