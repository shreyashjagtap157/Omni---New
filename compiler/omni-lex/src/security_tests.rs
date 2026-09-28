//! SRC-0005 source-security acceptance and rejection tests.
//!
//! SRC-0005 reads, verbatim from `spec/registry/rule-texts.json`:
//!
//! > Outside comments and literals, bidi control characters, noncharacters,
//! > unassigned code points, variation selectors, and default-ignorable format
//! > characters are source errors.
//!
//! Two halves are tested separately. First, every prohibited class is rejected
//! in source position with an exact byte span and the correct class, wherever
//! it appears: at the start of an identifier, in the middle, at the end,
//! standalone, next to punctuation, next to whitespace, and repeated. Second,
//! nothing else regresses: the prohibited characters stay legal *inside*
//! comments (SRC-0006) and inside literals (SRC-0007), lossless reconstruction
//! still holds for every one of these inputs, and malformed UTF-8 still reports
//! as malformed UTF-8 rather than being conflated with a prohibited code point.

use crate::scanner::Scanner;
use crate::token::{ErrorReason, Punct, Span, Token, TokenKind, Trivia};
use omni_unicode::annotation::SecurityMode;
use omni_unicode::classify::ProhibitedKind;

/// One representative from each SRC-0005 class, taken from the UCD 17.0.0
/// tables rather than from memory.
const BIDI: &[(u32, ProhibitedKind)] = &[
    (0x061C, ProhibitedKind::BidiControl),
    (0x200E, ProhibitedKind::BidiControl),
    (0x200F, ProhibitedKind::BidiControl),
    (0x202A, ProhibitedKind::BidiControl),
    (0x202E, ProhibitedKind::BidiControl),
    (0x2066, ProhibitedKind::BidiControl),
    (0x2069, ProhibitedKind::BidiControl),
];
const NONCHARACTERS: &[(u32, ProhibitedKind)] = &[
    (0xFDD0, ProhibitedKind::Noncharacter),
    (0xFDEF, ProhibitedKind::Noncharacter),
    (0xFFFE, ProhibitedKind::Noncharacter),
    (0xFFFF, ProhibitedKind::Noncharacter),
    (0x1FFFE, ProhibitedKind::Noncharacter),
    (0x10FFFE, ProhibitedKind::Noncharacter),
    (0x10FFFF, ProhibitedKind::Noncharacter),
];
const UNASSIGNED: &[(u32, ProhibitedKind)] = &[
    (0x0378, ProhibitedKind::Unassigned),
    (0x0380, ProhibitedKind::Unassigned),
    (0x0590, ProhibitedKind::Unassigned),
    (0x2FE0, ProhibitedKind::Unassigned),
];
const VARIATION_SELECTORS: &[(u32, ProhibitedKind)] = &[
    (0x180B, ProhibitedKind::VariationSelector),
    (0x180D, ProhibitedKind::VariationSelector),
    (0xFE00, ProhibitedKind::VariationSelector),
    (0xFE0F, ProhibitedKind::VariationSelector),
    (0xE0100, ProhibitedKind::VariationSelector),
    (0xE01EF, ProhibitedKind::VariationSelector),
];
// Representatives that are Default_Ignorable_Code_Point *and* assigned.
//
// This distinction matters: U+FFF0..U+FFF8 and the E0000-range tag characters
// are also unassigned, and SRC-0005 names unassigned code points before
// default-ignorable format characters, so those would legitimately be reported
// as `Unassigned`. Using assigned representatives keeps this table testing the
// `DefaultIgnorable` branch rather than the ordering rule.
const DEFAULT_IGNORABLE: &[(u32, ProhibitedKind)] = &[
    (0x00AD, ProhibitedKind::DefaultIgnorable),
    (0x034F, ProhibitedKind::DefaultIgnorable),
    (0x115F, ProhibitedKind::DefaultIgnorable),
    (0x200B, ProhibitedKind::DefaultIgnorable),
    (0xFFA0, ProhibitedKind::DefaultIgnorable),
];

// U+FEFF is deliberately *not* in the table above. It is a
// Default_Ignorable_Code_Point, so SRC-0005 would prohibit it, but SRC-0001 is
// more specific: "An optional UTF-8 BOM is accepted only at byte offset zero."
// A leading U+FEFF is therefore a BOM and is accepted; anywhere else it is a
// prohibited default-ignorable character. The interaction is asserted
// explicitly in `feff_is_a_bom_at_offset_zero_and_prohibited_elsewhere` rather
// than being hidden inside the generic sweep.
const BOM: (u32, ProhibitedKind) = (0xFEFF, ProhibitedKind::DefaultIgnorable);

