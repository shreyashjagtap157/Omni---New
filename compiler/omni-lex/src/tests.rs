//! Workstream 0.0.1 acceptance and rejection tests.
//!
//! Covers byte-oriented entry, malformed UTF-8, BOM placement, line-ending
//! normalization, exact byte spans and end-of-input at every lexical state
//! (workstream A), then Unicode identifiers, the normative keyword table and
//! raw identifiers (workstream B).

use crate::scanner::Scanner;
use crate::token::{Kw, Punct, Span, Token, TokenKind, TriviaKind};

/// Reconstructs `bytes` from the tokens and trivia the scanner produced.
fn reconstruct(bytes: &[u8]) -> Vec<u8> {
    let mut scanner = Scanner::new(bytes, 0);
    let mut out = Vec::with_capacity(bytes.len() + 16);
    while let Some(token) = scanner.next_token() {
        push_trivia(&mut out, bytes, &token.leading_trivia);
        push_span(&mut out, bytes, token.span);
        push_trivia(&mut out, bytes, &token.trailing_trivia);
    }
    push_trivia(&mut out, bytes, &scanner.take_eof_trivia());
    out
}

fn push_span(out: &mut Vec<u8>, bytes: &[u8], span: Span) {
    assert_eq!(span.file_id, 0, "scanner must stamp its own file id");
    assert!(span.start <= span.end, "span must be half-open");
    assert!(
        span.end as usize <= bytes.len(),
        "span {}..{} exceeds {} input bytes",
        span.start,
        span.end,
        bytes.len()
    );
    out.extend_from_slice(&bytes[span.start as usize..span.end as usize]);
}

fn push_trivia(out: &mut Vec<u8>, bytes: &[u8], trivia: &[crate::token::Trivia]) {
    for tr in trivia {
        push_span(out, bytes, tr.span);
    }
}

/// Asserts the byte stream round-trips and returns the token stream.
fn assert_lossless(bytes: &[u8]) -> Vec<Token> {
    let mut scanner = Scanner::new(bytes, 0);
    let mut out = Vec::with_capacity(bytes.len() + 16);
    let mut tokens = Vec::new();
    while let Some(token) = scanner.next_token() {
        push_trivia(&mut out, bytes, &token.leading_trivia);
        push_span(&mut out, bytes, token.span);
        push_trivia(&mut out, bytes, &token.trailing_trivia);
        tokens.push(token);
    }
    push_trivia(&mut out, bytes, &scanner.take_eof_trivia());
    assert_eq!(out.as_slice(), bytes, "reconstruction must be byte-exact");
    tokens
}

fn spans_of(bytes: &[u8]) -> Vec<(TokenKind, u32, u32)> {
    assert_lossless(bytes).into_iter().map(|t| (t.kind, t.span.start, t.span.end)).collect()
}

// --------------------------------------------------------------------------
// Byte-oriented entry
// --------------------------------------------------------------------------

#[test]
fn byte_entry_is_the_primary_constructor() {
    let tokens = spans_of(b"let x = 42;");
    assert_eq!(
        tokens,
        vec![
            (TokenKind::Keyword(crate::token::Kw::Let), 0, 3),
            (TokenKind::Ident, 4, 5),
            (TokenKind::Punct(crate::token::Punct::Eq), 6, 7),
            (TokenKind::Int, 8, 10),
            (TokenKind::Punct(crate::token::Punct::Semicolon), 10, 11),
        ]
    );
}

#[test]
fn from_str_is_byte_identical_to_byte_entry() {
    let text = "fn main(a: i32) -> i32 { return 1; }";
    let mut from_text = Scanner::from_str(text, 3);
    let mut from_bytes = Scanner::new(text.as_bytes(), 3);

    loop {
        let a = from_text.next_token();
        let b = from_bytes.next_token();
        assert_eq!(a, b, "text and byte entry must agree token for token");
        if a.is_none() {
            break;
        }
    }
    assert_eq!(from_text.take_eof_trivia(), from_bytes.take_eof_trivia());
}

