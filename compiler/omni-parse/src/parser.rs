//! Lossless recursive-descent + Pratt parser for Omni.
//!
//! # CST contracts (0.0.2.2)
//!
//! These are the guarantees this parser provides to every downstream consumer.
//! They are established by tests, not by convention, and the adversarial suite
//! in `tests/cst_adversarial.rs` re-derives them independently.
//!
//! * **Exact source reconstruction.** For every input the parser accepts,
//!   `parse_source().syntax().text() == source`, including malformed input,
//!   recovery input, empty input, trivia-only input, BOM input, and input that
//!   is malformed through to EOF.
//! * **Exact tiling.** Source-bearing leaves tile the source in order with no
//!   gap and no overlap, so every byte is covered exactly once. String equality
//!   alone is a weaker property: a tree that drops one token and re-emits an
//!   identical-looking one elsewhere still compares equal.
//! * **Trivia preservation.** Every trivia region the lexer produces appears in
//!   the CST exactly once, with the same byte total. Trivia is never
//!   regenerated from source text when the lexer already supplies its span.
//! * **EOF handling.** The EOF token is appended exactly once, at the root, and
//!   only its carried trivia is emitted. It is never emitted as an ordinary
//!   source-bearing token.
//! * **Absent tokens.** A token the grammar required but the input did not
//!   supply is represented as [`SyntaxKind::MissingToken`]: zero-width, so it
//!   can neither fabricate nor duplicate source bytes, and distinct from
//!   [`SyntaxKind::ErrorNode`] so it cannot be confused with recovery.
//! * **Determinism.** The same source always yields the same tree shape, the
//!   same leaf kinds and spans, and the same diagnostics.
//! * **Spans are byte offsets.** Leaves land on UTF-8 character boundaries; no
//!   UTF-16 or Unicode-scalar indexing, no normalization, no line-ending
//!   rewriting. The lexer remains the source-span authority.
//! * **Malformed input stays diagnosed.** Losslessness is never achieved by
//!   accepting everything; malformed input must remain lossless *and* produce
//!   diagnostics.
//!
//! ## Scope
//!
//! This milestone establishes the CST substrate. It does **not** expand Edition 1
//! grammar coverage — `block_expr`, the remaining binary operators, complete
//! `struct`/`enum` bodies, `if`, `match`, loops, closures, arrays, tuples, and
//! indexing remain 0.0.2.3's work, and `parse_item_stub` is a deliberate
//! placeholder. Candidate 2 syntax is not implemented and not enabled.

use omni_lex::token::{Kw, Punct, Trivia};
use omni_lex::{Scanner, Span, Token, TokenKind};
use omni_syntax::{SyntaxKind, SyntaxNode};
use rowan::GreenNodeBuilder;