/// Every prohibited representative, with the class each must be reported as.
fn all_cases() -> Vec<(u32, ProhibitedKind)> {
    let mut v = Vec::new();
    v.extend_from_slice(BIDI);
    v.extend_from_slice(NONCHARACTERS);
    v.extend_from_slice(UNASSIGNED);
    v.extend_from_slice(VARIATION_SELECTORS);
    v.extend_from_slice(DEFAULT_IGNORABLE);
    v
}

fn push_span(out: &mut Vec<u8>, bytes: &[u8], span: Span) {
    assert!(span.start <= span.end, "span must be half-open");
    assert!(span.end as usize <= bytes.len(), "span must be within the input");
    out.extend_from_slice(&bytes[span.start as usize..span.end as usize]);
}

/// Scan `bytes`, assert the stream reconstructs the input exactly, and return
/// the tokens. Losslessness is asserted on *every* call, so no security test
/// can pass by dropping a byte.
fn reconstruct(bytes: &[u8]) -> Vec<Token> {
    let mut scanner = Scanner::new(bytes, 0);
    let mut out = Vec::with_capacity(bytes.len() + 16);
    let mut tokens = Vec::new();
    while let Some(token) = scanner.next_token() {
        for tr in &token.leading_trivia {
            push_span(&mut out, bytes, tr.span);
        }
        push_span(&mut out, bytes, token.span);
        for tr in &token.trailing_trivia {
            push_span(&mut out, bytes, tr.span);
        }
        tokens.push(token);
    }
    for tr in scanner.take_eof_trivia() {
        push_span(&mut out, bytes, tr.span);
    }
    assert_eq!(out.as_slice(), bytes, "reconstruction must stay byte-exact");
    tokens
}

fn error_tokens(tokens: &[Token]) -> Vec<&Token> {
    tokens.iter().filter(|t| t.kind == TokenKind::Error).collect()
}

/// The single error token in `tokens`, with its reason.
fn only_error(tokens: &[Token], context: &str) -> ErrorReason {
    let errs = error_tokens(tokens);
    assert_eq!(errs.len(), 1, "expected exactly one error for {context:?}, got {errs:?}");
    *errs[0].error_reason.as_ref().expect("an error token must state why it failed")
}

fn expect_prohibited(source: &str, expected: ProhibitedKind) {
    let tokens = reconstruct(source.as_bytes());
    match only_error(&tokens, source) {
        ErrorReason::ProhibitedSource { kind } => {
            assert_eq!(kind, expected, "wrong class for {source:?}");
        }
        other => panic!("expected a SRC-0005 error for {source:?}, got {other:?}"),
    }
}
// --------------------------------------------------------------------------
// Positive: prohibited characters stay legal where the rules say they are
// --------------------------------------------------------------------------

#[test]
fn prohibited_characters_are_legal_inside_literals() {
    // SRC-0005 scopes itself to "outside comments and literals". Inside a
    // literal, SRC-0007 makes every scalar value data and SRC-0006 does not
    // apply, so these are lexed normally with no error at all.
    for (cp, _) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        for (source, kind) in [
            (format!("\"a{c}b\""), TokenKind::String),
            (format!("r\"a{c}b\""), TokenKind::RawString),
            (format!("f\"a{c}b\""), TokenKind::InterpolatedString),
        ] {
            let tokens = reconstruct(source.as_bytes());
            assert!(error_tokens(&tokens).is_empty(), "U+{cp:04X} must be legal in {source:?}");
            assert_eq!(tokens[0].kind, kind);
        }
    }
}

#[test]
fn prohibited_characters_are_legal_inside_character_literals() {
    for (cp, _) in BIDI {
        let c = char::from_u32(*cp).expect("scalar");
        let source = format!("'{c}'");
        let tokens = reconstruct(source.as_bytes());
        assert!(error_tokens(&tokens).is_empty(), "U+{cp:04X} in a char literal");
        assert_eq!(tokens[0].kind, TokenKind::Char);
    }
}

#[test]
fn comments_containing_security_characters_are_reported_in_strict_mode() {
    // SRC-0006 covers exactly two classes: "bidi controls and invisible format
    // characters". Noncharacters and unassigned code points are SRC-0005
    // classes and are *not* SRC-0006, so a comment containing one is left to
    // the comment policy rather than reported here.
    let mut cases: Vec<(u32, ProhibitedKind)> = Vec::new();
    cases.extend_from_slice(BIDI);
    cases.extend_from_slice(DEFAULT_IGNORABLE);
    cases.push(BOM);

    for (cp, _) in cases {
        let c = char::from_u32(cp).expect("scalar");
        for source in [format!("// a{c}b\n"), format!("/* a{c}b */")] {
            let tokens = reconstruct(source.as_bytes());
            let found = tokens
                .iter()
                .any(|t| t.error_reason == Some(ErrorReason::UnannotatedCommentSecurity));
            assert!(found, "U+{cp:04X} in {source:?} must be reported by SRC-0006");
        }
    }
}

