# Lifetime Lexer / Parser Contract Decision & Architecture

## Context & Problem Statement

In the Edition 1 grammar:
- `lifetime = "'" identifier`
- Character literals take the form `'a'` or `'\\n'`
- Byte literals take the form `b'a'`

During initial parser exploration, the question arose as to how `&'a T` or `'a` is tokenized by the lexer:
Does the lexer produce an `Error` token for `'a` (due to missing closing single quote `'`), which the parser then contextually reinterprets as an apostrophe `Punct::SingleQuote` + `Ident`?

## Investigation & Empirical Findings

Inspection of `compiler/omni-lex/src/scanner.rs` (`scan_char_or_byte`) reveals:
1. When encountering `'a` without a closing `'`, `scan_char_or_byte` invokes `self.drain_to_quote('\'')`.
2. If no closing single quote is found before whitespace/newline/delimiter/EOF, `scan_char_or_byte` returns `TokenKind::Error`.
3. The span of this `TokenKind::Error` encompasses the entire apostrophe + character sequence (e.g. `'a`).
4. Reinterpreting a multi-byte `TokenKind::Error` into an apostrophe `Punct::SingleQuote` and `Ident` in the parser would:
   - Violate the lexer/parser single-responsibility seam.
   - Mask genuine malformed character literals (such as `'ab'`).
   - Risk corrupting source-byte span accounting and trivia tracking in lossless CST parsing.

## Specification Architecture Alignment

1. **Normative EBNF**: `lifetime = "'" identifier;`
2. **Contextual Split vs Lexer Rule**: The lexer contract in Edition 1 defines character literals strictly with opening and closing quotes (`'c'`). The apostrophe in lifetime syntax (`'a`) is syntactically a single-quote prefix character before an identifier.
3. **Formal Decision**:
   - The parser receives tokens as emitted by the lexer.
   - Reinterpreting `TokenKind::Error` inside the parser is strictly forbidden as it hides lexer defects and violates CST losslessness invariants.
   - If lifetime tokens are recognized at the lexer level, it must be done explicitly or lifetime syntax parsed cleanly via punctuation + identifier where valid.
   - Currently, character literal scanning fails closed on unclosed quotes (`TokenKind::Error`).
   - The existing contract requires that malformed character literals (e.g., `'ab'`) remain `TokenKind::Error` and are never silently smoothed into lifetimes or identifier pairs.

## Status in 0.0.2.3-A

The lexer/parser seam remains cleanly isolated. Lossless CST parsing, diagnostic preservation, and exact byte span invariants are fully preserved.