#[test]
fn arbitrary_bytes_never_panic_and_reconstruct() {
    let corpus: &[&[u8]] = &[
        b"",
        b"\x00",
        b"\x00\x00\x00",
        b"\xFF",
        b"\x80\xFF\xC2abc",
        b"\xC2",
        b"\xC2\x41",
        b"\xE0\x80\x80",
        b"\xE0\xA0\x80",
        b"\xED\xA0\x80",
        b"\xF5\x80\x80\x80",
        b"\xF0\x9F\xA6\x80",
        &[0xEF, 0xBB, 0xBF],
        &[0xEF, 0xBB, 0xBF, 0xEF, 0xBB, 0xBF],
        &[0xEF, 0xBB, 0xBF, 0xFF, 0xFE],
        &[0xC0, 0x80],
        &[0x80, 0x00, 0xFF],
        b"let x = \xFF\xFE 1;",
        b"unterminated \"string",
        b"unterminated 'ch",
        b"/* unterminated",
        b"r#\"unterminated raw",
        b"b'\xFF'",
        b"0x",
        b"0b",
        b"1__0",
        b"..",
        b"...",
        b"<<>>",
        b"????",
        b"///",
        b"#!",
    ];
    for bytes in corpus {
        assert_lossless(bytes);
    }
}

// --------------------------------------------------------------------------
// Malformed UTF-8
// --------------------------------------------------------------------------

#[test]
fn malformed_utf8_is_scanned_as_replacement_errors_with_exact_spans() {
    // 0x80 and 0xFF are never valid lead/continuation bytes.
    let tokens = assert_lossless(b"a\x80\xFFz");
    assert_eq!(tokens.len(), 4, "each malformed byte is its own token");
    assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, 1));
    assert_eq!((tokens[1].span.start, tokens[1].span.end), (1, 2));
    assert_eq!((tokens[2].span.start, tokens[2].span.end), (2, 3));
    assert_eq!((tokens[3].span.start, tokens[3].span.end), (3, 4));
    assert_eq!(tokens[1].kind, TokenKind::Error);
    assert_eq!(tokens[2].kind, TokenKind::Error);
}

#[test]
fn truncated_multibyte_sequence_is_bounded_by_the_end_of_input() {
    // U+0645 is D9 85; drop the continuation byte.
    let bytes = "\u{0645}".as_bytes();
    let truncated = &bytes[..1];
    let tokens = assert_lossless(truncated);
    assert_eq!(tokens.len(), 1);
    assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, 1));
    assert_eq!(tokens[0].kind, TokenKind::Error);
}

#[test]
fn valid_multibyte_sequences_keep_their_multi_byte_spans() {
    let text = "let \u{03B1} = 1;".as_bytes();
    let tokens = assert_lossless(text);
    // XID_Start acceptance is workstream B; workstream A only owes the span.
    let greek = &tokens[1];
    assert_eq!((greek.span.start, greek.span.end), (4, 6), "\u{03B1} occupies two bytes");
    assert_eq!(&text[4..6], "\u{03B1}".as_bytes());
}

// --------------------------------------------------------------------------
// Byte-order mark
// --------------------------------------------------------------------------

#[test]
fn bom_at_offset_zero_is_span_covered_whitespace_trivia() {
    let bytes = b"\xEF\xBB\xBFlet x = 1;";
    let mut scanner = Scanner::new(bytes, 9);
    let token = scanner.next_token().expect("first token after the BOM");
    assert_eq!(token.kind, TokenKind::Keyword(crate::token::Kw::Let));
    assert_eq!((token.span.start, token.span.end), (3, 6), "BOM shifts real tokens");
    assert_eq!(token.leading_trivia.len(), 1);
    let bom = &token.leading_trivia[0];
    assert_eq!(bom.kind, TriviaKind::Whitespace);
    assert_eq!((bom.span.start, bom.span.end), (0, 3), "BOM bytes must stay covered");
    assert_eq!(bom.span.file_id, 9, "file id must propagate to trivia");

    assert_eq!(reconstruct(bytes), bytes);
}

#[test]
fn bom_only_source_reconstructs_through_eof_trivia() {
    let bytes = [0xEF, 0xBB, 0xBF];
    let mut scanner = Scanner::new(&bytes, 0);
    assert!(scanner.next_token().is_none(), "a BOM is trivia, not a token");
    let eof = scanner.take_eof_trivia();
    assert_eq!(eof.len(), 1);
    assert_eq!((eof[0].span.start, eof[0].span.end), (0, 3));
    assert_eq!(reconstruct(&bytes), &bytes[..]);
}