use crate::precedence::{matching_close, matching_open};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
enum Child {
    Node(Node),
    Token(usize),
    /// A zero-width placeholder standing in for an absent token.
    ///
    /// This is deliberately a distinct variant rather than an index. Pointing a
    /// `Token` at the EOF index (which is what `tokens.len() - 1` happens to be
    /// at end of input) silently re-emits the terminal token and, worse, re-emits
    /// the trivia it carries, duplicating source text in the tree.
    Missing,
}
#[derive(Debug, Clone)]
struct Node {
    kind: SyntaxKind,
    children: Vec<Child>,
}
impl Node {
    fn new(kind: SyntaxKind) -> Self {
        Self { kind, children: Vec::new() }
    }
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub green: rowan::GreenNode,
    pub diagnostics: Vec<Diagnostic>,
}
impl ParseResult {
    pub fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    pos: usize,
}
impl<'a> Parser<'a> {
    pub fn from_source(source: &'a str) -> Self {
        let mut scanner = Scanner::from_str(source, 0);
        let mut tokens: Vec<Token> = std::iter::from_fn(|| scanner.next_token()).collect();
        // The scanner parks bytes past the final token on `next_token()`'s
        // `None`; folding them into a terminal token is what keeps the emitted
        // green tree equal to the original source.
        let eof_end = source.len() as u32;
        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span { start: eof_end, end: eof_end, file_id: 0 },
            leading_trivia: scanner.take_eof_trivia(),
            trailing_trivia: Vec::new(),
            error_reason: None,
        });
        Self { source, tokens, diagnostics: Vec::new(), pos: 0 }
    }
    pub fn new(tokens: Vec<String>) -> Parser<'static> {
        let source = Box::leak(tokens.join(" ").into_boxed_str());
        Parser::from_source(source)
    }
    pub fn parse(&mut self) -> Result<(), String> {
        if self.source.is_empty() && self.tokens.is_empty() {
            return Ok(());
        }
        let result = self.parse_source();
        if result.is_ok() {
            Ok(())
        } else {
            Err(result.diagnostics.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join("; "))
        }
    }
    pub fn parse_source(&mut self) -> ParseResult {
        self.pos = 0;
        self.diagnostics.clear();
        let mut root = self.parse_source_file();
        // The EOF token is appended exactly once, here, and never anywhere else:
        // `parse_source_file` stops at EOF, and `expect_*` yields `Child::Missing`
        // rather than re-consuming the terminal token. Appending it more than
        // once would duplicate the trivia EOF carries.
        debug_assert!(
            !root
                .children
                .iter()
                .any(|c| matches!(c, Child::Token(i) if self.tokens[*i].kind == TokenKind::Eof)),
            "the EOF token must be appended exactly once"
        );
        if let Some(i) = self.tokens.iter().position(|t| t.kind == TokenKind::Eof) {
            root.children.push(Child::Token(i));
        }
        let mut b = GreenNodeBuilder::new();
        self.emit_node(&mut b, &root);
        let green = b.finish();
        ParseResult { green, diagnostics: self.diagnostics.clone() }
    }
    fn parse_source_file(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::SourceFile);
        while !self.eof() {
            if self.at_kw(Kw::Fn) {
                n.children.push(Child::Node(self.parse_fn()));
            } else if self.at_kw(Kw::Struct) || self.at_kw(Kw::Enum) {
                n.children.push(Child::Node(self.parse_item_stub()));
            } else {
                n.children.push(Child::Node(self.error_node("expected a top-level declaration")));
                let skipped = self.synchronize_top();
                if !skipped.is_empty() {
                    n.children.push(Child::Node(self.error_node_from(skipped)));
                }
            }
        }
        n
    }
    fn parse_item_stub(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ErrorNode);
        n.children.push(self.bump_child());
        if self.at_ident() {
            n.children.push(self.bump_child());
        }
        n
    }
    fn parse_fn(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::FnDef);
        n.children.push(self.expect_kw(Kw::Fn));
        if self.at_ident() {
            n.children
                .push(Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump_index())));
        } else {
            n.children.push(Child::Node(self.error_node("expected function name")));
        }
        n.children.push(Child::Node(self.parse_params()));
        if self.at_punct(Punct::Arrow) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type()));
        }
        n.children.push(Child::Node(self.parse_block()));
        n
    }
    fn parse_params(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ParamList);
        n.children.push(self.expect_punct(Punct::LParen));
        while !self.eof() && !self.at_punct(Punct::RParen) {
            let mut p = Node::new(SyntaxKind::Param);
            if self.at_ident() {
                p.children
                    .push(Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump_index())));
            } else {
                p.children.push(Child::Node(self.error_node("expected parameter name")));
                let skipped = self.recover_until(&[Punct::Comma, Punct::RParen]);
                if !skipped.is_empty() {
                    p.children.push(Child::Node(self.error_node_from(skipped)));
                }
            }
            if self.at_punct(Punct::Colon) {
                p.children.push(self.bump_child());
                p.children.push(Child::Node(self.parse_type()));
            } else {
                self.diagnostic("expected `:` after parameter name");
            }
            n.children.push(Child::Node(p));
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
            } else {
                break;
            }
        }
        n.children.push(self.expect_punct(Punct::RParen));
        n
    }
    fn parse_type(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Type);
        if self.at_ident()
            || matches!(
                self.current_kind(),
                Some(TokenKind::Keyword(
                    Kw::Never
                        | Kw::SelfKw
                        | Kw::Sized
                        | Kw::Bf16
                        | Kw::Bool
                        | Kw::Byte
                        | Kw::Char
                        | Kw::Dec128
                        | Kw::Dec32
                        | Kw::Dec64
                        | Kw::F128
                        | Kw::F16
                        | Kw::F32
                        | Kw::F64
                        | Kw::I128
                        | Kw::I16
                        | Kw::I32
                        | Kw::I64
                        | Kw::I8
                        | Kw::Isize
                        | Kw::Str
                        | Kw::U128
                        | Kw::U16
                        | Kw::U32
                        | Kw::U64
                        | Kw::U8
                        | Kw::Usize
                        | Kw::True
                        | Kw::False
                ))
            )
        {
            n.children.push(self.bump_child());
        } else {
            n.children.push(Child::Node(self.error_node("expected type name")));
        }
        n
    }
    fn parse_block(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Block);
        n.children.push(self.expect_punct(Punct::LBrace));
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            if self.at_kw(Kw::Let) {
                n.children.push(Child::Node(self.parse_let()));
            } else if self.at_kw(Kw::Return) {
                n.children.push(Child::Node(self.parse_return()));
            } else {
                n.children.push(Child::Node(self.parse_expr_stmt()));
            }
        }
        n.children.push(self.expect_punct(Punct::RBrace));
        n
    }
    fn parse_let(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::LetStmt);
        n.children.push(self.expect_kw(Kw::Let));
        if self.at_kw(Kw::Mut) {
            n.children.push(self.bump_child());
        }
        if self.at_ident() {
            n.children
                .push(Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump_index())));
        } else {
            n.children.push(Child::Node(self.error_node("expected binding name")));
        }
        if self.at_punct(Punct::Colon) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type()));
        }
        n.children.push(self.expect_punct(Punct::Eq));
        n.children.push(Child::Node(self.parse_expr_bp(0)));
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }
    fn parse_return(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ReturnExpr);
        n.children.push(self.expect_kw(Kw::Return));
        if !self.at_punct(Punct::Semicolon) && !self.at_punct(Punct::RBrace) {
            n.children.push(Child::Node(self.parse_expr_bp(0)));
        }
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }
    fn parse_expr_stmt(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ExprStmt);
        n.children.push(Child::Node(self.parse_expr_bp(0)));
        if self.at_punct(Punct::Semicolon) {
            n.children.push(self.bump_child());
        } else {
            self.diagnostic("expected `;` after expression");
        }
        n
    }
    fn parse_expr_bp(&mut self, min_bp: u8) -> Node {
        let mut lhs = self.parse_prefix();
        loop {
            if self.at_punct(Punct::LParen) && 7 >= min_bp {
                let mut call = Node::new(SyntaxKind::CallExpr);
                call.children.push(Child::Node(lhs));
                call.children.push(self.bump_child());
                while !self.eof() && !self.at_punct(Punct::RParen) {
                    call.children.push(Child::Node(self.parse_expr_bp(0)));
                    if self.at_punct(Punct::Comma) {
                        call.children.push(self.bump_child());
                    } else {
                        break;
                    }
                }
                call.children.push(self.expect_punct(Punct::RParen));
                lhs = call;
                continue;
            }
            let Some((op, left_bp, right_bp)) = self.infix() else { break };
            if left_bp < min_bp {
                break;
            }
            let mut bin = Node::new(SyntaxKind::BinaryExpr);
            bin.children.push(Child::Node(lhs));
            bin.children.push(self.bump_child());
            bin.children.push(Child::Node(self.parse_expr_bp(right_bp)));
            let _ = op;
            lhs = bin;
        }
        lhs
    }
    fn parse_prefix(&mut self) -> Node {
        if matches!(
            self.current_kind(),
            Some(TokenKind::Punct(Punct::Minus | Punct::Bang | Punct::Amp))
        ) {
            let mut n = Node::new(SyntaxKind::UnaryExpr);
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_expr_bp(6)));
            return n;
        }
        if self.at_punct(Punct::LParen) {
            let mut n = Node::new(SyntaxKind::UnaryExpr);
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_expr_bp(0)));
            n.children.push(self.expect_punct(Punct::RParen));
            return n;
        }
        match self.current_kind() {
            Some(TokenKind::Ident) => {
                Node::new(SyntaxKind::NameRef).with_token(self.bump_index())
            },
            Some(
                TokenKind::Int
                | TokenKind::Float
                | TokenKind::Char
                | TokenKind::Byte
                | TokenKind::String
                | TokenKind::RawString
                | TokenKind::InterpolatedString,
            )
            | Some(TokenKind::Keyword(Kw::True | Kw::False)) => {
                Node::new(SyntaxKind::LiteralExpr).with_token(self.bump_index())
            }
            _ => self.error_node("expected expression"),
        }
    }
    fn infix(&self) -> Option<(Punct, u8, u8)> {
        match self.current_kind() {
            Some(TokenKind::Punct(Punct::Eq)) => Some((Punct::Eq, 1, 1)),
            Some(TokenKind::Punct(Punct::Pipe)) => Some((Punct::Pipe, 2, 3)),
            Some(TokenKind::Punct(Punct::EqEq | Punct::NotEq)) => Some((Punct::EqEq, 3, 4)),
            Some(TokenKind::Punct(Punct::Lt | Punct::Le | Punct::Gt | Punct::Ge)) => {
                Some((Punct::Lt, 4, 5))
            }
            Some(TokenKind::Punct(Punct::Plus | Punct::Minus)) => Some((Punct::Plus, 5, 6)),
            Some(TokenKind::Punct(Punct::Star | Punct::Slash)) => Some((Punct::Star, 7, 8)),
            _ => None,
        }
    }
    fn emit_node(&self, b: &mut GreenNodeBuilder, n: &Node) {
        b.start_node(n.kind.into());
        for child in &n.children {
            match child {
                Child::Node(c) => self.emit_node(b, c),
                Child::Token(i) => self.emit_token(b, *i),
                // A missing token is zero-width and carries no trivia, so it can
                // never affect the reconstructed text.
                Child::Missing => {
                    b.token(SyntaxKind::MissingToken.into(), "");
                }
            }
        }
        b.finish_node();
    }
    fn emit_token(&self, b: &mut GreenNodeBuilder, index: usize) {
        let Some(t) = self.tokens.get(index) else {
            return;
        };
        for tr in &t.leading_trivia {
            self.emit_trivia(b, tr);
        }
        if t.kind == TokenKind::Eof {
            // Zero-width terminal: emit only the trivia it carries.
            for tr in &t.trailing_trivia {
                self.emit_trivia(b, tr);
            }
            return;
        }
        let (kind, text) = self.token_text(t);
        b.token(kind.into(), text);
        for tr in &t.trailing_trivia {
            self.emit_trivia(b, tr);
        }
    }
    fn emit_trivia(&self, b: &mut GreenNodeBuilder, tr: &Trivia) {
        let kind = match tr.kind {
            omni_lex::TriviaKind::Whitespace => SyntaxKind::Whitespace,
            omni_lex::TriviaKind::LineComment => SyntaxKind::LineComment,
            omni_lex::TriviaKind::BlockComment => SyntaxKind::BlockComment,
            omni_lex::TriviaKind::DocComment => SyntaxKind::DocComment,
        };
        b.token(kind.into(), &self.source[tr.span.start as usize..tr.span.end as usize]);
    }
    fn token_text(&self, t: &Token) -> (SyntaxKind, &str) {
        let kind = match t.kind {
            TokenKind::Ident => SyntaxKind::Ident,
            TokenKind::Int => SyntaxKind::Int,
            TokenKind::Float => SyntaxKind::Float,
            TokenKind::Char
            | TokenKind::Byte
            | TokenKind::String
            | TokenKind::RawString
            | TokenKind::InterpolatedString => SyntaxKind::LiteralExpr,
            TokenKind::Keyword(_) => SyntaxKind::Keyword,
            TokenKind::Punct(_) => SyntaxKind::Punct,
            TokenKind::Indent => SyntaxKind::Indent,
            TokenKind::Dedent => SyntaxKind::Dedent,
            TokenKind::Error => SyntaxKind::ErrorToken,
            TokenKind::Eof => SyntaxKind::ErrorToken,
        };
        (kind, &self.source[t.span.start as usize..t.span.end as usize])
    }
    fn with_span(&self, i: usize) -> Option<Span> {
        self.tokens.get(i).map(|t| t.span)
    }
    fn diagnostic(&mut self, msg: &str) {
        self.diagnostics.push(Diagnostic { message: msg.into(), span: self.with_span(self.pos) });
    }
    fn error_node(&mut self, msg: &str) -> Node {
        self.diagnostic(msg);
        if self.eof() {
            Node::new(SyntaxKind::ErrorNode)
        } else {
            Node::new(SyntaxKind::ErrorNode).with_token(self.bump_index())
        }
    }
    /// Skips tokens until a top-level `fn` keyword or end of input.
    ///
    /// The skipped tokens are returned so the caller can attach them to an
    /// `ErrorNode`. Advancing `pos` without preserving the tokens would drop
    /// them from the green tree and break lossless reconstruction, which is the
    /// CST's defining invariant.
    fn synchronize_top(&mut self) -> Vec<usize> {
        let mut skipped = Vec::new();
        while !self.eof() && !self.at_kw(Kw::Fn) {
            skipped.push(self.bump_index());
        }
        skipped
    }

    /// Skips tokens until one of `puncts` or end of input.
    ///
    /// As with [`Parser::synchronize_top`], the skipped tokens are returned so
    /// the caller can keep them in the tree.
    fn recover_until(&mut self, puncts: &[Punct]) -> Vec<usize> {
        let mut skipped = Vec::new();
        while !self.eof() && !puncts.iter().any(|p| self.at_punct(*p)) {
            skipped.push(self.bump_index());
        }
        skipped
    }

    /// Wraps `tokens` in a recovery `ErrorNode`.
    ///
    /// `GRAM-0007` requires recovery nodes to be tagged and non-translatable;
    /// `SyntaxKind::ErrorNode` is that tag, and a parse carrying one must not
    /// be treated as a valid tree.
    fn error_node_from(&self, tokens: Vec<usize>) -> Node {
        let mut n = Node::new(SyntaxKind::ErrorNode);
        n.children.extend(tokens.into_iter().map(Child::Token));
        n
    }
    /// Consumes the expected keyword, or records a diagnostic and yields a
    /// zero-width [`Child::Missing`].
    ///
    /// At end of input this must *not* fall back to `tokens.len() - 1`: that is
    /// the EOF token, and emitting it here would both duplicate the terminal
    /// token and duplicate the EOF trivia it carries.
    fn expect_kw(&mut self, kw: Kw) -> Child {
        if self.at_kw(kw) {
            self.bump_child()
        } else {
            self.diagnostic("expected keyword");
            self.missing()
        }
    }
    fn expect_punct(&mut self, p: Punct) -> Child {
        if self.at_punct(p) {
            self.bump_child()
        } else {
            self.diagnostic("expected punctuation");
            self.missing()
        }
    }
    /// The zero-width stand-in for an absent token.
    fn missing(&mut self) -> Child {
        Child::Missing
    }
    /// Return the current token without advancing. The parser cursor is
    /// intentionally bounded by the physical lexer-token vector.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Return a token kind at a relative cursor offset without indexing past
    /// EOF. Lookahead never mutates parser state.
    fn peek_kind(&self, offset: usize) -> Option<TokenKind> {
        self.tokens.get(self.pos.checked_add(offset)?).map(|t| t.kind)
    }

    /// Advance exactly one physical lexer token, but only when the caller has
    /// established that the cursor is not at EOF.
    ///
    /// This is the only raw index-producing consumption primitive. Callers that
    /// consume grammar-required punctuation should prefer bump_child, which
    /// deterministically returns Child::Missing at EOF.
    fn bump_index(&mut self) -> usize {
        debug_assert!(!self.eof(), "bump_index must never consume EOF");
        let i = self.pos;
        self.pos += 1;
        i
    }

    /// Consume one token or return the deterministic zero-width missing-token
    /// representation. This prevents EOF from ever being aliased as a missing
    /// source-bearing token.
    fn bump_child(&mut self) -> Child {
        if self.eof() {
            Child::Missing
        } else {
            Child::Token(self.bump_index())
        }
    }

    /// Consume an expected opening delimiter. Pair identity is delegated to
    /// the single delimiter-pair authority in precedence.rs.
    fn expect_open(&mut self, open: Punct) -> Child {
        self.expect_punct(open)
    }

    /// Consume the closing delimiter corresponding to open. A mismatch remains
    /// a normal missing-token recovery event and does not consume another token.
    fn expect_close(&mut self, open: Punct) -> Child {
        let Some(close) = matching_close(open) else {
            self.diagnostic("invalid opening delimiter");
            return Child::Missing;
        };
        self.expect_punct(close)
    }

    /// Inverse delimiter check used by later delimiter-aware recovery.
    fn is_matching_close(&self, close: Punct, open: Punct) -> bool {
        matching_open(close) == Some(open)
    }

    fn current_kind(&self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind)
    }
    fn at_kw(&self, kw: Kw) -> bool {
        self.current_kind() == Some(TokenKind::Keyword(kw))
    }
    fn at_punct(&self, p: Punct) -> bool {
        self.current_kind() == Some(TokenKind::Punct(p))
    }
    fn at_ident(&self) -> bool {
        self.current_kind() == Some(TokenKind::Ident)
    }
    fn eof(&self) -> bool {
        match self.tokens.get(self.pos) {
            Some(t) => t.kind == TokenKind::Eof,
            None => true,
        }
    }
}
impl Node {
    fn with_token(mut self, i: usize) -> Self {
        self.children.push(Child::Token(i));
        self
    }
}

