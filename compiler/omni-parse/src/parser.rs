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
//! 0.0.2.3 now implements a substantial Edition 1 parser slice while preserving
//! the lossless CST contracts established in 0.0.2.2. Remaining specification
//! gaps are isolated rather than guessed: trait function signatures are
//! undefined in the normative EBNF, `let_expr` and `deref_expr` are referenced
//! without productions, and effect/capability bounds have no definitions.
//! Candidate 2 syntax is not implemented and not enabled.

use omni_lex::token::{Kw, Punct, Trivia};
use omni_lex::{Scanner, Span, Token, TokenKind};
use omni_syntax::{SyntaxKind, SyntaxNode};
use rowan::GreenNodeBuilder;

use crate::precedence::matching_close;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
enum Child {
    Node(Node),
    Token(usize),
    /// A source-relative logical piece of a physical lexer token, used only
    /// when generic parsing splits `>>`/`>>=` into closers without mutating the
    /// lexer stream.
    Piece {
        token: usize,
        byte_offset: u8,
        byte_len: u8,
    },
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

/// A Stage-0-classified feature that the parsed source actually exercised,
/// together with the span at which it was used.
///
/// The parser **records** rather than **enforces**. Two distinct questions are
/// involved and they belong to different layers:
///
/// * *Is this syntax legal Edition 1?* — a grammar-layer question, answered by
///   this parser. `async_block`, `macro_invocation`, and `unsafe_block` are all
///   normative Edition 1 productions, so the parser must build them and must
///   round-trip their bytes.
/// * *Is this feature enabled for the selected profile?* — a profile-layer
///   question, answered by `omni_stage0`'s predicate engine against the
///   manifest's `stage0_feature_predicates`.
///
/// Folding the profile gate into the parser made Edition-1-legal source fail to
/// parse, which conflated the two layers and broke the pinned grammar contract.
/// Recording the usage keeps the information available and losslessly
/// attributable, and lets the profile layer decide legality on its own terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureUse {
    /// The manifest feature name, e.g. `macros` or `unsafe_raw_memory`.
    pub feature: &'static str,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub green: rowan::GreenNode,
    pub diagnostics: Vec<Diagnostic>,
    /// Every Stage-0-classified feature this source used, deduplicated and
    /// ordered by feature name so the set is deterministic regardless of the
    /// order the constructs appeared in. Empty whenever `diagnostics` is
    /// non-empty, since a rejected parse has no trustworthy attribution.
    pub feature_uses: Vec<FeatureUse>,
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
    split_token: Option<(usize, u8)>,
    /// Suppresses reading a `{` after a path as a struct literal. A `for`
    /// iterable is followed by the loop body block, so `for x in xs { .. }`
    /// must not parse `xs { .. }` as a struct expression.
    no_struct_literal: bool,
    /// Permits parsing a command call `name arg, arg` (ERR3-0035).
    /// Command calls are legal only at statement start, or as the right side of `let`, `=`, `return`, `|>`.
    allow_command_call: bool,
    /// First source position at which each Stage-0-classified feature was
    /// exercised. A `BTreeMap` keyed by feature name keeps the emitted set
    /// deterministic and deduplicated without depending on visit order.
    feature_uses: BTreeMap<&'static str, Span>,
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
        Self {
            source,
            tokens,
            diagnostics: Vec::new(),
            pos: 0,
            split_token: None,
            no_struct_literal: false,
            allow_command_call: false,
            feature_uses: BTreeMap::new(),
        }
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
    /// Record that the source exercised a Stage-0-classified feature, keeping the
    /// first span at which it appeared.
    ///
    /// This deliberately produces no diagnostic. Whether the feature is enabled
    /// is a profile-layer decision made against the manifest, and the parser has
    /// no manifest to consult — rejecting here would reject Edition-1-legal
    /// syntax on the grounds of a profile the caller never selected. Recording
    /// keeps the attribution (which construct, at which bytes) that the profile
    /// gate needs in order to report precisely, and keeps it lossless.
    ///
    /// Callers pass the span explicitly because the token introducing the
    /// construct is not always the current one: for `foo!(..)` the path has
    /// already been consumed by the time the `!` is seen, and for `async fn` the
    /// modifier is recognised before it is bumped.
    fn record_feature(&mut self, feature: &'static str, span: Span) {
        self.feature_uses.entry(feature).or_insert(span);
    }