#[test]
fn bom_after_offset_zero_is_rejected_not_silently_swallowed() {
    let bytes = b"abc\xEF\xBB\xBF";
    let tokens = assert_lossless(bytes);
    let last = tokens.last().expect("trailing BOM produces a token");
    assert_eq!(last.kind, TokenKind::Error, "a mid-file BOM must not be accepted as trivia");
    assert_eq!((last.span.start, last.span.end), (3, 6), "the rejected bytes stay spanned");
}

#[test]
fn second_bom_in_the_file_is_rejected() {
    let bytes = b"\xEF\xBB\xBF\xEF\xBB\xBFx";
    let tokens = assert_lossless(bytes);
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!((tokens[0].span.start, tokens[0].span.end), (3, 6));
    assert_eq!(tokens[1].kind, TokenKind::Ident);
    assert_eq!((tokens[1].span.start, tokens[1].span.end), (6, 7));
}

// --------------------------------------------------------------------------
// Line-ending normalization
// --------------------------------------------------------------------------

#[test]
fn crlf_spans_reference_original_bytes_and_normalise_to_one_newline() {
    let bytes = b"let a\r\nlet b\rlet c\nlet d";
    let tokens = assert_lossless(bytes);
    let starts: Vec<u32> = tokens.iter().map(|t| t.span.start).collect();
    // Offsets are taken against the original bytes, so the two-byte `\r\n`
    // pushes every later token two bytes further than a normalised view.
    assert_eq!(starts, vec![0, 4, 7, 11, 13, 17, 19, 23]);
    assert_eq!(&bytes[5..7], b"\r\n", "CRLF is two source bytes");
    assert_eq!(&bytes[12..13], b"\r", "lone CR is one source byte");
    assert_eq!(&bytes[18..19], b"\n", "lone LF is one source byte");
    assert_eq!(tokens[2].span.start, 7, "CRLF must count as one line break, two bytes");
    assert_eq!(tokens[4].span.start, 13, "lone CR must count as one line break, one byte");
    assert_eq!(tokens[6].span.start, 19, "lone LF must count as one line break, one byte");
}

#[test]
fn crlf_inside_trivia_is_one_whitespace_trivia_spanning_both_bytes() {
    let bytes = b"a\r\nb";
    let mut scanner = Scanner::new(bytes, 0);
    let first = scanner.next_token().expect("a");
    assert_eq!((first.span.start, first.span.end), (0, 1));
    assert!(first.trailing_trivia.is_empty(), "trailing trivia stops before the newline");

    let second = scanner.next_token().expect("b");
    assert_eq!((second.span.start, second.span.end), (3, 4));
    assert_eq!(second.leading_trivia.len(), 1);
    let ws = &second.leading_trivia[0];
    assert_eq!(ws.kind, TriviaKind::Whitespace);
    assert_eq!((ws.span.start, ws.span.end), (1, 3), "span covers the raw \\r\\n bytes");
    assert_eq!(reconstruct(bytes), bytes);
}

#[test]
fn mixed_line_endings_all_normalize_to_lf() {
    let bytes = b"a\r\nb\rc\nd";
    let tokens = assert_lossless(bytes);
    let mut breaks: Vec<&[u8]> = Vec::new();
    for token in &tokens {
        for tr in &token.leading_trivia {
            if tr.kind == TriviaKind::Whitespace {
                breaks.push(&bytes[tr.span.start as usize..tr.span.end as usize]);
            }
        }
    }
    assert_eq!(breaks.len(), 3, "each line break is its own whitespace trivia span");
    assert_eq!(breaks[0], &b"\r\n"[..], "CRLF collapses to one LF but keeps both bytes");
    assert_eq!(breaks[1], &b"\r"[..], "lone CR collapses to LF");
    assert_eq!(breaks[2], &b"\n"[..], "lone LF is already LF");
}

// --------------------------------------------------------------------------
// Exact byte spans and trailing trivia at end of input
// --------------------------------------------------------------------------