/// Desugar pipeline expressions and field projections.
pub fn desugar_node(
    node: rowan::SyntaxNode<omni_syntax::OmniLanguage>,
) -> rowan::SyntaxNode<omni_syntax::OmniLanguage> {
    node
}

#[cfg(test)]
mod tests {
    use super::*;
    use omni_syntax::{SyntaxElement, SyntaxKind as K};

    #[test]
    fn generic_closer_infrastructure_preserves_token_trivia_boundaries() {
        let mut p = Parser::from_source("T /*before*/ >>= /*after*/ U");
        assert_eq!(p.peek_kind(0), Some(TokenKind::Ident));
        assert_eq!(p.peek_kind(1), Some(TokenKind::ShrEq));
        let token = p.peek().expect("shift-assignment token");
        assert_eq!(&p.source[token.span.start as usize..token.span.end as usize], ">>=");
        let parts = crate::precedence::split_generic_closer(Punct::ShrEq).expect("split");
        assert_eq!(parts[0].expect("first").byte_offset, 0);
        assert_eq!(parts[1].expect("second").byte_offset, 1);
        assert_eq!(parts[2].expect("third").byte_offset, 2);
        assert_eq!(token.leading_trivia.len(), 1);
        assert_eq!(token.trailing_trivia.len(), 1);
    }