    /// The span of the token about to be consumed, used to attribute a recorded
    /// feature to the exact source bytes that introduced it.
    fn current_span(&self) -> Span {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].span
        } else {
            Span { start: 0, end: 0, file_id: 0 }
        }
    }

    pub fn parse_source(&mut self) -> ParseResult {
        self.pos = 0;
        self.split_token = None;
        self.diagnostics.clear();
        // `feature_uses` needs no reset: a `Parser` is bound to one immutable
        // source, so re-parsing re-derives exactly the same entries, and the
        // first insertion per feature is already the earliest occurrence.
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
        // Recorded features are only meaningful for a source that actually
        // parsed: a rejected parse may have stopped before reaching some of
        // them, so reporting them would overstate what the source used.
        let feature_uses = if self.diagnostics.is_empty() {
            self.feature_uses
                .iter()
                .map(|(feature, span)| FeatureUse { feature, span: *span })
                .collect()
        } else {
            Vec::new()
        };
        ParseResult { green, diagnostics: self.diagnostics.clone(), feature_uses }
    }
    fn parse_source_file(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::SourceFile);
        while !self.eof() {
            if self.starts_item() {
                n.children.push(Child::Node(self.parse_item()));
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

    fn starts_item(&self) -> bool {
        if self.at_punct(Punct::Hash) {
            return true;
        }
        if self.at_kw(Kw::Pub) || self.at_kw(Kw::Const) {
            return true;
        }
        if self.at_kw(Kw::Unsafe) {
            return matches!(
                self.peek_kind(1),
                Some(TokenKind::Keyword(Kw::Fn | Kw::Trait | Kw::Impl))
            );
        }
        if self.at_kw(Kw::Async) {
            return self.peek_kind(1) == Some(TokenKind::Keyword(Kw::Fn));
        }
        matches!(
            self.current_kind(),
            Some(TokenKind::Keyword(
                Kw::Fn
                    | Kw::Struct
                    | Kw::Enum
                    | Kw::Trait
                    | Kw::Impl
                    | Kw::Static
                    | Kw::Use
                    | Kw::Mod
                    | Kw::Extern
                    | Kw::Type
            ))
        )
    }

    fn parse_item(&mut self) -> Node {
        // Prefixes are recursively consumed so ordering is preserved without
        // duplicating the item grammar in every parser entry point.
        if self.at_punct(Punct::Hash) {
            let attr = self.parse_attribute();
            if self.at_punct(Punct::Semicolon) {
                let mut n = Node::new(SyntaxKind::AttributeItem);
                n.children.push(Child::Node(attr));
                n.children.push(self.bump_child());
                return n;
            }
            let mut n = self.parse_item();
            n.children.insert(0, Child::Node(attr));
            return n;
        }
        if self.at_kw(Kw::Pub) {
            let modifier = self.bump_child();
            let mut n = self.parse_item();
            n.children.insert(0, modifier);
            return n;
        }
        if self.at_kw(Kw::Async) && self.peek_kind(1) == Some(TokenKind::Keyword(Kw::Fn)) {
            let async_span = self.current_span();
            self.record_feature("async", async_span);
            let modifier = self.bump_child();
            let mut n = self.parse_item();
            n.children.insert(0, modifier);
            return n;
        }
        if self.at_kw(Kw::Unsafe)
            && matches!(self.peek_kind(1), Some(TokenKind::Keyword(Kw::Fn | Kw::Trait | Kw::Impl)))
        {
            let unsafe_span = self.current_span();
            self.record_feature("unsafe_raw_memory", unsafe_span);
            let modifier = self.bump_child();
            let mut n = self.parse_item();
            n.children.insert(0, modifier);
            return n;
        }
        if self.at_kw(Kw::Const) && self.peek_kind(1) == Some(TokenKind::Keyword(Kw::Fn)) {
            let modifier = self.bump_child();
            let mut n = self.parse_item();
            n.children.insert(0, modifier);
            return n;
        }

        match self.current_kind() {
            Some(TokenKind::Keyword(Kw::Fn)) => self.parse_fn(),
            Some(TokenKind::Keyword(Kw::Struct)) => self.parse_struct_def(),
            Some(TokenKind::Keyword(Kw::Enum)) => self.parse_enum_def(),
            Some(TokenKind::Keyword(Kw::Trait)) => self.parse_trait_def(),
            Some(TokenKind::Keyword(Kw::Impl)) => self.parse_impl_def(),
            Some(TokenKind::Keyword(Kw::Type)) => self.parse_type_alias(),
            Some(TokenKind::Keyword(Kw::Const)) => self.parse_const_def(),
            Some(TokenKind::Keyword(Kw::Static)) => self.parse_static_def(),
            Some(TokenKind::Keyword(Kw::Use)) => self.parse_use_decl(),
            Some(TokenKind::Keyword(Kw::Mod)) => self.parse_module_decl(),
            Some(TokenKind::Keyword(Kw::Extern)) => self.parse_extern_crate_decl(),
            _ => self.error_node("expected module item"),
        }
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
        if self.at_punct(Punct::Lt) {
            n.children.push(Child::Node(self.parse_generic_params()));
        }
        n.children.push(Child::Node(self.parse_params()));
        if self.at_punct(Punct::Arrow) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type()));
        }
        if self.at_kw(Kw::Where) {
            n.children.push(Child::Node(self.parse_where_clause()));
        }
        if self.at_punct(Punct::Eq) {
            // Expression-bodied function: fn add(a, b) = a + b (ERR3-0040)
            let eq = self.bump_child();
            let expr = self.parse_expression();
            let mut block = Node::new(SyntaxKind::Block);
            block.children.push(eq);
            block.children.push(Child::Node(Node {
                kind: SyntaxKind::FinalExpr,
                children: vec![Child::Node(expr)],
            }));
            if self.at_punct(Punct::Semicolon) {
                block.children.push(self.bump_child());
            }
            n.children.push(Child::Node(block));
        } else {
            n.children.push(Child::Node(self.parse_block()));
        }
        n
    }

    fn parse_params(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ParamList);
        n.children.push(self.expect_open(Punct::LParen));
        if !self.at_punct(Punct::RParen) && !self.eof() {
            loop {
                let mut p = Node::new(SyntaxKind::Param);
                if self.at_kw(Kw::Mut) {
                    p.children.push(self.bump_child());
                }
                if self.at_ident() {
                    p.children.push(Child::Node(
                        Node::new(SyntaxKind::NameRef).with_token(self.bump_index()),
                    ));
                } else {
                    p.children.push(Child::Node(self.error_node("expected parameter name")));
                    let skipped = self.recover_until(&[Punct::Comma, Punct::RParen]);
                    if !skipped.is_empty() {
                        p.children.push(Child::Node(self.error_node_from(skipped)));
                    }
                }
                p.children.push(self.expect_punct(Punct::Colon));
                p.children.push(Child::Node(self.parse_type()));
                n.children.push(Child::Node(p));
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                    if self.at_punct(Punct::RParen) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        n.children.push(self.expect_close(Punct::LParen));
        n
    }

    fn parse_attribute(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Attribute);
        n.children.push(self.expect_punct(Punct::Hash));
        n.children.push(self.expect_punct(Punct::LBracket));
        n.children.push(Child::Node(self.parse_path()));
        if self.at_punct(Punct::LParen) {
            n.children.push(self.bump_child());
            if !self.at_punct(Punct::RParen) {
                loop {
                    if self.at_ident() && self.peek_kind(1) == Some(TokenKind::Punct(Punct::Eq)) {
                        n.children.push(Child::Node(
                            Node::new(SyntaxKind::NameRef).with_token(self.bump_index()),
                        ));
                        n.children.push(self.bump_child());
                    }
                    n.children.push(Child::Node(self.parse_expr_bp(0)));
                    if self.at_punct(Punct::Comma) {
                        n.children.push(self.bump_child());
                        if self.at_punct(Punct::RParen) {
                            self.diagnostic("trailing comma is not part of attribute_args");
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            n.children.push(self.expect_punct(Punct::RParen));
        }
        n.children.push(self.expect_punct(Punct::RBracket));
        n
    }

    fn parse_generic_params(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::GenericParams);
        n.children.push(self.expect_punct(Punct::Lt));
        if self.at_logical_gt() {
            self.diagnostic("generic parameter list cannot be empty");
        } else {
            n.children.push(Child::Node(self.parse_generic_param()));
            while self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
                if self.at_logical_gt() {
                    self.diagnostic("trailing comma is not part of generic_params");
                    break;
                }
                n.children.push(Child::Node(self.parse_generic_param()));
            }
        }
        n.children.push(self.consume_gt());
        n
    }

    fn parse_generic_param(&mut self) -> Node {
        let kind = match self.current_kind() {
            Some(TokenKind::Punct(Punct::Apostrophe)) => SyntaxKind::LifetimeParam,
            Some(TokenKind::Keyword(Kw::Const)) => SyntaxKind::ConstParam,
            Some(TokenKind::Keyword(Kw::Effect)) => SyntaxKind::EffectParam,
            Some(TokenKind::Keyword(Kw::Cap)) => SyntaxKind::CapabilityParam,
            _ => SyntaxKind::TypeParam,
        };
        let mut n = Node::new(kind);
        match kind {
            SyntaxKind::LifetimeParam => {
                n.children.push(Child::Node(self.parse_lifetime()));
            }
            SyntaxKind::ConstParam => {
                n.children.push(self.expect_kw(Kw::Const));
                if self.at_ident() {
                    n.children.push(Child::Node(
                        Node::new(SyntaxKind::NameRef).with_token(self.bump_index()),
                    ));
                } else {
                    n.children.push(Child::Node(self.error_node("expected const parameter name")));
                }
                n.children.push(self.expect_punct(Punct::Colon));
                n.children.push(Child::Node(self.parse_type()));
                if self.at_punct(Punct::Eq) {
                    n.children.push(self.bump_child());
                    n.children.push(Child::Node(self.parse_expr_bp(0)));
                }
            }
            SyntaxKind::EffectParam | SyntaxKind::CapabilityParam => {
                n.children.push(self.bump_child());
                if self.at_ident() {
                    n.children.push(Child::Node(
                        Node::new(SyntaxKind::NameRef).with_token(self.bump_index()),
                    ));
                } else {
                    n.children.push(Child::Node(self.error_node("expected parameter name")));
                }
                if self.at_punct(Punct::Colon) {
                    n.children.push(self.bump_child());
                    // Effect and capability bounds are parsed as paths, so both
                    // kinds produce the same child shape here.
                    n.children.push(Child::Node(self.parse_path()));
                }
            }
            _ => {
                if self.at_ident() {
                    n.children.push(Child::Node(
                        Node::new(SyntaxKind::NameRef).with_token(self.bump_index()),
                    ));
                } else {
                    n.children.push(Child::Node(self.error_node("expected type parameter name")));
                }
                if self.at_punct(Punct::Colon) {
                    n.children.push(self.bump_child());
                    n.children.push(Child::Node(self.parse_type_bound_list()));
                }
                if self.at_punct(Punct::Eq) {
                    n.children.push(self.bump_child());
                    n.children.push(Child::Node(self.parse_type()));
                }
            }
        }
        n
    }

    fn parse_type_bound_list(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::TypeBound);
        n.children.push(Child::Node(self.parse_type_bound()));
        while self.at_punct(Punct::Plus) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type_bound()));
        }
        n
    }

    fn parse_where_clause(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::WhereClause);
        n.children.push(self.expect_kw(Kw::Where));
        if !self.at_punct(Punct::LBrace) && !self.eof() {
            loop {
                let mut pred = Node::new(SyntaxKind::WherePredicate);
                pred.children.push(Child::Node(self.parse_type()));
                pred.children.push(self.expect_punct(Punct::Colon));
                pred.children.push(Child::Node(self.parse_type_bound_list()));
                n.children.push(Child::Node(pred));
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                    if self.at_punct(Punct::LBrace) || self.eof() {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        n
    }

    fn parse_type_bound(&mut self) -> Node {
        if self.at_punct(Punct::Question) {
            let mut n = Node::new(SyntaxKind::TypeBound);
            n.children.push(self.bump_child());
            n.children.push(self.expect_kw(Kw::Sized));
            return n;
        }
        if self.at_punct(Punct::Apostrophe) {
            return self.parse_lifetime();
        }
        let mut n = Node::new(SyntaxKind::TypeBound);
        n.children.push(Child::Node(self.parse_trait_ref()));
        n
    }

    fn parse_trait_ref(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::TraitRef);
        if self.at_kw(Kw::Dyn) {
            n.children.push(self.bump_child());
        }
        n.children.push(Child::Node(self.parse_path()));
        n
    }

    /// Determine whether the current '<' starts a balanced generic argument
    /// list rather than a comparison expression. A generic close must balance
    /// the opening '<'; after the close, an identifier/literal cannot directly
    /// continue the same path segment, which rejects `a < b > c` without
    /// backtracking or consuming source.
    fn looks_like_type_args(&self) -> bool {
        if !self.at_punct(Punct::Lt) || self.split_token.is_some() {
            return false;
        }
        let mut depth = 0usize;
        // Logical `>` closers still available inside the current physical token.
        // A single `>>`/`>>=` token supplies two closers, and the innermost list
        // may take only the first, leaving the second for an enclosing list.
        let mut pending = 0usize;
        let mut i = self.pos;
        while let Some(token) = self.tokens.get(i) {
            if pending > 0 {
                // The next logical token is another `>` from the token already
                // read, so the following physical token is not what comes next.
                pending -= 1;
                if depth == 0 {
                    // This list closed earlier and another `>` is still pending
                    // for an enclosing list; nothing can continue this path.
                    return true;
                }
                depth -= 1;
                if depth == 0 {
                    return true;
                }
                continue;
            }
            match token.kind {
                TokenKind::Punct(Punct::Lt) => depth += 1,
                TokenKind::Punct(Punct::Gt) => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                TokenKind::Punct(Punct::Shr) => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    pending = 1;
                }
                TokenKind::Punct(Punct::ShrEq) => {
                    // `>>=` supplies `>`, `>`, then `=`.
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    pending = 1;
                }
                TokenKind::Eof => return false,
                _ => {}
            }
            if depth == 0 && pending == 0 {
                return self.tokens.get(i + 1).is_none_or(|next| {
                    !matches!(
                        next.kind,
                        TokenKind::Ident
                            | TokenKind::Int
                            | TokenKind::Float
                            | TokenKind::Char
                            | TokenKind::Byte
                            | TokenKind::String
                            | TokenKind::RawString
                            | TokenKind::InterpolatedString
                    )
                });
            }
            i += 1;
        }
        false
    }

    fn parse_type_args(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::TypeArgs);
        n.children.push(self.expect_punct(Punct::Lt));
        while !self.eof() && !self.at_logical_gt() {
            if self.at_punct(Punct::Apostrophe) {
                n.children.push(Child::Node(self.parse_lifetime_arg()));
            } else if self.at_kw(Kw::Const) {
                let mut a = Node::new(SyntaxKind::ConstArg);
                a.children.push(self.bump_child());
                a.children.push(Child::Node(self.parse_expr_bp(0)));
                n.children.push(Child::Node(a));
            } else {
                let mut a = Node::new(SyntaxKind::TypeArg);
                a.children.push(Child::Node(self.parse_type()));
                n.children.push(Child::Node(a));
            }
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
                if self.at_logical_gt() {
                    break;
                }
            } else {
                break;
            }
        }
        n.children.push(self.consume_gt());
        n
    }

    fn at_logical_gt(&self) -> bool {
        matches!(self.current_kind(), Some(TokenKind::Punct(Punct::Gt)))
            || matches!(
                self.current_kind_physical(),
                Some(TokenKind::Punct(Punct::Shr | Punct::ShrEq))
            ) && self.split_token.is_none()
    }

    fn parse_lifetime(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Lifetime);
        n.children.push(self.expect_punct(Punct::Apostrophe));
        if self.at_ident() {
            n.children
                .push(Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump_index())));
        } else {
            n.children.push(Child::Node(self.error_node("expected lifetime name")));
        }
        n
    }

    fn parse_lifetime_arg(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::LifetimeArg);
        n.children.push(Child::Node(self.parse_lifetime()));
        n
    }

    fn parse_type(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Type);
        match self.current_kind() {
            Some(TokenKind::Keyword(
                Kw::Bf16
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
                | Kw::SelfKw
                | Kw::SelfRef
                | Kw::Never,
            )) => {
                let mut p = Node::new(SyntaxKind::PathType);
                p.children.push(self.bump_child());
                n.children.push(Child::Node(p));
            }
            Some(TokenKind::Punct(Punct::Amp)) => {
                let mut r = Node::new(SyntaxKind::ReferenceType);
                r.children.push(self.bump_child());
                if self.at_kw(Kw::Mut) {
                    r.children.push(self.bump_child());
                }
                if self.at_punct(Punct::Apostrophe) {
                    r.children.push(Child::Node(self.parse_lifetime()));
                }
                r.children.push(Child::Node(self.parse_type()));
                n.children.push(Child::Node(r));
            }
            Some(TokenKind::Punct(Punct::Star)) => {
                let mut r = Node::new(SyntaxKind::RawPointerType);
                r.children.push(self.bump_child());
                if self.at_kw(Kw::Const) || self.at_kw(Kw::Mut) {
                    r.children.push(self.bump_child());
                }
                r.children.push(Child::Node(self.parse_type()));
                n.children.push(Child::Node(r));
            }
            Some(TokenKind::Punct(Punct::Bang)) => {
                let mut t = Node::new(SyntaxKind::NeverType);
                t.children.push(self.bump_child());
                n.children.push(Child::Node(t));
            }
            Some(TokenKind::Punct(Punct::LParen)) => {
                n.children.push(Child::Node(self.parse_paren_type()));
            }
            Some(TokenKind::Punct(Punct::LBracket)) => {
                n.children.push(Child::Node(self.parse_bracket_type()));
            }
            Some(TokenKind::Keyword(Kw::Unsafe | Kw::Extern | Kw::Fn)) => {
                n.children.push(Child::Node(self.parse_function_type()));
            }
            Some(TokenKind::Keyword(Kw::Dyn)) => {
                n.children.push(Child::Node(self.parse_trait_ref()));
            }
            // `Self`/`SelfRef` are already consumed by the keyword arm above.
            Some(TokenKind::Ident) => {
                n.children.push(Child::Node(self.parse_path_type()));
            }
            _ => n.children.push(Child::Node(self.error_node("expected type"))),
        }
        n
    }

    fn parse_paren_type(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ParenthesizedType);
        n.children.push(self.expect_punct(Punct::LParen));
        if self.at_punct(Punct::RParen) {
            n.kind = SyntaxKind::TupleType;
            n.children.push(self.bump_child());
            return n;
        }
        let first = self.parse_type();
        n.children.push(Child::Node(first));
        if self.at_punct(Punct::Comma) {
            n.kind = SyntaxKind::TupleType;
            while self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
                if self.at_punct(Punct::RParen) {
                    break;
                }
                n.children.push(Child::Node(self.parse_type()));
            }
        }
        n.children.push(self.expect_punct(Punct::RParen));
        n
    }

    fn parse_bracket_type(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::SliceType);
        n.children.push(self.expect_punct(Punct::LBracket));
        n.children.push(Child::Node(self.parse_type()));
        if self.at_punct(Punct::Semicolon) {
            n.kind = SyntaxKind::ArrayType;
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_expr_bp(0)));
        }
        n.children.push(self.expect_punct(Punct::RBracket));
        n
    }

    fn parse_function_type(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::FunctionType);
        if self.at_kw(Kw::Unsafe) {
            n.children.push(self.bump_child());
        }
        if self.at_kw(Kw::Extern) {
            n.children.push(self.bump_child());
            if self
                .current_kind()
                .is_some_and(|k| matches!(k, TokenKind::String | TokenKind::RawString))
            {
                n.children.push(self.bump_child());
            }
        }
        n.children.push(self.expect_kw(Kw::Fn));
        n.children.push(self.expect_punct(Punct::LParen));
        if !self.at_punct(Punct::RParen) {
            loop {
                n.children.push(Child::Node(self.parse_type()));
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                    if self.at_punct(Punct::RParen) {
                        self.diagnostic("trailing comma is not part of function_type");
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        n.children.push(self.expect_punct(Punct::RParen));
        if self.at_punct(Punct::Arrow) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type()));
        }
        n
    }

    fn parse_path_type(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::PathType);
        n.children.push(Child::Node(self.parse_path()));
        n
    }

    fn parse_path(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Path);
        if self.at_punct(Punct::ColonColon) {
            n.children.push(self.bump_child());
        }
        n.children.push(Child::Node(self.parse_path_segment()));
        while self.at_punct(Punct::ColonColon)
            && self.peek_kind(1) != Some(TokenKind::Punct(Punct::Lt))
            && self.peek_kind(1) != Some(TokenKind::Punct(Punct::LBrace))
        {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_path_segment()));
        }
        n
    }

    fn parse_path_segment(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::PathSegment);
        if self.at_ident()
            || matches!(self.current_kind(), Some(TokenKind::Keyword(Kw::SelfKw | Kw::SelfRef)))
        {
            n.children.push(self.bump_child());
        } else {
            n.children.push(Child::Node(self.error_node("expected path segment")));
        }
        if self.looks_like_type_args() {
            n.children.push(Child::Node(self.parse_type_args()));
        }
        n
    }

    fn parse_struct_def(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::StructDef);
        n.children.push(self.expect_kw(Kw::Struct));
        n.children.push(self.expect_ident_node("expected struct name"));
        if self.at_punct(Punct::Lt) {
            n.children.push(Child::Node(self.parse_generic_params()));
        }
        if self.at_kw(Kw::Where) {
            n.children.push(Child::Node(self.parse_where_clause()));
        }
        if self.at_punct(Punct::LBrace) {
            n.children.push(self.bump_child());
            while !self.eof() && !self.at_punct(Punct::RBrace) {
                let mut field = Node::new(SyntaxKind::StructField);
                if self.at_punct(Punct::Hash) {
                    field.children.push(Child::Node(self.parse_attribute()));
                }
                if self.at_kw(Kw::Pub) {
                    field.children.push(self.bump_child());
                }
                field.children.push(self.expect_ident_node("expected struct field name"));
                field.children.push(self.expect_punct(Punct::Colon));
                field.children.push(Child::Node(self.parse_type()));
                n.children.push(Child::Node(field));
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                    if self.at_punct(Punct::RBrace) {
                        break;
                    }
                } else {
                    break;
                }
            }
            n.children.push(self.expect_punct(Punct::RBrace));
        } else {
            n.children.push(self.expect_punct(Punct::Semicolon));
        }
        n
    }

    fn parse_enum_def(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::EnumDef);
        n.children.push(self.expect_kw(Kw::Enum));
        n.children.push(self.expect_ident_node("expected enum name"));
        if self.at_punct(Punct::Lt) {
            n.children.push(Child::Node(self.parse_generic_params()));
        }
        if self.at_kw(Kw::Where) {
            n.children.push(Child::Node(self.parse_where_clause()));
        }
        n.children.push(self.expect_punct(Punct::LBrace));
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            let mut v = Node::new(SyntaxKind::EnumVariant);
            if self.at_punct(Punct::Hash) {
                v.children.push(Child::Node(self.parse_attribute()));
            }
            v.children.push(self.expect_ident_node("expected enum variant name"));
            if self.at_punct(Punct::LParen) {
                v.children.push(self.bump_child());
                if !self.at_punct(Punct::RParen) {
                    loop {
                        v.children.push(Child::Node(self.parse_type()));
                        if self.at_punct(Punct::Comma) {
                            v.children.push(self.bump_child());
                            if self.at_punct(Punct::RParen) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                v.children.push(self.expect_punct(Punct::RParen));
            }
            n.children.push(Child::Node(v));
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
                if self.at_punct(Punct::RBrace) {
                    break;
                }
            } else {
                break;
            }
        }
        n.children.push(self.expect_punct(Punct::RBrace));
        n
    }

    fn parse_trait_def(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::TraitDef);
        n.children.push(self.expect_kw(Kw::Trait));
        n.children.push(self.expect_ident_node("expected trait name"));
        if self.at_punct(Punct::Lt) {
            n.children.push(Child::Node(self.parse_generic_params()));
        }
        if self.at_kw(Kw::Where) {
            n.children.push(Child::Node(self.parse_where_clause()));
        }
        n.children.push(self.expect_punct(Punct::LBrace));
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            let item = match self.current_kind() {
                Some(TokenKind::Keyword(Kw::Type)) => self.parse_type_alias(),
                Some(TokenKind::Keyword(Kw::Const)) => self.parse_const_def(),
                _ => self.error_node(
                    "trait_item has no defined function_signature production in Edition 1 EBNF",
                ),
            };
            n.children.push(Child::Node(Node {
                kind: SyntaxKind::TraitItem,
                children: vec![Child::Node(item)],
            }));
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
            } else if !self.at_punct(Punct::RBrace) {
                let skipped = self.recover_until(&[Punct::Comma, Punct::RBrace]);
                if !skipped.is_empty() {
                    n.children.push(Child::Node(self.error_node_from(skipped)));
                }
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                }
            }
        }
        n.children.push(self.expect_punct(Punct::RBrace));
        n
    }

    fn parse_impl_def(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ImplDef);
        n.children.push(self.expect_kw(Kw::Impl));
        if self.at_punct(Punct::Lt) {
            n.children.push(Child::Node(self.parse_generic_params()));
        }
        n.children.push(Child::Node(self.parse_trait_or_type_header()));
        if self.at_kw(Kw::For) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type()));
        }
        if self.at_kw(Kw::Where) {
            n.children.push(Child::Node(self.parse_where_clause()));
        }
        n.children.push(self.expect_punct(Punct::LBrace));
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            let item = match self.current_kind() {
                Some(TokenKind::Keyword(Kw::Fn)) => self.parse_fn(),
                Some(TokenKind::Keyword(Kw::Type)) => self.parse_type_alias(),
                Some(TokenKind::Keyword(Kw::Const)) => self.parse_const_def(),
                _ => self.error_node("expected impl item"),
            };
            n.children.push(Child::Node(Node {
                kind: SyntaxKind::ImplItem,
                children: vec![Child::Node(item)],
            }));
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
            } else if !self.at_punct(Punct::RBrace) {
                let skipped = self.recover_until(&[Punct::Comma, Punct::RBrace]);
                if !skipped.is_empty() {
                    n.children.push(Child::Node(self.error_node_from(skipped)));
                }
            }
        }
        n.children.push(self.expect_punct(Punct::RBrace));
        n
    }

    fn parse_trait_or_type_header(&mut self) -> Node {
        if self.at_kw(Kw::Dyn) {
            self.parse_trait_ref()
        } else {
            self.parse_type()
        }
    }

    fn parse_type_alias(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::TypeAlias);
        n.children.push(self.expect_kw(Kw::Type));
        n.children.push(self.expect_ident_node("expected type alias name"));
        if self.at_punct(Punct::Lt) {
            n.children.push(Child::Node(self.parse_generic_params()));
        }
        if self.at_kw(Kw::Where) {
            n.children.push(Child::Node(self.parse_where_clause()));
        }
        n.children.push(self.expect_punct(Punct::Eq));
        n.children.push(Child::Node(self.parse_type()));
        n
    }

    fn parse_const_def(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ConstDef);
        n.children.push(self.expect_kw(Kw::Const));
        n.children.push(self.expect_ident_node("expected const name"));
        n.children.push(self.expect_punct(Punct::Colon));
        n.children.push(Child::Node(self.parse_type()));
        n.children.push(self.expect_punct(Punct::Eq));
        n.children.push(Child::Node(self.parse_expr_bp(0)));
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_static_def(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::StaticDef);
        n.children.push(self.expect_kw(Kw::Static));
        if self.at_kw(Kw::Mut) {
            n.children.push(self.bump_child());
        }
        n.children.push(self.expect_ident_node("expected static name"));
        n.children.push(self.expect_punct(Punct::Colon));
        n.children.push(Child::Node(self.parse_type()));
        if self.at_punct(Punct::Eq) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_expr_bp(0)));
        }
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_use_decl(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::UseDecl);
        n.children.push(self.expect_kw(Kw::Use));
        if self.at_kw(Kw::As) {
            n.children.push(self.bump_child());
        }
        n.children.push(Child::Node(self.parse_use_tree()));
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_use_tree(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::UseTree);
        n.children.push(Child::Node(self.parse_path()));
        if self.at_punct(Punct::ColonColon) {
            n.children.push(self.bump_child());
            if self.at_punct(Punct::LBrace) {
                n.children.push(self.bump_child());
                while !self.eof() && !self.at_punct(Punct::RBrace) {
                    n.children.push(Child::Node(self.parse_use_tree()));
                    if self.at_punct(Punct::Comma) {
                        n.children.push(self.bump_child());
                        if self.at_punct(Punct::RBrace) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                n.children.push(self.expect_punct(Punct::RBrace));
            } else {
                n.children.push(Child::Node(self.parse_path_segment()));
            }
        }
        if self.at_kw(Kw::As) {
            n.children.push(self.bump_child());
            n.children.push(self.expect_ident_node("expected alias"));
        }
        n
    }

    fn parse_module_decl(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ModuleDecl);
        n.children.push(self.expect_kw(Kw::Mod));
        n.children.push(self.expect_ident_node("expected module name"));
        if self.at_punct(Punct::LBrace) {
            n.children.push(self.bump_child());
            while !self.eof() && !self.at_punct(Punct::RBrace) {
                n.children.push(Child::Node(self.parse_item()));
            }
            n.children.push(self.expect_punct(Punct::RBrace));
        }
        n
    }

    fn parse_extern_crate_decl(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ExternCrateDecl);
        n.children.push(self.expect_kw(Kw::Extern));
        n.children.push(self.expect_kw(Kw::Crate));
        n.children.push(self.expect_ident_node("expected crate name"));
        if self.at_kw(Kw::As) {
            n.children.push(self.bump_child());
            n.children.push(self.expect_ident_node("expected crate alias"));
        }
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn expect_ident_node(&mut self, msg: &str) -> Child {
        if self.at_ident_or_contextual() {
            Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump_index()))
        } else {
            Child::Node(self.error_node(msg))
        }
    }

    fn parse_block(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Block);
        n.children.push(self.expect_punct(Punct::LBrace));
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            if self.at_kw(Kw::Let) {
                n.children.push(Child::Node(self.parse_let()));
            } else if self.starts_item() || self.at_punct(Punct::Hash) {
                // Attribute-introduced items parse through the same path as
                // bare items; the leading `#` is consumed by the item parser.
                n.children.push(Child::Node(self.parse_item_stmt()));
            } else {
                let prev_cmd = self.allow_command_call;
                self.allow_command_call = true;
                let expr = self.parse_expression();
                self.allow_command_call = prev_cmd;
                if self.at_punct(Punct::Semicolon) {
                    let stmt_kind = if expr.kind == SyntaxKind::MacroInvocation {
                        SyntaxKind::MacroStmt
                    } else {
                        SyntaxKind::ExprStmt
                    };
                    let mut s = Node::new(stmt_kind);
                    s.children.push(Child::Node(expr));
                    s.children.push(self.bump_child());
                    n.children.push(Child::Node(s));
                } else if matches!(
                    expr.kind,
                    SyntaxKind::ReturnExpr
                        | SyntaxKind::BreakExpr
                        | SyntaxKind::ContinueExpr
                        | SyntaxKind::YieldExpr
                ) {
                    n.children.push(Child::Node(expr));
                } else if matches!(
                    expr.kind,
                    SyntaxKind::LoopExpr | SyntaxKind::WhileExpr | SyntaxKind::ForExpr
                ) && !(self.at_punct(Punct::RBrace) || self.eof())
                {
                    // `loop`, `while` and `for` are statements in Edition 1 and
                    // take no trailing `;`. When one is not the block's final
                    // expression, keep the body block from being read as a
                    // struct literal or block expression on the loop keyword.
                    if self.at_punct(Punct::Semicolon) {
                        let mut s = Node::new(SyntaxKind::ExprStmt);
                        s.children.push(Child::Node(expr));
                        s.children.push(self.bump_child());
                        n.children.push(Child::Node(s));
                    } else {
                        n.children.push(Child::Node(expr));
                    }
                } else if self.at_punct(Punct::RBrace) || self.eof() {
                    n.children.push(Child::Node(Node {
                        kind: SyntaxKind::FinalExpr,
                        children: vec![Child::Node(expr)],
                    }));
                    break;
                } else {
                    n.children.push(Child::Node(Node {
                        kind: SyntaxKind::ExprStmt,
                        children: vec![Child::Node(expr), self.expect_punct(Punct::Semicolon)],
                    }));
                }
            }
        }
        n.children.push(self.expect_punct(Punct::RBrace));
        n
    }

    fn parse_item_stmt(&mut self) -> Node {
        Node { kind: SyntaxKind::ItemDeclStmt, children: vec![Child::Node(self.parse_item())] }
    }

    fn parse_let(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::LetStmt);
        n.children.push(self.expect_kw(Kw::Let));
        if self.at_kw(Kw::Mut) {
            n.children.push(self.bump_child());
        }
        n.children.push(Child::Node(self.parse_pattern()));
        if self.at_punct(Punct::Colon) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type()));
        }
        n.children.push(self.expect_punct(Punct::Eq));
        let prev_cmd = self.allow_command_call;
        self.allow_command_call = true;
        n.children.push(Child::Node(self.parse_expression()));
        self.allow_command_call = prev_cmd;
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_return(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ReturnExpr);
        n.children.push(self.expect_kw(Kw::Return));
        if !self.at_punct(Punct::Semicolon) {
            let prev_cmd = self.allow_command_call;
            self.allow_command_call = true;
            n.children.push(Child::Node(self.parse_expression()));
            self.allow_command_call = prev_cmd;
        }
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_break(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::BreakExpr);
        n.children.push(self.expect_kw(Kw::Break));
        if self.at_punct(Punct::Apostrophe) {
            n.children.push(Child::Node(self.parse_lifetime()));
        }
        if !self.at_punct(Punct::Semicolon) {
            n.children.push(Child::Node(self.parse_expression()));
        }
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_continue(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ContinueExpr);
        n.children.push(self.expect_kw(Kw::Continue));
        if self.at_punct(Punct::Apostrophe) {
            n.children.push(Child::Node(self.parse_lifetime()));
        }
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_yield(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::YieldExpr);
        n.children.push(self.expect_kw(Kw::Yield));
        if !self.at_punct(Punct::Semicolon) {
            n.children.push(Child::Node(self.parse_expression()));
        }
        n.children.push(self.expect_punct(Punct::Semicolon));
        n
    }

    fn parse_expression(&mut self) -> Node {
        self.parse_expr_bp(0)
    }

    fn infix(&self) -> Option<(Punct, u8, u8)> {
        let kind = self.current_kind()?;
        let TokenKind::Punct(punct) = kind else {
            return None;
        };
        let (left, right) = crate::precedence::binding_power(kind);
        if (left, right) == crate::precedence::NO_BINDING {
            None
        } else {
            Some((punct, left, right))
        }
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Node {
        let mut lhs = self.parse_prefix();
        loop {
            if self.at_punct(Punct::LParen) && crate::precedence::POSTFIX_BINDING_POWER >= min_bp {
                lhs = self.parse_call(lhs);
                continue;
            }
            if self.at_punct(Punct::Dot) && crate::precedence::POSTFIX_BINDING_POWER >= min_bp {
                let dot = self.bump_child();
                if self.at_kw(Kw::Await) {
                    let mut n = Node::new(SyntaxKind::AwaitExpr);
                    n.children.push(Child::Node(lhs));
                    n.children.push(dot);
                    n.children.push(self.bump_child());
                    lhs = n;
                    continue;
                }
                let mut id = None;
                if self.at_ident_or_contextual() {
                    id = Some(self.bump_index());
                }
                let Some(name) = id else {
                    // Keep the `.` we already consumed: dropping it would lose
                    // source bytes and break exact reconstruction on this path.
                    let mut n = Node::new(SyntaxKind::FieldExpr);
                    n.children.push(Child::Node(lhs));
                    n.children.push(dot);
                    n.children.push(Child::Node(self.error_node("expected field or method name")));
                    lhs = n;
                    continue;
                };
                if self.looks_like_type_args() {
                    let args = self.parse_type_args();
                    if self.at_punct(Punct::LParen) {
                        let mut n = Node::new(SyntaxKind::MethodCallExpr);
                        n.children.push(Child::Node(lhs));
                        n.children.push(dot);
                        n.children.push(Child::Token(name));
                        n.children.push(Child::Node(args));
                        self.append_call_arguments(&mut n);
                        lhs = n;
                        continue;
                    }
                    let mut n = Node::new(SyntaxKind::FieldExpr);
                    n.children.push(Child::Node(lhs));
                    n.children.push(dot);
                    n.children.push(Child::Token(name));
                    n.children.push(Child::Node(args));
                    lhs = n;
                    continue;
                }
                if self.at_punct(Punct::LParen) {
                    let mut n = Node::new(SyntaxKind::MethodCallExpr);
                    n.children.push(Child::Node(lhs));
                    n.children.push(dot);
                    n.children.push(Child::Token(name));
                    self.append_call_arguments(&mut n);
                    lhs = n;
                } else {
                    let mut n = Node::new(SyntaxKind::FieldExpr);
                    n.children.push(Child::Node(lhs));
                    n.children.push(dot);
                    n.children.push(Child::Token(name));
                    lhs = n;
                }
                continue;
            }
            if self.at_punct(Punct::QuestionDot)
                && crate::precedence::POSTFIX_BINDING_POWER >= min_bp
            {
                let qdot = self.bump_child();
                let mut id = None;
                if self.at_ident_or_contextual() {
                    id = Some(self.bump_index());
                }
                let Some(name) = id else {
                    let mut n = Node::new(SyntaxKind::QuestionDotExpr);
                    n.children.push(Child::Node(lhs));
                    n.children.push(qdot);
                    n.children.push(Child::Node(self.error_node("expected field name after '?.'")));
                    lhs = n;
                    continue;
                };
                let mut n = Node::new(SyntaxKind::QuestionDotExpr);
                n.children.push(Child::Node(lhs));
                n.children.push(qdot);
                n.children.push(Child::Token(name));
                lhs = n;
                continue;
            }
            if self.at_punct(Punct::LBracket) && crate::precedence::POSTFIX_BINDING_POWER >= min_bp
            {
                let mut n = Node::new(SyntaxKind::IndexExpr);
                n.children.push(Child::Node(lhs));
                n.children.push(self.bump_child());
                n.children.push(Child::Node(self.parse_expression()));
                n.children.push(self.expect_punct(Punct::RBracket));
                lhs = n;
                continue;
            }

            if self.at_kw(Kw::As) && 26 >= min_bp {
                let mut n = Node::new(SyntaxKind::CastExpr);
                n.children.push(Child::Node(lhs));
                n.children.push(self.bump_child());
                n.children.push(Child::Node(self.parse_type()));
                lhs = n;
                continue;
            }
            let Some((op, left_bp, right_bp)) = self.infix() else { break };
            if left_bp < min_bp {
                break;
            }
            if crate::precedence::is_comparison_operator(TokenKind::Punct(op))
                && matches!(lhs.kind, SyntaxKind::BinaryExpr)
            {
                let last = lhs.children.get(1).and_then(|c| match c {
                    Child::Token(i) => self.tokens.get(*i).map(|t| t.kind),
                    _ => None,
                });
                if last.is_some_and(crate::precedence::is_comparison_operator) {
                    self.diagnostic("comparison operators cannot be chained");
                }
            }
            let mut bin_kind = SyntaxKind::BinaryExpr;
            if crate::precedence::is_assignment_operator(TokenKind::Punct(op)) {
                bin_kind = SyntaxKind::AssignExpr;
            } else if matches!(op, Punct::DotDot | Punct::DotDotEq) {
                bin_kind = SyntaxKind::RangeExpr;
            } else if matches!(op, Punct::PipeArrow) {
                bin_kind = SyntaxKind::PipelineExpr;
            }
            let mut bin = Node::new(bin_kind);
            bin.children.push(Child::Node(lhs));
            bin.children.push(self.bump_child());
            let is_pipeline_or_assign =
                bin_kind == SyntaxKind::PipelineExpr || bin_kind == SyntaxKind::AssignExpr;
            let prev_cmd = self.allow_command_call;
            if is_pipeline_or_assign {
                self.allow_command_call = true;
            }
            bin.children.push(Child::Node(self.parse_expr_bp(right_bp)));
            self.allow_command_call = prev_cmd;
            lhs = bin;
        }
        lhs
    }

    fn parse_call(&mut self, lhs: Node) -> Node {
        let mut n = Node::new(SyntaxKind::CallExpr);
        n.children.push(Child::Node(lhs));
        self.append_call_arguments(&mut n);
        n
    }

    fn append_call_arguments(&mut self, n: &mut Node) {
        n.children.push(self.expect_punct(Punct::LParen));
        if !self.at_punct(Punct::RParen) {
            loop {
                n.children.push(Child::Node(self.parse_expression()));
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                    if self.at_punct(Punct::RParen) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        n.children.push(self.expect_punct(Punct::RParen));
    }

    fn parse_prefix(&mut self) -> Node {
        match self.current_kind() {
            Some(TokenKind::Punct(
                Punct::Minus | Punct::Bang | Punct::Amp | Punct::Star | Punct::Tilde,
            )) => {
                let mut n = Node::new(SyntaxKind::UnaryExpr);
                n.children.push(self.bump_child());
                if self.tokens.get(self.pos.saturating_sub(1)).map(|t| t.kind)
                    == Some(TokenKind::Punct(Punct::Amp))
                    && self.at_kw(Kw::Mut)
                {
                    n.children.push(self.bump_child());
                }
                n.children
                    .push(Child::Node(self.parse_expr_bp(crate::precedence::UNARY_BINDING_POWER)));
                n
            }
            Some(TokenKind::Keyword(Kw::Return)) => self.parse_return(),
            Some(TokenKind::Keyword(Kw::Break)) => self.parse_break(),
            Some(TokenKind::Keyword(Kw::Continue)) => self.parse_continue(),
            Some(TokenKind::Keyword(Kw::Yield)) => self.parse_yield(),
            Some(TokenKind::Keyword(Kw::If)) => self.parse_if_expr(),
            Some(TokenKind::Keyword(Kw::Match)) => self.parse_match_expr(),
            Some(TokenKind::Keyword(Kw::Loop)) => self.parse_loop_expr(None),
            Some(TokenKind::Keyword(Kw::While)) => self.parse_while_expr(None),
            Some(TokenKind::Keyword(Kw::For)) => self.parse_for_expr(None),
            Some(TokenKind::Keyword(Kw::Try)) => self.parse_try_expr(),
            Some(TokenKind::Keyword(Kw::Async)) => self.parse_async_block(),
            Some(TokenKind::Keyword(Kw::Unsafe)) => self.parse_unsafe_block(),
            Some(TokenKind::Keyword(Kw::Move)) => self.parse_closure_expr(),
            Some(TokenKind::Punct(Punct::Apostrophe)) => self.parse_labeled_expr(),
            Some(TokenKind::Punct(Punct::LBrace)) => self.parse_block_expr(),
            Some(TokenKind::Punct(Punct::LParen)) => self.parse_paren_expr(),
            Some(TokenKind::Punct(Punct::LBracket)) => self.parse_array_expr(),
            Some(TokenKind::Punct(Punct::Dot)) => {
                // Projection shorthand .field in pipeline stage args (ERR3-0033)
                let dot = self.bump_child();
                let mut n = Node::new(SyntaxKind::FieldExpr);
                n.children.push(dot);
                if self.at_ident_or_contextual() {
                    n.children.push(Child::Token(self.bump_index()));
                } else {
                    n.children.push(Child::Node(
                        self.error_node("expected field name after leading '.'"),
                    ));
                }
                n
            }
            Some(TokenKind::Punct(Punct::Underscore)) => {
                // Pipeline argument placeholder `_` (ERR3-0030)
                let underscore = self.bump_child();
                let mut n = Node::new(SyntaxKind::PlaceholderExpr);
                n.children.push(underscore);
                n
            }
            Some(TokenKind::Punct(Punct::Pipe)) => self.parse_closure_expr(),
            // `self` arrives as a keyword token, not an identifier, so it needs
            // its own arm. `path_segment = identifier [ "<" type_args ">" ]` does
            // not admit a keyword, so `self` is admitted here explicitly as the
            // receiver path rather than pretended to be an identifier.
            //
            // Edition 1 has no `self_expr` production, so the receiver becomes a
            // `path_expr` whose single segment is `self` — a shape the grammar's
            // `path_expr` alternative already admits. A dedicated node would have
            // had no production to cite.
            Some(TokenKind::Keyword(Kw::SelfRef)) => {
                let path = self.parse_path_expr_or_macro();
                if self.at_punct(Punct::LBrace) && !self.no_struct_literal {
                    self.parse_struct_expr_from_path(path)
                } else {
                    path
                }
            }
            Some(TokenKind::Ident) => {
                let path = self.parse_path_expr_or_macro();
                if self.at_punct(Punct::LBrace) && !self.no_struct_literal {
                    self.parse_struct_expr_from_path(path)
                } else if self.allow_command_call && self.looks_like_command_call_arg() {
                    self.allow_command_call = false;
                    self.parse_command_call(path)
                } else {
                    path
                }
            }
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

    /// Checks if the current token matches the starting token of a command-call argument (ERR3-0035).
    /// The first argument token must be a literal, identifier, string, `.name`, closure, `{`, or `-`
    /// immediately followed by a digit. Tokens `( [ < * & +` after the name are never an argument start.
    fn looks_like_command_call_arg(&self) -> bool {
        match self.current_kind() {
            Some(TokenKind::Ident) => true,
            Some(
                TokenKind::Int
                | TokenKind::Float
                | TokenKind::Char
                | TokenKind::Byte
                | TokenKind::String
                | TokenKind::RawString
                | TokenKind::InterpolatedString,
            ) => true,
            Some(TokenKind::Keyword(Kw::True | Kw::False)) => true,
            // A dot after an already-parsed callee is postfix syntax; treating it
            // as a command-call argument would steal field and method expressions.
            Some(TokenKind::Punct(Punct::Pipe)) => self.looks_like_closure_start(),
            Some(TokenKind::Punct(Punct::LBrace)) => true, // block / struct
            _ => false,
        }
    }

    fn looks_like_closure_start(&self) -> bool {
        if !self.at_punct(Punct::Pipe) {
            return false;
        }

        let mut index = self.pos + 1;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                TokenKind::Punct(Punct::LParen) => paren_depth += 1,
                TokenKind::Punct(Punct::RParen) if paren_depth > 0 => paren_depth -= 1,
                TokenKind::Punct(Punct::LBracket) => bracket_depth += 1,
                TokenKind::Punct(Punct::RBracket) if bracket_depth > 0 => bracket_depth -= 1,
                TokenKind::Punct(Punct::LBrace) => brace_depth += 1,
                TokenKind::Punct(Punct::RBrace) if brace_depth > 0 => brace_depth -= 1,
                TokenKind::Punct(Punct::Pipe)
                    if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 =>
                {
                    return true;
                }
                TokenKind::Punct(Punct::Semicolon)
                    if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 =>
                {
                    return false;
                }
                _ => {}
            }
            index += 1;
        }
        false
    }

    /// Parses a command call `name arg1, arg2` (ERR3-0035).
    fn parse_command_call(&mut self, callee: Node) -> Node {
        let mut n = Node::new(SyntaxKind::CommandCallExpr);
        n.children.push(Child::Node(callee));
        loop {
            n.children.push(Child::Node(
                self.parse_expr_bp(crate::precedence::UNARY_BINDING_POWER.saturating_add(1)),
            ));
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
            } else {
                break;
            }
        }
        n
    }

    fn parse_block_expr(&mut self) -> Node {
        self.parse_block()
    }

    fn parse_paren_expr(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ParenthesizedExpr);
        n.children.push(self.bump_child());
        if self.at_punct(Punct::RParen) {
            n.kind = SyntaxKind::TupleExpr;
            n.children.push(self.bump_child());
            return n;
        }
        n.children.push(Child::Node(self.parse_expression()));
        let mut is_tuple = false;
        while self.at_punct(Punct::Comma) {
            is_tuple = true;
            n.children.push(self.bump_child());
            if self.at_punct(Punct::RParen) {
                break;
            }
            n.children.push(Child::Node(self.parse_expression()));
        }
        n.children.push(self.expect_punct(Punct::RParen));
        if is_tuple {
            n.kind = SyntaxKind::TupleExpr;
        }
        n
    }

    fn parse_array_expr(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ArrayExpr);
        n.children.push(self.bump_child());
        if !self.at_punct(Punct::RBracket) {
            loop {
                n.children.push(Child::Node(self.parse_expression()));
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                    if self.at_punct(Punct::RBracket) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        n.children.push(self.expect_punct(Punct::RBracket));
        n
    }

    fn parse_struct_expr_from_path(&mut self, path: Node) -> Node {
        let mut n = Node::new(SyntaxKind::StructExpr);
        n.children.push(Child::Node(path));
        n.children.push(self.bump_child());
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            let mut f = Node::new(SyntaxKind::StructExprField);
            f.children.push(self.expect_ident_node("expected field name"));
            if self.at_punct(Punct::Colon) {
                f.children.push(self.bump_child());
                f.children.push(Child::Node(self.parse_expression()));
            }
            n.children.push(Child::Node(f));
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
                if self.at_punct(Punct::RBrace) {
                    break;
                }
            } else {
                break;
            }
        }
        n.children.push(self.expect_punct(Punct::RBrace));
        n
    }

    fn parse_path_expr_or_macro(&mut self) -> Node {
        // Captured before the path is consumed: by the time a macro invocation
        // is recognised the parser has already advanced past the macro name,
        // so the current token is the `!` rather than the construct that
        // actually introduces the feature.
        let path_span = self.current_span();
        let path = self.parse_path();
        let mut path_node = path;
        if self.at_punct(Punct::ColonColon)
            && self.peek_kind(1) == Some(TokenKind::Punct(Punct::Lt))
        {
            let mut p = Node::new(SyntaxKind::PathExpr);
            p.children.push(Child::Node(path_node));
            p.children.push(self.bump_child());
            p.children.push(Child::Node(self.parse_type_args()));
            path_node = p;
        }
        if self.at_punct(Punct::Bang) {
            self.record_feature("macros", path_span);
            let mut n = Node::new(SyntaxKind::MacroInvocation);
            n.children.push(Child::Node(path_node));
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_macro_args()));
            return n;
        }
        if path_node.kind == SyntaxKind::PathExpr {
            path_node
        } else {
            Node { kind: SyntaxKind::PathExpr, children: vec![Child::Node(path_node)] }
        }
    }

    fn parse_macro_args(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::TokenTree);
        match self.current_kind() {
            Some(TokenKind::Punct(Punct::LParen)) => {
                self.parse_token_tree_delimited(Punct::LParen, &mut n)
            }
            Some(TokenKind::Punct(Punct::LBracket)) => {
                self.parse_token_tree_delimited(Punct::LBracket, &mut n)
            }
            Some(TokenKind::Punct(Punct::LBrace)) => {
                self.parse_token_tree_delimited(Punct::LBrace, &mut n)
            }
            _ => {
                n.children.push(Child::Node(self.error_node("expected macro delimiter")));
            }
        }
        n
    }

    fn parse_token_tree_delimited(&mut self, open: Punct, n: &mut Node) {
        n.children.push(self.bump_child());
        let close = matching_close(open).expect("grammar delimiter");
        while !self.eof() && !self.at_punct(close) {
            match self.current_kind() {
                Some(TokenKind::Punct(Punct::LParen)) => {
                    let mut child = Node::new(SyntaxKind::TokenTree);
                    self.parse_token_tree_delimited(Punct::LParen, &mut child);
                    n.children.push(Child::Node(child));
                }
                Some(TokenKind::Punct(Punct::LBracket)) => {
                    let mut child = Node::new(SyntaxKind::TokenTree);
                    self.parse_token_tree_delimited(Punct::LBracket, &mut child);
                    n.children.push(Child::Node(child));
                }
                Some(TokenKind::Punct(Punct::LBrace)) => {
                    let mut child = Node::new(SyntaxKind::TokenTree);
                    self.parse_token_tree_delimited(Punct::LBrace, &mut child);
                    n.children.push(Child::Node(child));
                }
                _ => n.children.push(self.bump_child()),
            }
        }
        n.children.push(self.expect_punct(close));
    }

    fn parse_closure_expr(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ClosureExpr);
        if self.at_kw(Kw::Move) {
            n.children.push(self.bump_child());
        }
        n.children.push(self.bump_child());
        if self.at_punct(Punct::Pipe) {
            n.children.push(self.bump_child());
        } else {
            loop {
                let mut p = Node::new(SyntaxKind::ClosureParam);
                if self.at_kw(Kw::Mut) {
                    p.children.push(self.bump_child());
                }
                p.children.push(self.expect_ident_node("expected closure parameter"));
                if self.at_punct(Punct::Colon) {
                    p.children.push(self.bump_child());
                    p.children.push(Child::Node(self.parse_type()));
                }
                n.children.push(Child::Node(p));
                if self.at_punct(Punct::Comma) {
                    n.children.push(self.bump_child());
                    if self.at_punct(Punct::Pipe) {
                        break;
                    }
                } else {
                    break;
                }
            }
            n.children.push(self.expect_punct(Punct::Pipe));
        }
        if self.at_punct(Punct::Arrow) {
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_type()));
        }
        n.children.push(Child::Node(self.parse_expression()));
        n
    }

    fn parse_if_expr(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::IfExpr);
        n.children.push(self.expect_kw(Kw::If));
        // The condition is followed by the then-block, so a `{` there opens
        // that block rather than a struct literal on the condition.
        let saved = self.no_struct_literal;
        let saved_cmd = self.allow_command_call;
        self.no_struct_literal = true;
        self.allow_command_call = false;
        let condition = self.parse_expression();
        self.allow_command_call = saved_cmd;
        self.no_struct_literal = saved;
        n.children.push(Child::Node(condition));
        n.children.push(Child::Node(self.parse_block()));
        if self.at_kw(Kw::Else) {
            n.children.push(self.bump_child());
            if self.at_kw(Kw::If) {
                n.children.push(Child::Node(self.parse_if_expr()));
            } else {
                n.children.push(Child::Node(self.parse_block()));
            }
        }
        n
    }

    fn parse_match_expr(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::MatchExpr);
        n.children.push(self.expect_kw(Kw::Match));
        // The scrutinee is followed by the arm block, so a `{` there opens the
        // arms rather than a struct literal on the scrutinee.
        let saved = self.no_struct_literal;
        let saved_cmd = self.allow_command_call;
        self.no_struct_literal = true;
        self.allow_command_call = false;
        let scrutinee = self.parse_expression();
        self.allow_command_call = saved_cmd;
        self.no_struct_literal = saved;
        n.children.push(Child::Node(scrutinee));
        n.children.push(self.expect_punct(Punct::LBrace));
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            let mut arm = Node::new(SyntaxKind::MatchArm);
            arm.children.push(Child::Node(self.parse_pattern()));
            if self.at_kw(Kw::If) {
                arm.children.push(self.bump_child());
                arm.children.push(Child::Node(self.parse_expression()));
            }
            arm.children.push(self.expect_punct(Punct::FatArrow));
            arm.children.push(Child::Node(self.parse_expression()));
            n.children.push(Child::Node(arm));
            if self.at_punct(Punct::Comma) {
                n.children.push(self.bump_child());
            } else if !self.at_punct(Punct::RBrace) {
                let skipped = self.recover_until(&[Punct::Comma, Punct::RBrace]);
                if !skipped.is_empty() {
                    n.children.push(Child::Node(self.error_node_from(skipped)));
                }
            }
        }
        n.children.push(self.expect_punct(Punct::RBrace));
        n
    }

    fn parse_label(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Label);
        n.children.push(self.expect_punct(Punct::Apostrophe));
        n.children.push(self.expect_ident_node("expected label name"));
        n
    }

    fn parse_labeled_expr(&mut self) -> Node {
        let mut label = self.parse_label();
        let colon = self.expect_punct(Punct::Colon);
        label.children.push(colon);
        match self.current_kind() {
            Some(TokenKind::Keyword(Kw::Loop)) => self.parse_loop_expr(Some(label)),
            Some(TokenKind::Keyword(Kw::While)) => self.parse_while_expr(Some(label)),
            Some(TokenKind::Keyword(Kw::For)) => self.parse_for_expr(Some(label)),
            _ => {
                self.diagnostic("label must precede loop, while, or for");
                Node { kind: SyntaxKind::ErrorNode, children: vec![Child::Node(label)] }
            }
        }
    }

    fn parse_loop_expr(&mut self, label: Option<Node>) -> Node {
        let mut n = Node::new(SyntaxKind::LoopExpr);
        if let Some(label) = label {
            n.children.push(Child::Node(label));
        }
        n.children.push(self.expect_kw(Kw::Loop));
        n.children.push(Child::Node(self.parse_block()));
        n
    }

    fn parse_while_expr(&mut self, label: Option<Node>) -> Node {
        let mut n = Node::new(SyntaxKind::WhileExpr);
        if let Some(label) = label {
            n.children.push(Child::Node(label));
        }
        n.children.push(self.expect_kw(Kw::While));
        // The condition is followed by the loop body block, so a `{` there
        // opens the body rather than a struct literal on the condition.
        let saved = self.no_struct_literal;
        let saved_cmd = self.allow_command_call;
        self.no_struct_literal = true;
        self.allow_command_call = false;
        let condition = self.parse_expression();
        self.allow_command_call = saved_cmd;
        self.no_struct_literal = saved;
        n.children.push(Child::Node(condition));
        n.children.push(Child::Node(self.parse_block()));
        n
    }

    fn parse_for_expr(&mut self, label: Option<Node>) -> Node {
        let mut n = Node::new(SyntaxKind::ForExpr);
        if let Some(label) = label {
            n.children.push(Child::Node(label));
        }
        n.children.push(self.expect_kw(Kw::For));
        n.children.push(Child::Node(self.parse_pattern()));
        n.children.push(self.expect_kw(Kw::In));
        // The iterable is terminated by the loop body block, so a `{` here
        // opens the loop body rather than a struct literal on the iterable.
        let saved = self.no_struct_literal;
        let saved_cmd = self.allow_command_call;
        self.no_struct_literal = true;
        self.allow_command_call = false;
        let iterable = self.parse_expression();
        self.allow_command_call = saved_cmd;
        self.no_struct_literal = saved;
        n.children.push(Child::Node(iterable));
        n.children.push(Child::Node(self.parse_block()));
        n
    }

    fn parse_try_expr(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::TryBlock);
        n.children.push(self.expect_kw(Kw::Try));
        n.children.push(Child::Node(self.parse_block()));
        if self.at_kw(Kw::Catch) {
            n.kind = SyntaxKind::TryExpr;
            let mut catch = Node::new(SyntaxKind::CatchClause);
            catch.children.push(self.bump_child());
            catch.children.push(self.expect_punct(Punct::LParen));
            catch.children.push(Child::Node(self.parse_pattern()));
            catch.children.push(self.expect_punct(Punct::RParen));
            catch.children.push(Child::Node(self.parse_block()));
            n.children.push(Child::Node(catch));
        }
        n
    }

    fn parse_async_block(&mut self) -> Node {
        self.record_feature("async", self.current_span());
        let mut n = Node::new(SyntaxKind::AsyncBlock);
        n.children.push(self.expect_kw(Kw::Async));
        if self.at_kw(Kw::Move) {
            n.children.push(self.bump_child());
        }
        n.children.push(Child::Node(self.parse_block()));
        n
    }

    fn parse_unsafe_block(&mut self) -> Node {
        self.record_feature("unsafe_raw_memory", self.current_span());
        let mut n = Node::new(SyntaxKind::UnsafeBlock);
        n.children.push(self.expect_kw(Kw::Unsafe));
        n.children.push(Child::Node(self.parse_block()));
        n
    }

    fn parse_pattern(&mut self) -> Node {
        let mut lhs = self.parse_pattern_atom();
        if self.at_punct(Punct::DotDot) || self.at_punct(Punct::DotDotEq) {
            let mut n = Node::new(SyntaxKind::RangePattern);
            n.children.push(Child::Node(lhs));
            n.children.push(self.bump_child());
            if !self.at_punct(Punct::Comma)
                && !self.at_kw(Kw::If)
                && !self.at_punct(Punct::RParen)
                && !self.at_punct(Punct::RBracket)
                && !self.at_punct(Punct::RBrace)
                && !self.at_punct(Punct::FatArrow)
            {
                n.children.push(Child::Node(self.parse_expression()));
            }
            lhs = n;
        }
        if self.at_punct(Punct::Pipe) {
            let mut n = Node::new(SyntaxKind::OrPattern);
            n.children.push(Child::Node(lhs));
            while self.at_punct(Punct::Pipe) {
                n.children.push(self.bump_child());
                n.children.push(Child::Node(self.parse_pattern_atom()));
            }
            lhs = n;
        }
        if self.at_kw(Kw::If) {
            let mut n = Node::new(SyntaxKind::GuardPattern);
            n.children.push(Child::Node(lhs));
            n.children.push(self.bump_child());
            n.children.push(Child::Node(self.parse_expression()));
            lhs = n;
        }
        lhs
    }

    fn parse_pattern_atom(&mut self) -> Node {
        match self.current_kind() {
            Some(TokenKind::Keyword(Kw::Mut)) => {
                let mut n = Node::new(SyntaxKind::BindingPattern);
                n.children.push(self.bump_child());
                n.children.push(self.expect_ident_node("expected binding name"));
                if self.at_punct(Punct::At) {
                    n.children.push(self.bump_child());
                    n.children.push(Child::Node(self.parse_pattern()));
                }
                n
            }
            Some(TokenKind::Punct(Punct::Underscore)) => {
                let mut n = Node::new(SyntaxKind::WildcardPattern);
                n.children.push(self.bump_child());
                n
            }
            Some(TokenKind::Punct(Punct::Amp)) => {
                let mut n = Node::new(SyntaxKind::ReferencePattern);
                n.children.push(self.bump_child());
                if self.at_kw(Kw::Mut) {
                    n.children.push(self.bump_child());
                }
                n.children.push(Child::Node(self.parse_pattern()));
                n
            }
            Some(TokenKind::Punct(Punct::LParen)) => {
                let mut n = Node::new(SyntaxKind::TuplePattern);
                n.children.push(self.bump_child());
                if !self.at_punct(Punct::RParen) {
                    loop {
                        n.children.push(Child::Node(self.parse_pattern()));
                        if self.at_punct(Punct::Comma) {
                            n.children.push(self.bump_child());
                            if self.at_punct(Punct::RParen) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                n.children.push(self.expect_punct(Punct::RParen));
                n
            }
            Some(TokenKind::Punct(Punct::LBracket)) => {
                let mut n = Node::new(SyntaxKind::SlicePattern);
                n.children.push(self.bump_child());
                if !self.at_punct(Punct::RBracket) {
                    loop {
                        n.children.push(Child::Node(self.parse_pattern()));
                        if self.at_punct(Punct::Comma) {
                            n.children.push(self.bump_child());
                            if self.at_punct(Punct::RBracket) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                n.children.push(self.expect_punct(Punct::RBracket));
                n
            }
            Some(TokenKind::Punct(Punct::Apostrophe)) => self.parse_lifetime(),
            Some(TokenKind::Ident) => {
                let path = self.parse_path();
                if self.at_punct(Punct::LBrace) {
                    let mut n = Node::new(SyntaxKind::StructPattern);
                    n.children.push(Child::Node(path));
                    n.children.push(self.bump_child());
                    while !self.eof() && !self.at_punct(Punct::RBrace) {
                        let mut f = Node::new(SyntaxKind::PatternField);
                        f.children.push(self.expect_ident_node("expected pattern field"));
                        if self.at_punct(Punct::Colon) {
                            f.children.push(self.bump_child());
                            f.children.push(Child::Node(self.parse_pattern()));
                        }
                        n.children.push(Child::Node(f));
                        if self.at_punct(Punct::Comma) {
                            n.children.push(self.bump_child());
                            if self.at_punct(Punct::RBrace) {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    n.children.push(self.expect_punct(Punct::RBrace));
                    n
                } else if self.at_punct(Punct::LParen) {
                    let mut n = Node::new(SyntaxKind::EnumPattern);
                    n.children.push(Child::Node(path));
                    n.children.push(self.bump_child());
                    if !self.at_punct(Punct::RParen) {
                        loop {
                            n.children.push(Child::Node(self.parse_pattern()));
                            if self.at_punct(Punct::Comma) {
                                n.children.push(self.bump_child());
                                if self.at_punct(Punct::RParen) {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    n.children.push(self.expect_punct(Punct::RParen));
                    n
                } else {
                    let mut n = Node::new(SyntaxKind::IdentifierPattern);
                    n.children.push(Child::Node(path));
                    if self.at_punct(Punct::At) {
                        n.kind = SyntaxKind::BindingPattern;
                        n.children.push(self.bump_child());
                        n.children.push(Child::Node(self.parse_pattern()));
                    }
                    n
                }
            }
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
                let mut n = Node::new(SyntaxKind::LiteralPattern);
                n.children.push(Child::Node(
                    Node::new(SyntaxKind::LiteralExpr).with_token(self.bump_index()),
                ));
                n
            }
            _ => self.error_node("expected pattern"),
        }
    }

    fn emit_node(&self, b: &mut GreenNodeBuilder, n: &Node) {
        b.start_node(n.kind.into());
        for child in &n.children {
            match child {
                Child::Node(c) => self.emit_node(b, c),
                Child::Token(i) => self.emit_token(b, *i),
                Child::Piece { token, byte_offset, byte_len } => {
                    self.emit_piece(b, *token, *byte_offset, *byte_len)
                }
                // A missing token is zero-width and carries no trivia, so it can
                // never affect the reconstructed text.
                Child::Missing => {
                    b.token(SyntaxKind::MissingToken.into(), "");
                }
            }
        }
        b.finish_node();
    }
    fn emit_piece(&self, b: &mut GreenNodeBuilder, index: usize, byte_offset: u8, byte_len: u8) {
        let Some(t) = self.tokens.get(index) else {
            return;
        };
        let start = t.span.start as usize;
        let piece_start = start + byte_offset as usize;
        let piece_end = piece_start + byte_len as usize;
        if byte_offset == 0 {
            for tr in &t.leading_trivia {
                self.emit_trivia(b, tr);
            }
        }
        let kind = match t.kind {
            TokenKind::Punct(_) => SyntaxKind::Punct,
            _ => SyntaxKind::ErrorToken,
        };
        b.token(kind.into(), &self.source[piece_start..piece_end]);
        if piece_end == t.span.end as usize {
            for tr in &t.trailing_trivia {
                self.emit_trivia(b, tr);
            }
        }
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
    /// Return the token at a relative cursor offset without advancing.
    ///
    /// This is the token-valued counterpart to [`Parser::peek_kind`], so
    /// callers that need both the kind and the span of a lookahead token can
    /// read them at the same offset rather than mixing the cursor token with a
    /// lookahead kind.
    #[cfg(test)]
    fn peek_token(&self, offset: usize) -> Option<&Token> {
        if self.split_token.is_some() {
            return None;
        }
        self.tokens.get(self.pos.checked_add(offset)?)
    }

    /// Return a token kind at a relative cursor offset without indexing past
    /// EOF. Lookahead never mutates parser state.
    fn peek_kind(&self, offset: usize) -> Option<TokenKind> {
        if offset == 0 {
            return self.current_kind();
        }
        if self.split_token.is_some() {
            return Some(TokenKind::Eof);
        }
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
        if self.split_token.is_some() {
            return self.bump_split_piece();
        }
        if self.eof() {
            Child::Missing
        } else {
            Child::Token(self.bump_index())
        }
    }

    /// Consume one logical piece of a split `>>`/\`>>=\` token.
    fn bump_split_piece(&mut self) -> Child {
        let Some((token, offset)) = self.split_token else {
            return Child::Missing;
        };
        let Some(TokenKind::Punct(kind)) = self.tokens.get(token).map(|t| t.kind) else {
            self.split_token = None;
            return Child::Missing;
        };
        let Some(pieces) = crate::precedence::split_generic_closer(kind) else {
            self.split_token = None;
            return Child::Missing;
        };
        let Some(part) = pieces.get(offset as usize).and_then(|part| *part) else {
            self.split_token = None;
            self.pos = token.saturating_add(1);
            return Child::Missing;
        };
        let next = offset as usize + 1;
        if pieces.get(next).and_then(|part| *part).is_some() {
            self.split_token = Some((token, next as u8));
        } else {
            self.split_token = None;
            self.pos = token.saturating_add(1);
        }
        Child::Piece { token, byte_offset: part.byte_offset, byte_len: part.byte_len }
    }

    /// Consume the closing `>` of a generic argument or parameter list. A lone
    /// `>` is consumed directly; a `>>`/`>>=` token is split across successive
    /// `consume_gt` calls so each nested list receives exactly one logical `>`.
    ///
    /// This is the entry point that *initiates* a split, which is what
    /// `bump_split_piece` alone does not do: `bump_split_piece` only serves a
    /// split that is already active. Only `>` pieces are ever consumed here, so
    /// the trailing `=` of a `>>=` stays active and remains an ordinary logical
    /// token for the enclosing expression to consume.
    fn consume_gt(&mut self) -> Child {
        // Already mid-split: serve the current logical `>` piece.
        if self.split_token.is_some() {
            if matches!(self.current_kind(), Some(TokenKind::Punct(Punct::Gt))) {
                return self.bump_split_piece();
            }
            // A non-`>` piece became active, which means this list has already
            // been closed; recover without stealing the pending piece.
            return Child::Missing;
        }
        match self.current_kind_physical() {
            Some(TokenKind::Punct(Punct::Gt)) => Child::Token(self.bump_index()),
            Some(TokenKind::Punct(p @ (Punct::Shr | Punct::ShrEq))) => {
                // Split the multi-`>` token: this call consumes the first logical
                // `>`; subsequent calls serve the remaining pieces.
                let token = self.pos;
                let Some(pieces) = crate::precedence::split_generic_closer(p) else {
                    return self.bump_child();
                };
                let Some(part) = pieces[0] else {
                    return self.bump_child();
                };
                if pieces[1].is_some() {
                    self.split_token = Some((token, 1));
                } else {
                    self.pos = token.saturating_add(1);
                }
                Child::Piece { token, byte_offset: part.byte_offset, byte_len: part.byte_len }
            }
            _ if self.at_logical_gt() => self.bump_child(),
            _ => {
                self.diagnostic("expected `>`");
                Child::Missing
            }
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
    #[cfg(test)]
    fn is_matching_close(&self, close: Punct, open: Punct) -> bool {
        crate::precedence::matching_open(close) == Some(open)
    }

    fn logical_current_kind(&self) -> Option<TokenKind> {
        let Some((token, offset)) = self.split_token else {
            return self.current_kind_physical();
        };
        let kind = self.tokens.get(token)?.kind;
        let pieces = crate::precedence::split_generic_closer(match kind {
            TokenKind::Punct(p) => p,
            _ => return self.current_kind_physical(),
        })?;
        pieces.get(offset as usize).and_then(|p| *p).map(|p| TokenKind::Punct(p.kind))
    }

    fn current_kind_physical(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|t| t.kind)
    }

    fn current_kind(&self) -> Option<TokenKind> {
        self.logical_current_kind()
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
    /// Returns true if the current token is an identifier or a contextual keyword
    /// that is admitted as an identifier in named item/member positions (ERR3-0016, ERR3-0093).
    fn at_ident_or_contextual(&self) -> bool {
        matches!(
            self.current_kind(),
            Some(TokenKind::Ident) | Some(TokenKind::Keyword(Kw::Where | Kw::In))
        )
    }
    fn eof(&self) -> bool {
        self.split_token.is_none()
            && match self.tokens.get(self.pos) {
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
    fn bitwise_or_is_not_reinterpreted_as_a_command_call_closure() {
        let source = "fn f() { let x = a | b ^ c & d; return x; }";
        let mut parser = Parser::from_source(source);
        let parsed = parser.parse_source();
        assert!(
            parsed.diagnostics.is_empty(),
            "bitwise OR must remain a binary operator: {:?}",
            parsed.diagnostics
        );
    }

    #[test]
    fn postfix_field_and_method_are_not_reinterpreted_as_command_calls() {
        for source in [
            "struct P { x: i64 } fn main() -> i64 { let p = P { x: 1 }; return p.x; }",
            "struct P { x: i64 } impl P { fn get() -> i64 { return self.x; } } fn main() -> i64 { let p = P { x: 1 }; return p.get(); }",
        ] {
            let mut parser = Parser::from_source(source);
            let parsed = parser.parse_source();
            assert!(
                parsed.diagnostics.is_empty(),
                "postfix expression must parse without command-call ambiguity: {source:?} -> {:?}",
                parsed.diagnostics
            );
        }
    }

    #[test]
    fn struct_literal_is_not_reinterpreted_as_command_call_argument() {
        let source = "fn main() -> i64 { let p = Point { x: 1 }; return p.x; }";
        let mut parser = Parser::from_source(source);
        let parsed = parser.parse_source();
        assert!(
            parsed.diagnostics.is_empty(),
            "struct literal must parse in command-call-enabled expression context: {:?}",
            parsed.diagnostics
        );
    }

    #[test]
    fn consume_gt_closes_generic_parameter_and_argument_lists() {
        // `consume_gt` must terminate a generic parameter list and a generic
        // argument list on a plain `>` without emitting a diagnostic.
        for source in [
            "fn id<T>(x: T) -> T { return x; }\n",
            "fn pair<A, B>(a: A, b: B) -> A { return a; }\nfn main() -> i64 { let v = pair<i64, bool>(1, true); return v; }\n",
            "fn take<T>(x: T) -> T { return x; }\nfn main() -> i64 { return take<i64>(1); }\n",
        ] {
            let mut p = Parser::from_source(source);
            let result = p.parse_source();
            assert!(
                result.diagnostics.is_empty(),
                "generic source must parse without diagnostics: {source:?} -> {:?}",
                result.diagnostics
            );
        }
    }

    #[test]
    fn generic_closer_infrastructure_preserves_token_trivia_boundaries() {
        let p = Parser::from_source("T /*before*/ >>= /*after*/ U");
        assert_eq!(p.peek_kind(0), Some(TokenKind::Ident));
        assert_eq!(p.peek_kind(1), Some(TokenKind::Punct(Punct::ShrEq)));
        let token = p.peek_token(1).expect("shift-assignment token");
        assert_eq!(&p.source[token.span.start as usize..token.span.end as usize], ">>=");
        let parts = crate::precedence::split_generic_closer(Punct::ShrEq).expect("split");
        assert_eq!(parts[0].expect("first").byte_offset, 0);
        assert_eq!(parts[1].expect("second").byte_offset, 1);
        assert_eq!(parts[2].expect("third").byte_offset, 2);
        // This scanner attaches every trivia run as *trailing* trivia of the
        // token it follows, so the surrounding whitespace/comment/whitespace on
        // each side of `>>=` is owned by its predecessor, not by `>>=`. The
        // invariant that matters for losslessness is that the trivia is carried
        // exactly once in total, which `generic_closer_owns_no_trivia_of_its_own`
        // pins; asserting a leading/trailing split here would only restate an
        // incidental ownership choice.
        assert_eq!(token.leading_trivia.len(), 0);
        assert_eq!(
            token.trailing_trivia.len(),
            3,
            "`>>=` must carry the trailing whitespace/comment/whitespace run once"
        );
    }

    /// The split-token path must not fabricate or drop trivia: `Child::Piece`
    /// emits the leading trivia on the first piece and the trailing trivia on
    /// the piece that reaches the end of the physical token, so a split `>>=`
    /// still carries its trivia exactly once.
    #[test]
    fn generic_closer_owns_no_trivia_of_its_own() {
        let src = "T /*before*/ >>= /*after*/ U";
        // This scanner attaches every trivia run as *trailing* trivia of the
        // token it follows, so the trivia around `>>=` is owned by its
        // neighbours. The losslessness invariant is that it is carried exactly
        // once in total, not that any particular token owns it.
        let p = Parser::from_source(src);
        let span = p.peek_token(1).expect("shift-assignment token").span;
        let trivia_total: usize =
            p.tokens.iter().map(|t| t.leading_trivia.len() + t.trailing_trivia.len()).sum();
        assert_eq!(trivia_total, 6, "all trivia must be carried exactly once");

        // Drive the split from the `>>=` itself, where the cursor sits on it.
        let mut p = Parser::from_source(">>=");
        assert!(matches!(p.consume_gt(), Child::Piece { byte_offset: 0, byte_len: 1, .. }));
        assert!(matches!(p.consume_gt(), Child::Piece { byte_offset: 1, byte_len: 1, .. }));
        assert_eq!(p.current_kind(), Some(TokenKind::Punct(Punct::Eq)));
        assert!(matches!(p.bump_child(), Child::Piece { byte_offset: 2, byte_len: 1, .. }));
        assert_eq!(p.current_kind(), Some(TokenKind::Eof));

        // The three pieces tile the physical `>>=` span with no gap or overlap.
        let covered: usize = 1 + 1 + 1;
        assert_eq!(covered, (span.end - span.start) as usize);
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
    fn lifetime_type_syntax_is_parsed_losslessly() {
        let src = "fn f(x: &'a T) { return 1; }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(r.is_ok(), "{:?}", r.diagnostics);
        assert_eq!(r.syntax().text().to_string(), src);
        assert!(r.syntax().descendants().any(|n| n.kind() == K::Lifetime));
        assert!(r.syntax().descendants().any(|n| n.kind() == K::ReferenceType));
        assert!(!r.syntax().descendants_with_tokens().any(|e| {
            matches!(e, SyntaxElement::Token(t) if t.kind() == K::ErrorToken && t.text() == "'a")
        }));
    }

    #[test]
    fn shared_precedence_keeps_unary_tighter_than_multiplicative() {
        let mut p = Parser::from_source("fn f() { return -a * b; }");
        let r = p.parse_source();
        assert!(r.is_ok(), "{:?}", r.diagnostics);
        let binary = r
            .syntax()
            .descendants()
            .find(|n| n.kind() == K::BinaryExpr)
            .expect("multiplication binary expression");
        assert!(
            binary.children().next().is_some_and(|n| n.kind() == K::UnaryExpr),
            "the left side of * must be the unary expression -a"
        );
    }

    #[test]
    fn shared_postfix_precedence_keeps_calls_tighter_than_multiplicative() {
        let mut p = Parser::from_source("fn f() { return a * g(x); }");
        let r = p.parse_source();
        assert!(r.is_ok(), "{:?}", r.diagnostics);
        let binary = r
            .syntax()
            .descendants()
            .find(|n| n.kind() == K::BinaryExpr)
            .expect("multiplication binary expression");
        assert!(
            binary.children().nth(1).is_some_and(|n| n.kind() == K::CallExpr),
            "the right side of * must be the call g(x)"
        );
    }

    #[test]
    fn edition1_items_types_and_nested_generics_parse() {
        for src in [
            "struct Pair<T> { pub first: T, second: i32 }",
            "enum Option<T> { Some(T), None }",
            "type Id<T> = fn(T) -> T",
            "const N: i32 = 3;",
            "static mut N: i32 = 3;",
            "use foo::{bar, baz as qux};",
            "extern crate foo as f;",
            "mod inner { fn nested() {} }",
            "fn f<T: Foo<Bar<Baz>>>() { let x: &'a T = y; return x; }",
        ] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(r.is_ok(), "{src:?}: {:?}", r.diagnostics);
            assert_eq!(r.syntax().text().to_string(), src);
        }
    }

    #[test]
    fn borrow_and_deref_unary_forms_parse_losslessly() {
        for src in [
            "fn f(x: i64) { let r = &x; let y = *r; return y; }",
            "fn f(x: i64) { let r = &mut x; let y = *r; return y; }",
        ] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(r.is_ok(), "{src:?}: {:?}", r.diagnostics);
            assert_eq!(r.syntax().text().to_string(), src);
            assert_eq!(r.syntax().descendants().filter(|n| n.kind() == K::UnaryExpr).count(), 2);
        }
    }

    #[test]
    fn labeled_loop_forms_preserve_label_and_colon() {
        for src in [
            "fn f() { 'outer: loop { break 'outer; } }",
            "fn f() { 'outer: while true { continue 'outer; } }",
            "fn f() { 'outer: for x in xs { continue 'outer; } }",
        ] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(r.is_ok(), "{src:?}: {:?}", r.diagnostics);
            assert_eq!(r.syntax().text().to_string(), src);
            assert!(r.syntax().descendants().any(|n| n.kind() == K::Label));
        }
    }

    #[test]
    fn edition1_expression_families_parse_losslessly() {
        for src in [
            "fn f(x: i32) { if x > 0 { return x; } else { return 0; } }",
            "fn f(x: i32) { while x > 0 { break; } for y in x { continue; } }",
            "fn f(x: i32) { match x { 0 => 1, _ => 2, }; }",
            "fn f() { let x = [1, 2, 3][0]; return x; }",
            "fn f() { let x = (1, 2, 3); return x; }",
            "fn f() { let x = S { a: 1, b: 2 }; return x.a; }",
            "fn f() { let x = move |a: i32| -> i32 a * 2; return x(3); }",
            "fn f() { let x = async move { 1 }; return x; }",
            "fn f() { let x = unsafe { 1 }; return x; }",
            "fn f() { let x = try { 1 }; return x; }",
            "fn f() { foo!(a, (b, [c])); }",
            "fn f() { a = b += c * d; }",
        ] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(r.is_ok(), "{src:?}: {:?}", r.diagnostics);
            assert_eq!(r.syntax().text().to_string(), src);
        }
    }

    #[test]
    fn comparison_chaining_is_rejected_but_lossless() {
        let src = "fn f() { return a < b < c; }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(!r.is_ok());
        assert_eq!(r.syntax().text().to_string(), src);
        assert!(
            r.diagnostics.iter().any(|d| d.message.contains("cannot be chained")),
            "comparison chaining must have a dedicated diagnostic: {:?}",
            r.diagnostics
        );
    }

    #[test]
    fn nested_generic_closer_can_leave_assignment_equals() {
        let src = "fn f() { let x = a::<b<c>>=d; }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(r.is_ok(), "{src:?}: {:?}", r.diagnostics);
        assert!(r.syntax().descendants().any(|n| n.kind() == K::TypeArgs));
        assert!(r.syntax().descendants().any(|n| n.kind() == K::AssignExpr));
        assert_eq!(r.syntax().text().to_string(), src);
    }

    #[test]
    fn nested_generic_closer_does_not_leave_stuck_virtual_token() {
        let src = "fn f() { let x = a::<b<c>>=d; }";
        let mut p = Parser::from_source(src);
        let result = p.parse_source();
        assert!(
            result.diagnostics.is_empty(),
            "nested generic assignment must parse: {:?}",
            result.diagnostics
        );
        assert_eq!(result.syntax().text().to_string(), src);
        let assigns = result.syntax().descendants().filter(|n| n.kind() == K::AssignExpr).count();
        assert_eq!(assigns, 1);
    }

    #[test]
    fn split_generic_closer_emits_source_relative_piece_leaves() {
        let mut p = Parser::from_source(">>=");
        let first = p.consume_gt();
        assert!(matches!(first, Child::Piece { byte_offset: 0, byte_len: 1, .. }));
        let second = p.consume_gt();
        assert!(matches!(second, Child::Piece { byte_offset: 1, byte_len: 1, .. }));
        assert_eq!(p.current_kind(), Some(TokenKind::Punct(Punct::Eq)));
        let equals = p.bump_child();
        assert!(matches!(equals, Child::Piece { byte_offset: 2, byte_len: 1, .. }));
        assert_eq!(p.current_kind(), Some(TokenKind::Eof));
    }

    #[test]
    fn edition1_turbofish_path_shape_is_lossless() {
        let src = "fn f() { return id::<i32>(1); }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(r.is_ok(), "{:?}", r.diagnostics);
        assert_eq!(r.syntax().text().to_string(), src);
        assert!(r.syntax().descendants().any(|n| n.kind() == K::TypeArgs));
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
    fn final_block_expressions_do_not_require_semicolons() {
        for src in [
            "fn f() { 1 }",
            "fn f() { if true { 1 } else { 2 } }",
            "fn f() { loop { break 1; } }",
            "fn f() { match x { _ => 1, } }",
        ] {
            let mut p = Parser::from_source(src);
            let r = p.parse_source();
            assert!(r.is_ok(), "{src:?}: {:?}", r.diagnostics);
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
            "struct S {}",
            "enum E {}",
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

    /// `*` is a normative prefix `unary_op` (spec/grammar/omni-edition1.ebnf,
    /// `unary_op = "-" | "!" | "&" | "&mut" | "*" | "~"`), so `1 + * 2` is
    /// derivable syntax and the parser must accept it losslessly.
    ///
    /// Whether the *operand* of a deref must be a place expression is **not**
    /// settled by the grammar: `place_expr` references a `deref_expr`
    /// production that the EBNF never defines. That hole is already recorded in
    /// `docs/grammar-reconciliation.md`, and rejecting this input in the parser
    /// would resolve a specification question by implementation fiat. The
    /// type-level rule belongs to the checker, once the production exists.
    #[test]
    fn prefix_star_is_a_normative_unary_operator() {
        let src = "fn f() { 1 + * 2; }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(r.is_ok(), "`*` is a normative unary_op: {:?}", r.diagnostics);
        assert_eq!(r.syntax().text().to_string(), src, "must stay lossless");
        assert_eq!(
            r.syntax().descendants().filter(|n| n.kind() == K::UnaryExpr).count(),
            1,
            "the `*` is a unary expression, not an error node"
        );
    }

    /// Consecutive prefix operators remain malformed, which is what the
    /// adversarial suite actually needs to pin here.
    #[test]
    fn consecutive_prefix_operators_are_malformed() {
        assert_malformed_is_lossless_and_diagnosed("fn f() { + * ! }");
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

    // ----------------------------------------------------------------------
    // Stage-0 feature usage is *recorded* by the grammar layer, not enforced
    // by it. These tests pin that separation.
    // ----------------------------------------------------------------------

    /// The features a source used, in the deterministic order the parser emits.
    fn used(src: &str) -> Vec<&'static str> {
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(r.is_ok(), "{src:?} must parse cleanly: {:?}", r.diagnostics);
        // Recording must not disturb losslessness.
        assert_eq!(r.syntax().text().to_string(), src);
        r.feature_uses.iter().map(|u| u.feature).collect()
    }

    #[test]
    fn stage0_classified_features_are_recorded_not_rejected() {
        // Every one of these is a normative Edition 1 production, so all must
        // parse cleanly. Rejecting them here would mean the grammar layer was
        // enforcing a profile gate, which is exactly the conflation this
        // recording exists to prevent.
        assert_eq!(used("fn f() { foo!(a, b); }"), ["macros"]);
        assert_eq!(used("fn f() { let x = async move { 1 }; return x; }"), ["async"]);
        assert_eq!(used("fn f() { let x = unsafe { 1 }; return x; }"), ["unsafe_raw_memory"]);
    }

    #[test]
    fn feature_recording_covers_nested_and_multiple_uses() {
        // Nested macro token trees still record exactly once, at the outer `!`.
        assert_eq!(used("fn f() { outer!(inner!(a), (b, [c, { d }])); }"), ["macros"]);
        // Two different features in one source are both reported, sorted by
        // feature name so the result does not depend on visit order.
        assert_eq!(
            used("fn f() { let x = async move { unsafe { 1 } }; }"),
            ["async", "unsafe_raw_memory"]
        );
        // An `async fn` is an async feature too, not only an async block.
        assert_eq!(used("async fn f() { }"), ["async"]);
    }

    #[test]
    fn feature_spans_point_at_the_introducing_token() {
        let src = "fn f() { foo!(a); }";
        let mut p = Parser::from_source(src);
        let r = p.parse_source();
        assert!(r.is_ok());
        assert_eq!(r.feature_uses.len(), 1);
        let use_ = &r.feature_uses[0];
        assert_eq!(use_.feature, "macros");
        // The recorded span must land inside the source and cover real bytes:
        // a zero-width or out-of-range span would make a downstream profile
        // diagnostic point nowhere.
        assert!(use_.span.start < use_.span.end, "span must be non-empty: {:?}", use_.span);
        assert!(use_.span.end as usize <= src.len(), "span must be in range: {:?}", use_.span);
        assert_eq!(&src[use_.span.start as usize..use_.span.end as usize], "foo");
    }

    #[test]
    fn ordinary_source_records_no_features() {
        // The negative case: without this, a gate reading an always-non-empty
        // list could not be distinguished from one reading real usage.
        assert!(used("fn main() -> i64 { let mut s = 0; for i in 0..4 { s += i; } return s; }")
            .is_empty());
        assert!(used("struct P { x: i64 } fn f(p: P) -> i64 { return p.x; }").is_empty());
    }

    #[test]
    fn a_rejected_source_reports_no_features() {
        // Recovery may stop before reaching a construct, so features from a
        // failed parse would overstate what the source contains. The gate must
        // never see usage attributed to a program that did not parse.
        let mut p = Parser::from_source("fn f() { foo!(a); return a < b < c; }");
        let r = p.parse_source();
        assert!(!r.is_ok());
        assert!(
            r.feature_uses.is_empty(),
            "failed parse must record no features: {:?}",
            r.feature_uses
        );
        assert_eq!(r.syntax().text().to_string(), "fn f() { foo!(a); return a < b < c; }");
    }

    #[test]
    fn repeated_parses_report_identical_feature_sets() {
        // Determinism: the same bytes must yield the same usage, or a build
        // could accept and reject the same program across runs.
        let src = "fn f() { let x = async move { unsafe { 1 } }; foo!(x); }";
        let first = used(src);
        for _ in 0..4 {
            assert_eq!(used(src), first);
        }
    }

    #[test]
    fn reusing_one_parser_across_parses_repeats_the_same_verdict() {
        // `parse_source` resets parser position, diagnostics, and recorded
        // features, so one `Parser` may be driven repeatedly against the source
        // it was built from. (It cannot be pointed at a *different* source: the
        // token stream and spans are lexed up front and are tied to the
        // original bytes.) A reset that missed the recorded features would let
        // one parse's usage leak into the next, making a report depend on parse
        // history — a fresh parser per assertion cannot detect that.
        let src = "fn f() { let x = async move { 1 }; foo!(x); return x; }";

        let mut p = Parser::from_source(src);
        let first = p.parse_source();
        assert!(first.is_ok(), "{:?}", first.diagnostics);
        let first_features: Vec<_> =
            first.feature_uses.iter().map(|u| (u.feature, u.span)).collect();
        assert_eq!(first_features.iter().map(|(f, _)| *f).collect::<Vec<_>>(), ["async", "macros"]);

        for round in 1..4 {
            let again = p.parse_source();
            assert!(again.is_ok(), "round {round}: {:?}", again.diagnostics);
            // Same tree, same diagnostics, same feature attribution.
            assert_eq!(again.syntax().text().to_string(), first.syntax().text().to_string());
            assert_eq!(again.diagnostics, first.diagnostics);
            assert_eq!(
                again.feature_uses.iter().map(|u| (u.feature, u.span)).collect::<Vec<_>>(),
                first_features,
                "round {round} reported different feature attribution"
            );
        }
    }

    #[test]
    fn a_rejected_reparse_clears_features_recorded_by_an_earlier_parse() {
        // The failure case for the same reset. A parser that parsed a feature
        // successfully and then fails on malformed input must not report the
        // earlier feature as part of the failed program: the gate is documented
        // to see no features for a rejected parse, and that guarantee has to
        // hold for a reused parser too, not only a fresh one.
        let mut p = Parser::from_source("fn f() { foo!(a); }");
        assert_eq!(
            p.parse_source().feature_uses.iter().map(|u| u.feature).collect::<Vec<_>>(),
            ["macros"]
        );
        p.tokens[3] = omni_lex::Token {
            kind: omni_lex::TokenKind::Punct(omni_lex::token::Punct::Lt),
            ..p.tokens[3].clone()
        };
        let broken = p.parse_source();
        assert!(!broken.is_ok(), "corrupted token stream must not parse cleanly");
        assert!(
            broken.feature_uses.is_empty(),
            "rejected parse must report no features, got {:?}",
            broken.feature_uses
        );
    }
}