#[test]
fn trailing_comment_after_final_newline_survives() {
    let bytes = b"let x = 42;\n\n// done\n";
    let tokens = assert_lossless(bytes);
    let last = tokens.last().expect("one token at minimum");
    assert_eq!((last.span.start, last.span.end), (10, 11));
    assert!(
        last.trailing_trivia.is_empty(),
        "trailing trivia stops at the newline, the tail belongs to EOF trivia"
    );

    let mut scanner = Scanner::new(bytes, 0);
    while scanner.next_token().is_some() {}
    let eof = scanner.take_eof_trivia();
    assert!(!eof.is_empty(), "the tail of the file must not be dropped");
    let covered: Vec<u8> = eof
        .iter()
        .flat_map(|t| bytes[t.span.start as usize..t.span.end as usize].iter().copied())
        .collect();
    assert_eq!(covered, b"\n\n// done\n");
}

#[test]
fn trivia_only_source_is_carried_by_eof_trivia() {
    let bytes = b"  \n// only a comment\n";
    let mut scanner = Scanner::new(bytes, 0);
    assert!(scanner.next_token().is_none());
    let eof = scanner.take_eof_trivia();
    let covered: Vec<u8> = eof
        .iter()
        .flat_map(|t| bytes[t.span.start as usize..t.span.end as usize].iter().copied())
        .collect();
    assert_eq!(covered, bytes);
    assert_eq!(reconstruct(bytes), bytes);
}

#[test]
fn empty_source_yields_no_tokens_and_no_eof_trivia() {
    let mut scanner = Scanner::new(b"", 0);
    assert!(scanner.next_token().is_none());
    assert!(scanner.take_eof_trivia().is_empty());
    assert!(Scanner::new(b"", 0).take_eof_trivia().is_empty());
}

#[test]
fn eof_trivia_is_drained_once() {
    let bytes = b"x\n";
    let mut scanner = Scanner::new(bytes, 0);
    assert!(scanner.next_token().is_some());
    assert!(scanner.next_token().is_none());
    assert_eq!(scanner.take_eof_trivia().len(), 1);
    assert!(scanner.take_eof_trivia().is_empty());
    // Re-querying past end of input must not resurrect or destroy anything.
    assert!(scanner.next_token().is_none());
    assert!(scanner.take_eof_trivia().is_empty());
}

// --------------------------------------------------------------------------
// End of input at every lexical state
// --------------------------------------------------------------------------

const SAMPLE: &[u8] = b"fn main(a: i32) -> i32 {\n    let x = 0xFF;\n    // comment\n    let s = \"a\\nb\";\n    let r = r#\"raw\"#;\n    let c = 'z';\n    return x;\n}\n";

#[test]
fn eof_at_every_prefix_of_a_rich_source_reconstructs() {
    for cut in 0..=SAMPLE.len() {
        let prefix = &SAMPLE[..cut];
        assert_lossless(prefix);
    }
}

#[test]
fn eof_inside_a_string_literal_terminates_cleanly() {
    let bytes = b"\"abc";
    let tokens = assert_lossless(bytes);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Error);
    assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, 4));
}

#[test]
fn eof_inside_a_block_comment_terminates_cleanly() {
    let bytes = b"a /* outer /* inner";
    let tokens = assert_lossless(bytes);
    assert_eq!(tokens.len(), 1, "an unterminated comment is trivia, not a token");
    let mut scanner = Scanner::new(bytes, 0);
    assert!(scanner.next_token().is_some());
    assert!(scanner.next_token().is_none());
    let eof = scanner.take_eof_trivia();
    assert!(eof.is_empty(), "the comment already ended inside a token's trailing trivia");
    let trailing = &tokens[0].trailing_trivia;
    assert_eq!(trailing.len(), 2, "whitespace then the unterminated block comment");
    assert_eq!(trailing[1].kind, TriviaKind::BlockComment);
    assert_eq!(trailing[1].span.end, bytes.len() as u32, "comment runs to end of input");
}