#[test]
fn noncharacters_in_comments_are_not_a_src_0006_finding() {
    // The negative half of the scope boundary: SRC-0006 does not reach these.
    for (cp, _) in NONCHARACTERS {
        let c = char::from_u32(*cp).expect("scalar");
        let tokens = reconstruct(format!("// a{c}b\n").as_bytes());
        assert!(
            error_tokens(&tokens).is_empty(),
            "U+{cp:04X} in a comment is not an SRC-0006 finding"
        );
    }
}

#[test]
fn lenient_mode_accepts_comments_but_still_preserves_the_bytes() {
    // Lenient mode is for tooling that must not fail a build. The characters
    // are still carried: the comment trivia keeps its exact span.
    let source = "// a\u{202E}b\nfn f() {}\n";
    let bytes = source.as_bytes();
    let mut scanner = Scanner::with_security_mode(bytes, 0, SecurityMode::Lenient);
    let mut out = Vec::new();
    while let Some(t) = scanner.next_token() {
        for tr in &t.leading_trivia {
            out.extend_from_slice(&bytes[tr.span.start as usize..tr.span.end as usize]);
        }
        out.extend_from_slice(&bytes[t.span.start as usize..t.span.end as usize]);
        for tr in &t.trailing_trivia {
            out.extend_from_slice(&bytes[tr.span.start as usize..tr.span.end as usize]);
        }
        assert_ne!(t.error_reason, Some(ErrorReason::UnannotatedCommentSecurity));
    }
    for tr in scanner.take_eof_trivia() {
        out.extend_from_slice(&bytes[tr.span.start as usize..tr.span.end as usize]);
    }
    assert_eq!(out, bytes, "lenient mode must still round-trip exactly");
}

#[test]
fn an_annotated_comment_is_accepted() {
    // SRC-0006 requires a *visible escaped annotation*. The annotation reuses
    // the LEX-0007 `\u{...}` escape, so it is pure ASCII and carries no
    // security-significant character of its own.
    let source = "// a\\u{202E}b\nfn f() {}\n";
    let tokens = reconstruct(source.as_bytes());
    assert!(error_tokens(&tokens).is_empty(), "an annotated comment must be accepted: {tokens:?}");
}

// --------------------------------------------------------------------------
// Negative: every prohibited class is a source error in source position
// --------------------------------------------------------------------------

#[test]
fn every_prohibited_class_is_rejected_standalone() {
    for (cp, expected) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        expect_prohibited(&c.to_string(), expected);
    }
}

#[test]
fn every_prohibited_class_is_rejected_at_the_start_of_an_identifier() {
    for (cp, expected) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        expect_prohibited(&format!("{c}name"), expected);
    }
}

#[test]
fn every_prohibited_class_is_rejected_in_the_middle_of_an_identifier() {
    for (cp, expected) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        expect_prohibited(&format!("na{c}me"), expected);
    }
}

#[test]
fn every_prohibited_class_is_rejected_at_the_end_of_an_identifier() {
    for (cp, expected) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        expect_prohibited(&format!("name{c}"), expected);
    }
}

#[test]
fn every_prohibited_class_is_rejected_next_to_punctuation() {
    for (cp, expected) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        expect_prohibited(&format!("a{c};"), expected);
        expect_prohibited(&format!(";{c}a"), expected);
    }
}

#[test]
fn every_prohibited_class_is_rejected_next_to_whitespace() {
    for (cp, expected) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        expect_prohibited(&format!("a {c} b"), expected);
    }
}
#[test]
fn repeated_prohibited_characters_are_all_reported() {
    // Every occurrence gets its own error token; none is silently skipped.
    let tokens = reconstruct("a\u{202E}\u{202E}\u{202E}b".as_bytes());
    let errs = error_tokens(&tokens);
    assert_eq!(errs.len(), 3, "each occurrence is reported separately");
    for e in &errs {
        assert_eq!(e.span.end - e.span.start, 3, "U+202E is three UTF-8 bytes");
    }
}

