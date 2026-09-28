#![no_main]

use libfuzzer_sys::fuzz_target;
use omni_lex::{ErrorReason, Scanner, SecurityMode, TokenKind};
use omni_unicode::classify::prohibited_kind;

/// Reconstruct the byte stream from tokens and trivia, asserting it is exact.
fn round_trip(data: &[u8], mode: SecurityMode) {
    let mut scanner = Scanner::with_security_mode(data, 0, mode);
    let mut out: Vec<u8> = Vec::with_capacity(data.len() + 16);

    let mut emit = |out: &mut Vec<u8>, lo: usize, hi: usize| {
        assert!(lo <= hi, "span must be half-open");
        assert!(hi <= data.len(), "span must stay within the input");
        out.extend_from_slice(&data[lo..hi]);
    };

    while let Some(token) = scanner.next_token() {
        for trivia in &token.leading_trivia {
            emit(&mut out, trivia.span.start as usize, trivia.span.end as usize);
        }
        emit(&mut out, token.span.start as usize, token.span.end as usize);
        for trivia in &token.trailing_trivia {
            emit(&mut out, trivia.span.start as usize, trivia.span.end as usize);
        }

        match token.error_reason {
            // An error token must state why it failed; a non-error token must
            // not claim one. This is the invariant diagnostics depend on.
            Some(_) => assert_eq!(token.kind, TokenKind::Error, "an error token must say so"),
            None => assert_ne!(token.kind, TokenKind::Error, "an error token must say why"),
        }

        if let Some(ErrorReason::ProhibitedSource { kind }) = token.error_reason {
            // The reported span must be non-empty, inside the input, valid
            // UTF-8, and must actually cover a code point of the class reported.
            assert!(token.span.end > token.span.start, "a prohibited span must be non-empty");
            let text = std::str::from_utf8(&data[token.span.start as usize..token.span.end as usize])
                .expect("a prohibited span must be valid UTF-8");
            assert!(
                text.chars().any(|c| prohibited_kind(c) == Some(kind)),
                "a ProhibitedSource token must span a code point of the class it reports"
            );
        }

        // Outside a comment or a literal, a prohibited code point must never be
        // accepted as a valid token. The scanner reports those as errors, so a
        // *valid* token may legitimately cover a prohibited code point only
        // inside a literal, where SRC-0007 makes it data.
        if token.kind != TokenKind::Error {
            let text = std::str::from_utf8(&data[token.span.start as usize..token.span.end as usize])
                .unwrap_or("");
            if let Some(kind) = text.chars().find_map(prohibited_kind) {
                let inside_literal = matches!(
                    token.kind,
                    TokenKind::String
                        | TokenKind::RawString
                        | TokenKind::InterpolatedString
                        | TokenKind::Char
                        | TokenKind::Byte
                );
                assert!(
                    inside_literal,
                    "a valid {:?} token must not carry a prohibited code point ({kind}) \
                     outside a literal",
                    token.kind
                );
            }
        }
    }
    for trivia in scanner.take_eof_trivia() {
        emit(&mut out, trivia.span.start as usize, trivia.span.end as usize);
    }

    if data != out.as_slice() {
        panic!(
            "lossless reconstruction failed\noriginal:      {data:?}\nreconstructed: {out:?}"
        );
    }
}

fuzz_target!(|data: &[u8]| {
    // No UTF-8 gating: byte entry is the contract, so arbitrary bytes must be
    // scannable without panicking and must still reconstruct exactly.
    round_trip(data, SecurityMode::Strict);
    // Lenient mode must agree on the byte stream; it differs only in whether an
    // unannotated comment is reported.
    round_trip(data, SecurityMode::Lenient);
});
