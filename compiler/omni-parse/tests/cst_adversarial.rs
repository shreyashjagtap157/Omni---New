//! Independent adversarial probe for the 0.0.2.2 CST contracts.
//!
//! This harness deliberately does NOT reuse `omni_parse`'s internal test
//! helpers: it re-derives every expectation from the public API and from the
//! lexer's own spans, so a defect in the implementation's own accounting cannot
//! mask itself.

use omni_lex::{Scanner, TokenKind};
use omni_parse::Parser;
use omni_syntax::{SyntaxElement, SyntaxKind};

fn lexer_intervals(src: &str) -> Vec<(u32, u32)> {
    let mut scanner = Scanner::from_str(src, 0);
    let mut out = Vec::new();
    while let Some(t) = scanner.next_token() {
        for tr in &t.leading_trivia {
            out.push((tr.span.start, tr.span.end));
        }
        if t.kind != TokenKind::Eof {
            out.push((t.span.start, t.span.end));
        }
        for tr in &t.trailing_trivia {
            out.push((tr.span.start, tr.span.end));
        }
    }
    for tr in scanner.take_eof_trivia() {
        out.push((tr.span.start, tr.span.end));
    }
    out
}

/// The full CST contract, derived independently of the implementation's own
/// test helpers.
fn check(src: &str) {
    let mut p = Parser::from_source(src);
    let r = p.parse_source();
    let syntax = r.syntax();

    assert_eq!(syntax.text().to_string(), src, "exact reconstruction of {src:?}");

    // Tiling: leaves tile the source exactly once, in order.
    let mut cursor: u32 = 0;
    let mut leaves = Vec::new();
    for el in syntax.descendants_with_tokens() {
        if let SyntaxElement::Token(t) = el {
            let s: u32 = t.text_range().start().into();
            let e: u32 = t.text_range().end().into();
            assert_eq!(s, cursor, "gap/overlap at leaf {:?} for {src:?}", t.text());
            assert_eq!(&src[s as usize..e as usize], t.text(), "leaf text mismatch in {src:?}");
            leaves.push((s, e, t.kind()));
            cursor = e;
        }
    }
    assert_eq!(cursor as usize, src.len(), "leaves cover {cursor}/{} of {src:?}", src.len());

    for (_, _, k) in &leaves {
        assert!(*k != SyntaxKind::Unknown, "an unrecognized raw kind appeared in {src:?}");
    }

    // Byte offsets, not character counts.
    for (s, e, _) in &leaves {
        assert!(src.is_char_boundary(*s as usize), "leaf split a char at {s} in {src:?}");
        assert!(src.is_char_boundary(*e as usize), "leaf split a char at {e} in {src:?}");
    }

    // Zero-width leaves are only ever the absent-token marker.
    for (s, e, k) in &leaves {
        if s == e {
            assert!(*k == SyntaxKind::MissingToken, "unexpected zero-width leaf {k:?} in {src:?}");
        }
    }

    // The lexer's own spans must be in range for this source.
    for (s, e) in lexer_intervals(src) {
        assert!(s <= e && e as usize <= src.len(), "lexer span out of range in {src:?}");
    }

    // Determinism: shape, text, and diagnostics all repeat exactly.
    let snapshot = |p: &mut Parser| {
        let r = p.parse_source();
        let shape: Vec<u16> =
            r.syntax().descendants_with_tokens().map(|e| e.kind().to_raw().0).collect();
        (shape, r.syntax().text().to_string(), r.diagnostics.clone())
    };
    let first = snapshot(&mut Parser::from_source(src));
    let second = snapshot(&mut Parser::from_source(src));
    assert_eq!(first.0, second.0, "nondeterministic shape for {src:?}");
    assert_eq!(first.1, second.1, "nondeterministic text for {src:?}");
    assert_eq!(first.2, second.2, "nondeterministic diagnostics for {src:?}");

    let mut p3 = Parser::from_source(src);
    p3.parse_source();
    assert_eq!(first.0, snapshot(&mut p3).0, "re-parse diverged for {src:?}");
}

fn check_malformed(src: &str) {
    check(src);
    let mut p = Parser::from_source(src);
    assert!(!p.parse_source().is_ok(), "malformed input {src:?} was silently accepted");
}

#[test]
fn probe_basic_and_valid() {
    for src in [
        "",
        " ",
        "\n",
        "\t\t",
        "// only\n",
        "/// doc only\n",
        "//! crate\n",
        "/* block */",
        "\u{FEFF}",
        "\u{FEFF}\u{FEFF}",
        "fn f() {}",
        "fn f() { return 1; }",
        "fn f(a: i32) -> i32 { return a; }",
        "fn f() { let x = 1; }",
        "fn f() { let mut x: i32 = 2; return x; }",
        "fn f() { g(1, 2); }",
        "fn f() { -x; }",
        "fn f() { !x; }",
        "fn f() { &x; }",
        "fn f() { a + b * c; }",
        "fn f() { (1 + 2) * 3; }",
        "fn f() { return 'c'; }",
        "fn f() { let \u{00e9}t\u{00e9} = 1; }",
        "fn f() { // c\n return 1; }",
        "fn f() { /* a */ return /* b */ 1; /* c */ }",
        "fn f() { return 1; } // tail",
        "fn f() { return 1; }\n\n// tail\n",
    ] {
        check(src);
    }
}