    #[test]
    fn parser_lookahead_and_missing_token_are_state_safe() {
        let mut p = Parser::from_source("");
        assert_eq!(p.peek_kind(0), Some(TokenKind::Eof));
        assert_eq!(p.peek_kind(1), None);
        assert!(matches!(p.bump_child(), Child::Missing));
        assert_eq!(p.pos, 0, "missing consumption must not advance at EOF");
        assert_eq!(p.current_kind(), Some(TokenKind::Eof));
    }

    #[test]
    fn delimiter_helpers_are_pair_aware() {
        let mut p = Parser::from_source("(");
        assert!(matches!(p.expect_open(Punct::LParen), Child::Token(0)));
        assert!(matches!(p.expect_close(Punct::LParen), Child::Missing));
        assert_eq!(p.pos, 1);
        assert!(p.is_matching_close(Punct::RParen, Punct::LParen));
        assert!(!p.is_matching_close(Punct::RBrace, Punct::LParen));
    }

    /// Parse `src` and assert every CST contract at once.
    ///
    /// String equality alone is not sufficient: two different trees can
    /// concatenate to the same text (a token dropped here and an identical one
    /// re-emitted there). So this also checks the *tiling* invariant, which pins
    /// every leaf to its exact source interval and therefore catches both loss
    /// and duplication.
    fn assert_lossless(src: &str) {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        let syntax = r.syntax();

        // Exact source preservation.
        assert_eq!(syntax.text().to_string(), src, "reconstruction for {src:?}");

        // The leaves exactly tile the source: no gap (a lost token or lost
        // trivia), no overlap (a duplicated token), and spans stay aligned with
        // the lexer's byte offsets.
        let mut cursor: u32 = 0;
        for el in syntax.descendants_with_tokens() {
            if let SyntaxElement::Token(t) = el {
                let start: u32 = t.text_range().start().into();
                let end: u32 = t.text_range().end().into();
                assert_eq!(
                    start,
                    cursor,
                    "leaf {:?} starts at {start}, expected {cursor} (src={src:?})",
                    t.text()
                );
                assert_eq!(
                    &src[start as usize..end as usize],
                    t.text(),
                    "leaf text disagrees with source at {start}..{end} (src={src:?})"
                );
                cursor = end;
            }
        }
        assert_eq!(cursor as usize, src.len(), "leaves cover {cursor} of {}", src.len());

        // Determinism: the same source yields the same kind sequence.
        let shape = |s: &omni_syntax::SyntaxNode| {
            s.descendants_with_tokens().map(|e| e.kind().to_raw().0).collect::<Vec<_>>()
        };
        let first = shape(&syntax);
        let mut p2 = Parser::from_source(src);
        assert_eq!(first, shape(&p2.parse_source().syntax()), "nondeterministic for {src:?}");
    }