#[test]
fn prohibited_characters_inside_a_raw_identifier_are_rejected() {
    let tokens = reconstruct("r#a\u{202E}".as_bytes());
    let reasons: Vec<ErrorReason> =
        error_tokens(&tokens).iter().filter_map(|t| t.error_reason).collect();
    assert!(
        reasons.contains(&ErrorReason::ProhibitedSource { kind: ProhibitedKind::BidiControl }),
        "a raw identifier must not smuggle a bidi control past SRC-0005, got {reasons:?}"
    );
}

#[test]
fn the_error_span_covers_the_offending_code_point() {
    // A prohibited code point is reported as a token that *contains* it. When
    // the scanner rejected the leading character directly, the span is exactly
    // the code point; when the character was absorbed into an identifier (a
    // variation selector is XID_Continue), the whole identifier is the error
    // token so that lossless reconstruction is preserved.
    for (cp, _) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        let width = c.len_utf8() as u32;
        let start = 2u32;
        let source = format!("ab{c}cd");
        let tokens = reconstruct(source.as_bytes());
        let err = error_tokens(&tokens).first().copied().expect("an error");
        assert!(err.span.start <= start, "span must start at or before U+{cp:04X}");
        assert!(err.span.end >= start + width, "span must cover all {width} bytes of U+{cp:04X}");
    }
}

#[test]
fn a_standalone_prohibited_code_point_has_an_exact_span() {
    // With nothing to absorb it, the span is exactly the code point.
    for (cp, expected) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        let source = format!(" {c} ");
        let tokens = reconstruct(source.as_bytes());
        let err = error_tokens(&tokens).first().copied().expect("an error");
        assert_eq!(err.span.start, 1, "exact start for U+{cp:04X}");
        assert_eq!(err.span.end, 1 + c.len_utf8() as u32, "exact end for U+{cp:04X}");
        assert_eq!(
            err.error_reason,
            Some(ErrorReason::ProhibitedSource { kind: expected }),
            "U+{cp:04X} must be reported as {expected}, got {:?}",
            err.error_reason
        );
    }
}

#[test]
fn feff_is_a_bom_at_offset_zero_and_prohibited_elsewhere() {
    // SRC-0001 accepts a BOM only at byte offset zero, and SRC-0001 is the more
    // specific rule for exactly that position.
    let bom = char::from_u32(BOM.0).expect("scalar");

    let source = format!("{bom}let x = 1;");
    let tokens = reconstruct(source.as_bytes());
    assert!(error_tokens(&tokens).is_empty(), "a leading BOM is accepted by SRC-0001");

    // Anywhere else it is a prohibited default-ignorable format character.
    expect_prohibited(&format!("let x = 1;{bom}"), BOM.1);
    expect_prohibited(&format!("a{bom}"), BOM.1);
    // Two BOMs: the first is a BOM, the second is prohibited.
    let tokens = reconstruct(format!("{bom}{bom}").as_bytes());
    let reasons: Vec<ErrorReason> =
        error_tokens(&tokens).iter().filter_map(|t| t.error_reason).collect();
    assert!(
        reasons.contains(&ErrorReason::ProhibitedSource { kind: BOM.1 }),
        "a second BOM is not a BOM, got {reasons:?}"
    );
}

#[test]
fn the_reported_class_is_the_first_class_src_0005_names() {
    // U+061C is both Bidi_Control and Default_Ignorable_Code_Point. SRC-0005
    // names bidi controls first, so that is what must be reported, every time.
    for _ in 0..8 {
        expect_prohibited("a\u{061C}b", ProhibitedKind::BidiControl);
        expect_prohibited("a\u{FE0F}b", ProhibitedKind::VariationSelector);
    }
}

// --------------------------------------------------------------------------
// Interaction with malformed UTF-8
// --------------------------------------------------------------------------

#[test]
fn malformed_utf8_is_still_reported_as_malformed_utf8() {
    // SRC-0001 must not be conflated with SRC-0005.
    let tokens = reconstruct(b"a\xFFb");
    assert_eq!(only_error(&tokens, "a FF b"), ErrorReason::MalformedUtf8);
}

#[test]
fn malformed_utf8_and_prohibited_characters_coexist_without_confusion() {
    let mut bytes = "a\u{202E}".as_bytes().to_vec();
    bytes.push(0xFF);
    bytes.extend_from_slice(b"b");
    let tokens = reconstruct(&bytes);
    let reasons: Vec<ErrorReason> =
        error_tokens(&tokens).iter().filter_map(|t| t.error_reason).collect();
    assert_eq!(
        reasons,
        vec![
            ErrorReason::ProhibitedSource { kind: ProhibitedKind::BidiControl },
            ErrorReason::MalformedUtf8
        ],
        "the two classes must be reported distinctly and in source order"
    );
}