#[test]
fn probe_unicode_spans() {
    for src in [
        "fn f() { return \"\u{00e9}\u{4e2d}\u{6587}\"; }",
        "fn f() { return '\u{1f600}'; }",
        "fn f() { return \"\u{1f600}\"; }",
        "fn \u{00e9}() { return \"\u{0301}\u{0300}\"; }",
        "fn f() { /* \u{4e2d}\u{6587} \u{1f600} */ return 1; }",
        "fn f() { let \u{03b4} = 1; }",
        "\u{FEFF}fn f() { let \u{00e9} = \"\u{4e2d}\u{6587}\"; }",
    ] {
        check(src);
        let mut p = Parser::from_source(src);
        assert!(p.parse_source().is_ok(), "valid Unicode source {src:?} must be accepted");
    }
    // An emoji is category So, not XID_Start, so it is not an identifier. It
    // must still round-trip losslessly and still be diagnosed.
    check_malformed("fn f() { let \u{1f600} = 1; }");
    // An unterminated multi-byte string is both lossless and diagnosed.
    check_malformed("fn f() { return \"\u{00e9}");
    check_malformed("fn f() { return '\u{4e2d}");
}

#[test]
fn probe_recovery_shapes() {
    for src in [
        "fn main( { return 1; }",
        "fn f() {\n// c",
        "fn",
        "fn ",
        "fn f",
        "fn { }",
        "fn f(",
        "fn f()",
        "fn f() {",
        "fn f() ->",
        "fn f() -> i32",
        "fn f(,) {}",
        "fn f(a,) {}",
        "fn f(a: ) {}",
        "fn f(a i32) {}",
        "fn f() { let = 1; }",
        "fn f() { let x = ; }",
        "fn f() { let x 1; }",
        "fn f() { return }",
        "fn f() { return 1 }",
        "fn f() { 1 2 3 }",
        "fn f() { + * ! }",
        "fn f() { ) }",
        "fn f() { ) ( }",
        "fn f() {} }",
        "}}}}",
        "))",
        "fn f() { @@@ }",
        "@@@ fn g() {}",
        "fn f() {} @@@ fn g() {} @@@",
        "fn f() { /* unterminated",
        "fn f() { return 1; } /* unterminated",
        "fn f() { \"unterminated",
        "fn f() { 'x",
        "fn f() { @@@ /* inner */ @@@ }",
        "fn f() { @@@ // c\n @@@ }",
        "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ fn g() {}",
        "fn f() {\n    return 1;\n",
        "fn f() { let x = 1;\n// c",
        "fn f(a: i32,\n// c\n",
        "fn f() { return f(1,2",
        "fn f() { return f(1,2)\n// c",
    ] {
        check_malformed(src);
    }
}

#[test]
fn probe_lexical_error_preservation() {
    for src in [
        "fn f() { return 1; } /* unterminated",
        "/* unterminated",
        "fn f() { return \"abc",
        "\u{FEFF}/* unterminated",
    ] {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        let has_err = r
            .syntax()
            .descendants_with_tokens()
            .any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == SyntaxKind::ErrorToken));
        assert!(has_err, "lexical error lost for {src:?}");
        assert_eq!(r.syntax().text().to_string(), src);
    }
}

#[test]
fn probe_randomized_sources() {
    // Deterministic LCG; any failure is reproducible from this file alone.
    let atoms = [
        "fn", "f", "(", ")", "{", "}", "let", "mut", "return", "1", "x", "+", "*", "-", ";", ",",
        ":", "->", "=", "@", "struct", "enum", "true", "1.5", "&&", "// c\n", "/* b */", "/// d\n",
        "\u{00e9}", "\u{4e2d}", "&&&", "!", "&", "|", "<", ">", "==", "..", "\\",
    ];
    let mut seed: u64 = 0x9E3779B97F4A7C15;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    for _ in 0..20_000 {
        let n = next() % 14;
        let mut s = String::new();
        if next() % 5 == 0 {
            s.push('\u{FEFF}');
        }
        for _ in 0..n {
            s.push_str(atoms[next() % atoms.len()]);
            match next() % 4 {
                0 => s.push(' '),
                1 => s.push('\n'),
                _ => {}
            }
        }
        check(&s);
    }
}