    /// Assert that `src` is lossless *and* still rejected.
    ///
    /// Losslessness must never be achieved by accepting everything, so every
    /// malformed case is checked on both axes.
    fn assert_malformed_is_lossless_and_diagnosed(src: &str) {
        assert_lossless(src);
        let mut p = Parser::from_source(src);
        assert!(!p.parse_source().is_ok(), "{src:?} must still be reported, not silently accepted");
    }

    #[test]
    fn lifetime_syntax_is_lossless_but_remains_diagnosed_until_contract_is_resolved() {
        let src = "fn f(x: &'a T) { return 1; }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(!r.is_ok(), "the current lexer Error token must not be silently reinterpreted");
        assert_eq!(r.syntax().text().to_string(), src);
        assert!(r.syntax().descendants_with_tokens().any(|e| {
            matches!(e, SyntaxElement::Token(t) if t.kind() == K::ErrorToken)
        }));
    }

    #[test]
    fn parses_nested_expression_and_call() {
        let mut p =
            Parser::from_source("fn main(a: i32) -> i32 { let x = add(a, 2) * 3; return x; }");
        let r = p.parse_source();
        assert!(r.is_ok(), "{:?}", r.diagnostics);
        assert_eq!(
            r.syntax().text().to_string(),
            "fn main(a: i32) -> i32 { let x = add(a, 2) * 3; return x; }"
        );
        assert!(r.syntax().descendants().any(|n| n.kind() == SyntaxKind::CallExpr));
        assert!(r.syntax().descendants().any(|n| n.kind() == SyntaxKind::BinaryExpr));
    }
    #[test]
    fn parses_character_and_string_literals_as_literal_expressions() {
        for src in [
            "fn f() { return 'x'; }",
            "fn f() { return b'x'; }",
            "fn f() { return \"x\"; }",
            "fn f() { return r\"x\"; }",
            "fn f() { return f\"hello ${name}\"; }",
        ] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(r.is_ok(), "{:?} for {:?}", r.diagnostics, src);
            assert!(r.syntax().descendants().any(|n| n.kind() == SyntaxKind::LiteralExpr));
            assert_eq!(r.syntax().text().to_string(), src);
        }
    }
    #[test]
    fn reports_and_recovers() {
        let mut p = Parser::from_source("fn { let = 1 return 2; }");
        let r = p.parse_source();
        assert!(!r.is_ok());
        assert!(!r.syntax().text().is_empty());
    }
    // ----------------------------------------------------------------------
    // The regression that motivated 0.0.2.2.
    // ----------------------------------------------------------------------

    #[test]
    fn trailing_trivia_after_an_unclosed_block_is_not_duplicated() {
        // `bump_or_dummy` used to return `tokens.len() - 1`, which is the EOF
        // token. Every unclosed construct therefore re-emitted EOF, and because
        // EOF carries the trailing trivia, `// c` was emitted a second time:
        // this source reconstructed as "fn f() {\n// c\n// c".
        assert_malformed_is_lossless_and_diagnosed("fn f() {\n// c");
    }

    #[test]
    fn eof_trivia_is_emitted_exactly_once_whatever_follows() {
        for src in [
            "fn f() {\n// c",
            "fn f() {\n/// doc",
            "fn f() {\n/* b */",
            "fn f() {\n   ",
            "fn f() {",
            "fn f() { return 1;",
            "fn f() { let x = 1;",
            "fn f() { return",
            "fn f() { let",
            "fn f() ->",
            "fn f(",
            "fn",
            "\u{FEFF}fn f() {\n// c",
            "fn f() {}\n// trailing\n",
            "fn f() {}\n// trailing",
            "fn f() {} \n\n// a\n// b\n",
        ] {
            assert_lossless(src);
        }
    }

    #[test]
    fn degenerate_sources_round_trip() {
        for src in [
            "",
            " ",
            "   ",
            "\n",
            "\n\n\n",
            "\t",
            "\t\t\t",
            "  \n\t\n  ",
            "// line\n",
            "/// doc\n",
            "//! crate doc\n",
            "/* block */",
            "/* unterminated",
            "\u{FEFF}",
            "\u{FEFF}// doc\n",
            "\u{FEFF}fn f() {}\n",
            "\u{0}\u{1}",
        ] {
            assert_lossless(src);
        }
    }

    // ----------------------------------------------------------------------
    // The missing-token contract.
    // ----------------------------------------------------------------------

    #[test]
    fn a_missing_token_is_zero_width_and_tagged() {
        // An absent `;` must be represented, not fabricated from nearby bytes.
        let mut p = Parser::from_source("fn f() { return 1 }");
        let r = p.parse_source();
        assert!(!r.is_ok());
        assert!(
            r.syntax()
                .descendants_with_tokens()
                .any(|e| { matches!(e, SyntaxElement::Token(t) if t.kind() == K::MissingToken) }),
            "an absent token must be tagged MissingToken"
        );
        assert_eq!(r.syntax().text().to_string(), "fn f() { return 1 }");
    }

    #[test]
    fn missing_tokens_are_zero_width() {
        for src in ["fn f() { return 1 }", "fn f() { let x = 1 }", "fn f() { return"] {
            let mut p = Parser::from_source(src);
            for el in p.parse_source().syntax().descendants_with_tokens() {
                if let SyntaxElement::Token(t) = el {
                    if t.kind() == K::MissingToken {
                        assert_eq!(
                            t.text_range().start(),
                            t.text_range().end(),
                            "MissingToken must be zero-width in {src:?}"
                        );
                        assert!(t.text().is_empty());
                    }
                }
            }
        }
    }

    #[test]
    fn recovery_is_still_tagged_as_an_error_node() {
        // GRAM-0007: recovery nodes are tagged and non-translatable. Introducing
        // `MissingToken` must not erode that.
        let mut p = Parser::from_source("fn main( { return 1; }");
        let r = p.parse_source();
        assert!(!r.is_ok());
        assert!(r.syntax().descendants().any(|n| n.kind() == K::ErrorNode));
    }

    // ----------------------------------------------------------------------
    // Randomized differential check.
    // ----------------------------------------------------------------------

    #[test]
    fn randomized_token_soup_always_tiles_the_source() {
        // A deterministic LCG keeps any failure reproducible from this file
        // alone, with no reliance on a stored seed.
        let atoms = [
            "fn", "main", "(", ")", "{", "}", "let", "return", "1", "x", "+", "*", ";", ",", ":",
            "->", "=", "@", "struct", "enum", "mut", "true", "1.5", "&&", "// c\n",
        ];
        let mut seed: u64 = 0x2545F491_4F6CDD1D;
        let mut next = move || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        for _ in 0..20_000 {
            let n = next() % 12;
            let mut s = String::new();
            for _ in 0..n {
                s.push_str(atoms[next() % atoms.len()]);
                if next() % 3 == 0 {
                    s.push(' ');
                }
            }
            assert_lossless(&s);
        }
    }

    #[test]
    fn valid_source_round_trips_and_is_accepted() {
        for src in [
            "fn f() { return 1; }",
            "fn main() { return 0; }",
            "fn main(a: i32) -> i32 { let x = add(a, 2) * 3; return x; }",
            "fn f() { let mut x = 1; return x; }",
            "fn f() {\n    return 0;\n}\n\n// tail comment\n",
            "\u{FEFF}fn f() {}\n",
            "fn f() { /* c */ return /* d */ 1; }",
            "fn f() { return r\"raw\"; }",
            "fn f() { return b'x'; }",
            "fn f() { return f\"hello ${name}\"; }",
            // `struct`/`enum` are consumed by `parse_item_stub`, a deliberate
            // placeholder until 0.0.2.3 adds the real productions. A bare keyword
            // is therefore *accepted* today. That is a coverage gap, not a
            // losslessness claim, so it is asserted as-is rather than quietly
            // promoted to "rejected".
            "struct",
            "struct S",
            "enum E",
        ] {
            assert_lossless(src);
            let mut p = Parser::from_source(src);
            assert!(p.parse_source().is_ok(), "{src:?} must parse cleanly");
        }
    }

    /// A lexical error inside otherwise well-formed syntax must survive into the
    /// CST as an error token, never be normalized into ordinary punctuation.
    #[test]
    fn lexical_errors_remain_lexical_errors() {
        // An unterminated block comment is a lexical error: the lexer emits an
        // `ErrorToken`, and the parser must carry that through rather than
        // normalizing it into ordinary trivia or punctuation.
        for src in ["fn f() { return 1; } /* unterminated", "/* unterminated"] {
            assert_lossless(src);
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(
                r.syntax()
                    .descendants_with_tokens()
                    .any(|e| matches!(e, SyntaxElement::Token(t) if t.kind() == K::ErrorToken)),
                "a lexical error must remain an ErrorToken in the CST for {src:?}"
            );
        }
    }

    #[test]
    fn adversarial_malformed_input_is_lossless_and_diagnosed() {
        for src in [
            // Malformed top-level declarations.
            "let stray = 1",
            "@@@ fn f() {}",
            "@@@ fn f() {} @@@",
            "}}}}",
            "))",
            "struct S }",
            "enum E }",
            "fn f() {} garbage fn g() {} garbage",
            "@@@ ### $$$ %%% ^^^ &&& ***",
            // Malformed function declarations.
            "fn",
            "fn ",
            "fn f",
            "fn { }",
            "fn f()",
            "fn f() {",
            "fn f() ->",
            "fn f() -> i32",
            // Malformed parameter lists.
            "fn f(",
            "fn f(,) {}",
            "fn f(a,) {}",
            "fn f(,) -> i32 {}",
            "fn f(a: ) {}",
            "fn f(a i32) {}",
            "fn f(a: i32, , b: i32) {}",
            // Malformed bodies and statements.
            "fn f() { let = 1; }",
            "fn f() { let x = ; }",
            "fn f() { let x 1; }",
            "fn f() { let = ; }",
            "fn f() { let mut = 1; }",
            "fn f() { return 1 }",
            "fn f() { 1 2 3 }",
            "fn f() { + * ! }",
            "fn f() { f(1 2 3); }",
            "fn f() { f(,,,); }",
            "fn f() { f(1,,2); }",
            // Missing / extra / nested delimiters.
            "fn f() {{}",
            "fn f() {}}",
            "fn f() {} }",
            "fn f() { ((1); }",
            "fn f() { (((( }",
            "fn f() { 1 + * 2; }",
            "fn f() { @@@ }",
            // Recovery next to valid syntax, in both orders.
            "fn f() { return 1; } @@@ fn g() { return 2; }",
            "@@@ fn g() { return 2; } @@@ fn h() { return 3; }",
            // Long skipped runs.
            "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ fn g() {}",
            "fn f() { @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ }",
            // Comments inside recovery regions.
            "fn f() { @@@ /* inner */ @@@ }",
            // Valid syntax followed by malformed syntax.
            "fn f() { return 1; } fn g( { return",
        ] {
            assert_malformed_is_lossless_and_diagnosed(src);
        }
    }

    /// A trailing comment after otherwise-valid source is lossless and is *not*
    /// an error: comments are trivia, so they cannot invalidate a program.
    #[test]
    fn trailing_comments_are_trivia_not_errors() {
        for src in [
            "fn f() { return 1; } // tail",
            "fn f() { return 1; } /// tail\n",
            "fn f() { return 1; } /* tail */",
            "fn f() { return 1; }\n// tail\n// more\n",
        ] {
            assert_lossless(src);
            let mut p = Parser::from_source(src);
            assert!(p.parse_source().is_ok(), "{src:?} must parse cleanly");
        }
    }

    #[test]
    fn green_tree_text_is_the_original_source() {
        for src in [
            "fn main() {\n    return 0;\n}\n\n// tail comment\n",
            "\u{FEFF}fn f() {}\n",
            "// only a comment\n",
            "",
            "   \n\t\n",
            "fn f() { return 1; }",
        ] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert_eq!(r.syntax().text().to_string(), src, "lossless tree for {:?}", src);
        }
    }
}