#[test]
fn eof_after_a_single_operator_is_not_swallowed() {
    let ops: &[&[u8]] = &[
        b"+", b"-", b"*", b"/", b"%", b"^", b"=", b"==", b"!=", b"<", b"<=", b">", b">=", b"->",
        b"=>", b"(", b")", b"{", b"}", b"[", b"]", b",", b":", b"::", b";", b"&", b"&&", b"|",
        b"||", b"|>", b".", b"..", b"..=", b"?", b"??", b"@", b"#", b"$",
    ];
    for op in ops {
        let tokens = assert_lossless(op);
        assert_eq!(
            tokens.len(),
            1,
            "operator {:?} must lex as one token",
            String::from_utf8_lossy(op)
        );
        assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, op.len() as u32));
    }
}

#[test]
fn long_tokens_and_long_trivia_stay_exact() {
    const LONG: usize = 64 * 1024;
    let mut bytes = Vec::new();
    bytes.extend(std::iter::repeat_n(b'a', LONG)); // 0..LONG
    bytes.push(b' '); // LONG..LONG+1
    bytes.extend_from_slice(b"// "); // LONG+1..LONG+4
    bytes.extend(std::iter::repeat_n(b'c', LONG)); // LONG+4..2*LONG+4
    bytes.push(b'\n'); // 2*LONG+4..2*LONG+5
    bytes.extend(std::iter::repeat_n(b'9', LONG)); // 2*LONG+5..3*LONG+5

    assert_lossless(&bytes);
    let tokens = spans_of(&bytes);
    assert_eq!(tokens[0], (TokenKind::Ident, 0, LONG as u32));
    assert_eq!(
        tokens.last().unwrap().1,
        (2 * LONG + 5) as u32,
        "the integer literal starts only after the 128 KiB of leading trivia"
    );
}

#[test]
fn token_stream_is_monotonic_and_non_overlapping() {
    let bytes = b"  fn f(a: i32) { \n return 1; } // tail\n";
    let tokens = assert_lossless(bytes);
    let mut cursor = 0u32;
    for token in &tokens {
        assert!(
            token.span.start >= cursor,
            "span {} starts before the previous span ended at {}",
            token.span.start,
            cursor
        );
        assert!(token.span.end >= token.span.start);
        cursor = token.span.end;
    }
}

// --------------------------------------------------------------------------
// 0.0.1.B: Unicode identifiers
// --------------------------------------------------------------------------

#[test]
fn underscore_forms_are_identifiers() {
    let tokens = assert_lossless(b"let _x = __;");
    assert_eq!(
        tokens.into_iter().map(|t| (t.kind, t.span.start, t.span.end)).collect::<Vec<_>>(),
        vec![
            (TokenKind::Keyword(Kw::Let), 0, 3),
            (TokenKind::Ident, 4, 6),
            (TokenKind::Punct(Punct::Eq), 7, 8),
            (TokenKind::Ident, 9, 11),
            (TokenKind::Punct(Punct::Semicolon), 11, 12),
        ],
        "`_` is an identifier start, never punctuation (identifier = (XID_Start | \"_\") ...)"
    );
}

#[test]
fn xid_start_accepts_non_ascii_identifiers() {
    let text = "let \u{65E5}\u{672C}\u{8A9E} = 1;".as_bytes();
    let tokens = assert_lossless(text);
    let name = &tokens[1];
    assert_eq!(name.kind, TokenKind::Ident);
    assert_eq!((name.span.start, name.span.end), (4, 13), "each CJK character is 3 bytes");
    assert_eq!(&text[4..13], "\u{65E5}\u{672C}\u{8A9E}".as_bytes());

    let tokens = assert_lossless("\u{03B1}\u{03B2}\u{03B3}".as_bytes());
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::Ident);
    assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, 6));
}

#[test]
fn xid_continue_only_code_points_cannot_start_an_identifier() {
    // U+0301 COMBINING ACUTE ACCENT (Mn) and U+00B7 MIDDLE DOT are
    // XID_Continue but not XID_Start.
    for lone in ["\u{0301}", "\u{00B7}"] {
        let mut bytes = lone.as_bytes().to_vec();
        bytes.push(b'x');
        let tokens = assert_lossless(&bytes);
        assert_eq!(tokens.len(), 2, "{:?} must not start an identifier", lone);
        assert_eq!(tokens[0].kind, TokenKind::Error);
        assert_eq!(
            (tokens[0].span.start, tokens[0].span.end),
            (0, lone.len() as u32),
            "the rejected code point stays spanned"
        );
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(
            (tokens[1].span.start, tokens[1].span.end),
            (lone.len() as u32, bytes.len() as u32)
        );
    }
}