#[test]
fn probe_random_bytes_are_lossless() {
    // Arbitrary byte soup exercises the SRC-0001 malformed-UTF-8 path.
    let mut seed: u64 = 0xDEADBEEF12345678;
    let mut next = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (seed >> 33) as usize
    };
    for _ in 0..3_000 {
        let n = 1 + next() % 24;
        let bytes: Vec<u8> = (0..n).map(|_| (next() % 256) as u8).collect();
        let lossy = String::from_utf8_lossy(&bytes).into_owned();
        let mut p = Parser::from_source(&lossy);
        assert_eq!(p.parse_source().syntax().text().to_string(), lossy);
    }
}

#[test]
fn probe_long_source() {
    let mut src = String::new();
    for i in 0..2000 {
        src.push_str(&format!("fn f{i}() {{ return {i}; }}\n"));
    }
    src.push_str("fn unclosed() {\n// trailing\n");
    check(&src);
    check_malformed(&src);
}

#[test]
fn probe_deep_nesting_terminates() {
    for src in [
        format!("fn f() {{ {} }}", "(".repeat(400)),
        format!("fn f({})", "a,".repeat(400)),
        "{".repeat(400),
        format!("fn f() {{ {} 1; }}", "-".repeat(300)),
        format!("fn f() {{ return {}1; }}", "!".repeat(300)),
        format!("fn f() {} {{}}", "(".repeat(300)),
        format!("fn f() {{ return f({}); }}", ",".repeat(300)),
    ] {
        let mut p = Parser::from_source(&src);
        let r = p.parse_source();
        assert_eq!(r.syntax().text().to_string(), src);
    }
}

/// Trivia must appear in the CST exactly as many times as the lexer produced
/// it -- neither dropped nor duplicated -- and must total the same byte count.
#[test]
fn probe_trivia_exactly_once() {
    for src in [
        "fn f() {\n// a\n// b\n}",
        "fn f() {\n/// d\n/* x */ // y\nreturn 1;\n}",
        "\n\n// lead\nfn f() {\n// in\nreturn 1; // eol\n}\n// tail\n",
        "fn f() {\n// c",
        "fn f() {\n/// d",
        "fn f() {\n   ",
        "/* unterminated",
        "fn f() { /* unterminated",
        "fn f() { @@@ /* unterminated",
        "\u{FEFF}// doc\nfn f() {}\n",
    ] {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        let trivia: Vec<String> = r
            .syntax()
            .descendants_with_tokens()
            .filter_map(|e| match e {
                SyntaxElement::Token(t)
                    if matches!(
                        t.kind(),
                        SyntaxKind::Whitespace
                            | SyntaxKind::LineComment
                            | SyntaxKind::BlockComment
                            | SyntaxKind::DocComment
                    ) =>
                {
                    Some(t.text().to_string())
                }
                _ => None,
            })
            .collect();
        let cst_bytes: usize = trivia.iter().map(|t| t.len()).sum();

        let mut scanner = Scanner::from_str(src, 0);
        let mut lexer_count = 0usize;
        let mut lexer_bytes = 0usize;
        while let Some(t) = scanner.next_token() {
            // Trivia only: the token's own span is accounted for separately by
            // the tiling check, not here.
            for tr in t.leading_trivia.iter().chain(t.trailing_trivia.iter()) {
                lexer_count += 1;
                lexer_bytes += (tr.span.end - tr.span.start) as usize;
            }
        }
        for tr in scanner.take_eof_trivia() {
            lexer_count += 1;
            lexer_bytes += (tr.span.end - tr.span.start) as usize;
        }

        assert_eq!(trivia.len(), lexer_count, "trivia count mismatch for {src:?}");
        assert_eq!(cst_bytes, lexer_bytes, "trivia byte total mismatch for {src:?}");
    }
}

#[test]
fn probe_diagnostic_spans_are_real_intervals() {
    for src in ["fn f() { let = 1; }", "fn f() { return 1 }", "@@@ fn g() {}"] {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(!r.diagnostics.is_empty(), "{src:?} produced no diagnostics");
        for d in &r.diagnostics {
            if let Some(s) = d.span {
                assert!(s.start <= s.end, "inverted diagnostic span for {d:?}");
                assert!(s.end as usize <= src.len(), "diagnostic span past EOF for {src:?}");
            }
        }
    }
}

#[test]
fn probe_error_token_survives_into_the_cst() {
    for src in ["fn f() { return 1; } /* unterminated", "fn f() { return \"abc", "fn f() { 'x"] {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        let kinds: Vec<SyntaxKind> = r
            .syntax()
            .descendants_with_tokens()
            .filter_map(|e| match e {
                SyntaxElement::Token(t) => Some(t.kind()),
                _ => None,
            })
            .collect();
        assert!(kinds.contains(&SyntaxKind::ErrorToken), "no ErrorToken in {src:?}: {kinds:?}");
        assert_eq!(r.syntax().text().to_string(), src);
    }
}