#[test]
fn malformed_utf8_inside_a_string_still_breaks_the_literal() {
    let tokens = reconstruct(b"\"a\xFFb\"");
    assert!(!error_tokens(&tokens).is_empty(), "malformed UTF-8 inside a string is an error");
}
// --------------------------------------------------------------------------
// Interaction with line endings, BOM, and ordinary source
// --------------------------------------------------------------------------

#[test]
fn prohibited_characters_interact_correctly_with_crlf_and_cr() {
    for eol in ["\n", "\r\n", "\r"] {
        let source = format!("// c{eol}let x = 1;{eol}");
        let tokens = reconstruct(source.as_bytes());
        assert!(error_tokens(&tokens).is_empty(), "comment with {eol:?} stays legal");
        let source = format!("let x = 1;{eol}\u{202E}{eol}");
        expect_prohibited(&source, ProhibitedKind::BidiControl);
    }
}

#[test]
fn prohibited_characters_do_not_break_bom_handling() {
    let mut bytes = vec![0xEFu8, 0xBB, 0xBF];
    bytes.extend_from_slice("let x = 1;".as_bytes());
    let tokens = reconstruct(&bytes);
    assert!(error_tokens(&tokens).is_empty(), "a BOM is not prohibited");

    let mut bytes = vec![0xEFu8, 0xBB, 0xBF];
    bytes.extend_from_slice("\u{202E}".as_bytes());
    let tokens = reconstruct(&bytes);
    assert_eq!(
        only_error(&tokens, "BOM then RLO"),
        ErrorReason::ProhibitedSource { kind: ProhibitedKind::BidiControl }
    );
}

#[test]
fn ordinary_unicode_identifiers_are_untouched() {
    for source in [
        "let \u{65E5}\u{672C}\u{8A9E} = 1;",
        "let caf\u{e9} = 1;",
        "let e\u{301} = 1;",
        "let \u{3B1}\u{3B2}\u{3B3} = 1;",
        "let _ = 1;",
    ] {
        let tokens = reconstruct(source.as_bytes());
        assert!(
            error_tokens(&tokens).is_empty(),
            "SRC-0003 requires combining marks to stay legal in identifiers: {source:?}"
        );
    }
}

#[test]
fn a_valid_program_still_produces_no_errors() {
    let source = "fn main(a: i32) -> i32 { let x = add(a, 2) * 3; return x; }";
    let tokens = reconstruct(source.as_bytes());
    assert!(error_tokens(&tokens).is_empty(), "{source}");
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Punct(Punct::Semicolon)));
}

// --------------------------------------------------------------------------
// Determinism and no-crash
// --------------------------------------------------------------------------

#[test]
fn classification_is_stable_across_repeated_scans() {
    let source = "a\u{202E}\u{FDD0}\u{FE0F}\u{0378}\u{00AD}b";
    let shape = |tokens: &[Token]| -> Vec<(TokenKind, u32, u32, Option<ErrorReason>)> {
        tokens.iter().map(|t| (t.kind, t.span.start, t.span.end, t.error_reason)).collect()
    };
    let first = shape(&reconstruct(source.as_bytes()));
    for _ in 0..16 {
        assert_eq!(shape(&reconstruct(source.as_bytes())), first, "scanning must be deterministic");
    }
}

#[test]
fn no_scalar_value_panics_the_scanner() {
    // The SRC-0005 no-crash guarantee end to end through the lexer. The
    // classifier is swept exhaustively in `omni-unicode`; here every scalar is
    // still put through the real scanner, and every prohibited code point is
    // additionally exercised in each position the rules distinguish.
    for cp in 0..=0x10FFFFu32 {
        let Some(c) = char::from_u32(cp) else { continue };
        let _ = reconstruct(c.to_string().as_bytes());
    }
    for (cp, _) in all_cases() {
        let c = char::from_u32(cp).expect("scalar");
        for template in ["a{c}", "{c}a", "a{c}b", "// {c}\n", "\"a{c}\"", "/* {c} */", "r#a{c}"] {
            let source = template.replace("{c}", &c.to_string());
            let _ = reconstruct(source.as_bytes());
        }
    }
}

#[test]
fn trivia_ownership_is_unaffected_by_security_scanning() {
    // A security error still carries the trivia that preceded it, so a
    // formatter or CST builder can still reproduce the source exactly.
    let source = "  // note\n  \u{202E}let x = 1;";
    let tokens = reconstruct(source.as_bytes());
    let err = error_tokens(&tokens).first().copied().expect("an error");
    let trivias: Vec<&Trivia> = err.leading_trivia.iter().collect();
    assert!(!trivias.is_empty(), "leading trivia must be preserved on the error token");
}