#[test]
fn xid_continue_glues_across_ascii_and_non_ascii() {
    for src in ["a\u{00B7}b", "a\u{0301}", "\u{0430}\u{0431}\u{0432}"] {
        let bytes = src.as_bytes();
        let tokens = assert_lossless(bytes);
        assert_eq!(tokens.len(), 1, "{:?} is a single identifier", src);
        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, bytes.len() as u32));
    }
}

#[test]
fn keyword_prefixes_do_not_split_off_from_longer_identifiers() {
    for src in ["iffy", "letx", "returned", "crateful", "modulator"] {
        let tokens = assert_lossless(src.as_bytes());
        assert_eq!(tokens.len(), 1, "{:?} must lex as one identifier", src);
        assert_eq!(tokens[0].kind, TokenKind::Ident, "{:?} must not split", src);
    }
}

// --------------------------------------------------------------------------
// 0.0.1.B: the normative keyword table
// --------------------------------------------------------------------------

fn quoted_strings(s: &str) -> Vec<&str> {
    let bytes = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != b'"' {
                j += 1;
            }
            if j >= bytes.len() {
                break;
            }
            out.push(&s[start..j]);
            i = j + 1;
        } else {
            i += 1;
        }
    }
    out
}

fn normative_keywords() -> Vec<String> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/grammar/omni-edition1.ebnf");
    let ebnf = std::fs::read_to_string(path).expect("the spec grammar must be readable");
    let start = ebnf.find("keyword =").expect("the spec declares a keyword production");
    let rest = &ebnf[start..];
    let end = rest.find(';').expect("the keyword production is terminated");
    quoted_strings(&rest[..end]).iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn keyword_table_matches_the_normative_grammar() {
    let keywords = normative_keywords();
    let unique: std::collections::BTreeSet<&str> = keywords.iter().map(String::as_str).collect();
    assert_eq!(unique.len(), keywords.len(), "the keyword production must be duplicate-free");
    assert_eq!(keywords.len(), 91, "the normative keyword production declares 91 spellings");

    // Every declared keyword must lex as a keyword, never as an identifier.
    for kw in &keywords {
        let tokens = assert_lossless(kw.as_bytes());
        assert_eq!(tokens.len(), 1, "{:?} must lex as a single token", kw);
        assert!(
            matches!(tokens[0].kind, TokenKind::Keyword(_)),
            "{:?} must lex as a keyword, got {:?}",
            kw,
            tokens[0].kind
        );
    }

    // The scanner's table is the production plus the two identifier-shaped
    // terminals the grammar body needs (`mod`, `crate`) and nothing else.
    let scanner_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/scanner.rs");
    let scanner_src =
        std::fs::read_to_string(scanner_path).expect("scanner source must be readable");
    assert_eq!(
        scanner_src.matches("=> TokenKind::Keyword(").count(),
        keywords.len() + 2,
        "scanner keyword table must be production + mod + crate"
    );

    for extra in ["mod", "crate"] {
        let tokens = assert_lossless(extra.as_bytes());
        assert!(
            matches!(tokens[0].kind, TokenKind::Keyword(_)),
            "{:?} is a grammar-body terminal and must stay a keyword",
            extra
        );
    }
    for not_a_kw in ["parallel", "with", "module_", "mods"] {
        let tokens = assert_lossless(not_a_kw.as_bytes());
        assert_eq!(
            tokens[0].kind,
            TokenKind::Ident,
            "{:?} appears in neither the keyword production nor the grammar body",
            not_a_kw
        );
    }
}

#[test]
fn module_and_extern_crate_declarations_are_lexable() {
    // `module_decl = [ "pub" ] "mod" identifier ...`
    let tokens = assert_lossless(b"mod geometry { }");
    assert_eq!(tokens[0].kind, TokenKind::Keyword(Kw::Mod));
    assert_eq!(tokens[1].kind, TokenKind::Ident);
    assert_eq!((tokens[1].span.start, tokens[1].span.end), (4, 12));

    // `extern_crate_decl = "extern" "crate" identifier [ "as" identifier ] ";"`
    let tokens = assert_lossless(b"extern crate geometry as geo;");
    assert_eq!(tokens[0].kind, TokenKind::Keyword(Kw::Extern));
    assert_eq!(tokens[1].kind, TokenKind::Keyword(Kw::Crate));
    assert_eq!(tokens[2].kind, TokenKind::Ident);
}

// --------------------------------------------------------------------------
// 0.0.1.B: raw identifiers
// --------------------------------------------------------------------------

#[test]
fn raw_identifiers_escape_keywords() {
    // `raw_identifier = "r#" identifier`; LEX-0005 allows the underlying
    // name to equal a keyword.
    for kw in ["fn", "type", "let", "mod", "crate", "use"] {
        let src = format!("r#{}", kw);
        let tokens = assert_lossless(src.as_bytes());
        assert_eq!(tokens.len(), 1, "{:?} must lex as one token", src);
        assert_eq!(tokens[0].kind, TokenKind::Ident, "r#{} must not become a keyword", kw);
        assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, src.len() as u32));
    }

    let tokens = assert_lossless(b"let r#mod = 1;");
    assert_eq!(
        tokens.into_iter().map(|t| (t.kind, t.span.start, t.span.end)).collect::<Vec<_>>(),
        vec![
            (TokenKind::Keyword(Kw::Let), 0, 3),
            (TokenKind::Ident, 4, 9),
            (TokenKind::Punct(Punct::Eq), 10, 11),
            (TokenKind::Int, 12, 13),
            (TokenKind::Punct(Punct::Semicolon), 13, 14),
        ]
    );
}

#[test]
fn raw_identifier_tail_must_satisfy_the_identifier_rule() {
    let mut combining = b"r#".to_vec();
    combining.extend_from_slice("\u{0301}".as_bytes());
    let mut cases: Vec<&[u8]> = vec![b"r#", b"r#1", b"r#9x"];
    cases.push(combining.as_slice());

    for src in cases {
        let tokens = assert_lossless(src);
        assert_eq!(
            tokens[0].kind,
            TokenKind::Error,
            "{:?} has an invalid tail",
            String::from_utf8_lossy(src)
        );
        assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, 2), "the r# prefix is spanned");
    }

    // The offending tail character becomes its own token, not part of the error.
    let tokens = assert_lossless(b"r#1");
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[1].kind, TokenKind::Int);
    assert_eq!((tokens[1].span.start, tokens[1].span.end), (2, 3));

    let tokens = assert_lossless("r#\u{03B1}".as_bytes());
    assert_eq!(tokens.len(), 1, "a valid XID_Start tail is accepted");
    assert_eq!(tokens[0].kind, TokenKind::Ident);
    assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, 4));
}

#[test]
fn raw_string_takes_precedence_over_raw_identifier() {
    let tokens = assert_lossless(b"r#\"fn\"#");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenKind::RawString);
    assert_eq!((tokens[0].span.start, tokens[0].span.end), (0, 7));
}

#[test]
fn raw_identifier_is_only_recognised_directly_after_r() {
    let tokens = assert_lossless(b"xr#y");
    assert_eq!(
        tokens.into_iter().map(|t| (t.kind, t.span.start, t.span.end)).collect::<Vec<_>>(),
        vec![
            (TokenKind::Ident, 0, 2),
            (TokenKind::Punct(Punct::Hash), 2, 3),
            (TokenKind::Ident, 3, 4),
        ],
        "`#` may only start a raw identifier directly after an `r`"
    );
}

#[test]
fn identifier_trivia_and_separators_stay_byte_exact_after_unicode() {
    let text = "fn \u{51FD}\u{6570}(\u{53C2}: i32) -> i32 { return \u{503C}; }\n";
    let bytes = text.as_bytes();
    let tokens = assert_lossless(bytes);
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Ident));
    assert!(tokens.iter().any(|t| t.kind == TokenKind::Keyword(Kw::Fn)));
    assert_eq!(reconstruct(bytes), bytes);
}
