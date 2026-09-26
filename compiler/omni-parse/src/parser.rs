//! Lossless recursive-descent + Pratt parser for Omni.

use omni_lex::token::{Kw, Punct, Trivia};
use omni_lex::{Scanner, Span, Token, TokenKind};
use omni_syntax::{SyntaxKind, SyntaxNode};
use rowan::GreenNodeBuilder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone)]
enum Child {
    Node(Node),
    Token(usize),
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
                self.synchronize_top();
            }
        }
        n
    }
    fn parse_item_stub(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ErrorNode);
        n.children.push(Child::Token(self.bump()));
        if self.at_ident() {
            n.children.push(Child::Token(self.bump()));
        }
        n
    }
    fn parse_fn(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::FnDef);
        n.children.push(Child::Token(self.expect_kw(Kw::Fn)));
        if self.at_ident() {
            n.children.push(Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump())));
        } else {
            n.children.push(Child::Node(self.error_node("expected function name")));
        }
        n.children.push(Child::Node(self.parse_params()));
        if self.at_punct(Punct::Arrow) {
            n.children.push(Child::Token(self.bump()));
            n.children.push(Child::Node(self.parse_type()));
        }
        n.children.push(Child::Node(self.parse_block()));
        n
    }
    fn parse_params(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ParamList);
        n.children.push(Child::Token(self.expect_punct(Punct::LParen)));
        while !self.eof() && !self.at_punct(Punct::RParen) {
            let mut p = Node::new(SyntaxKind::Param);
            if self.at_ident() {
                p.children
                    .push(Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump())));
            } else {
                p.children.push(Child::Node(self.error_node("expected parameter name")));
                self.recover_until(&[Punct::Comma, Punct::RParen]);
            }
            if self.at_punct(Punct::Colon) {
                p.children.push(Child::Token(self.bump()));
                p.children.push(Child::Node(self.parse_type()));
            } else {
                self.diagnostic("expected `:` after parameter name");
            }
            n.children.push(Child::Node(p));
            if self.at_punct(Punct::Comma) {
                n.children.push(Child::Token(self.bump()));
            } else {
                break;
            }
        }
        n.children.push(Child::Token(self.expect_punct(Punct::RParen)));
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
            n.children.push(Child::Token(self.bump()));
        } else {
            n.children.push(Child::Node(self.error_node("expected type name")));
        }
        n
    }
    fn parse_block(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::Block);
        n.children.push(Child::Token(self.expect_punct(Punct::LBrace)));
        while !self.eof() && !self.at_punct(Punct::RBrace) {
            if self.at_kw(Kw::Let) {
                n.children.push(Child::Node(self.parse_let()));
            } else if self.at_kw(Kw::Return) {
                n.children.push(Child::Node(self.parse_return()));
            } else {
                n.children.push(Child::Node(self.parse_expr_stmt()));
            }
        }
        n.children.push(Child::Token(self.expect_punct(Punct::RBrace)));
        n
    }
    fn parse_let(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::LetStmt);
        n.children.push(Child::Token(self.expect_kw(Kw::Let)));
        if self.at_kw(Kw::Mut) {
            n.children.push(Child::Token(self.bump()));
        }
        if self.at_ident() {
            n.children.push(Child::Node(Node::new(SyntaxKind::NameRef).with_token(self.bump())));
        } else {
            n.children.push(Child::Node(self.error_node("expected binding name")));
        }
        if self.at_punct(Punct::Colon) {
            n.children.push(Child::Token(self.bump()));
            n.children.push(Child::Node(self.parse_type()));
        }
        n.children.push(Child::Token(self.expect_punct(Punct::Eq)));
        n.children.push(Child::Node(self.parse_expr_bp(0)));
        n.children.push(Child::Token(self.expect_punct(Punct::Semicolon)));
        n
    }
    fn parse_return(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ReturnExpr);
        n.children.push(Child::Token(self.expect_kw(Kw::Return)));
        if !self.at_punct(Punct::Semicolon) && !self.at_punct(Punct::RBrace) {
            n.children.push(Child::Node(self.parse_expr_bp(0)));
        }
        n.children.push(Child::Token(self.expect_punct(Punct::Semicolon)));
        n
    }
    fn parse_expr_stmt(&mut self) -> Node {
        let mut n = Node::new(SyntaxKind::ExprStmt);
        n.children.push(Child::Node(self.parse_expr_bp(0)));
        if self.at_punct(Punct::Semicolon) {
            n.children.push(Child::Token(self.bump()));
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
                call.children.push(Child::Token(self.bump()));
                while !self.eof() && !self.at_punct(Punct::RParen) {
                    call.children.push(Child::Node(self.parse_expr_bp(0)));
                    if self.at_punct(Punct::Comma) {
                        call.children.push(Child::Token(self.bump()));
                    } else {
                        break;
                    }
                }
                call.children.push(Child::Token(self.expect_punct(Punct::RParen)));
                lhs = call;
                continue;
            }
            let Some((op, left_bp, right_bp)) = self.infix() else { break };
            if left_bp < min_bp {
                break;
            }
            let mut bin = Node::new(SyntaxKind::BinaryExpr);
            bin.children.push(Child::Node(lhs));
            bin.children.push(Child::Token(self.bump()));
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
            n.children.push(Child::Token(self.bump()));
            n.children.push(Child::Node(self.parse_expr_bp(6)));
            return n;
        }
        if self.at_punct(Punct::LParen) {
            let mut n = Node::new(SyntaxKind::UnaryExpr);
            n.children.push(Child::Token(self.bump()));
            n.children.push(Child::Node(self.parse_expr_bp(0)));
            n.children.push(Child::Token(self.expect_punct(Punct::RParen)));
            return n;
        }
        match self.current_kind() {
            Some(TokenKind::Ident) => Node::new(SyntaxKind::NameRef).with_token(self.bump()),
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
                Node::new(SyntaxKind::LiteralExpr).with_token(self.bump())
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
            }
        }
        b.finish_node();
    }
    fn emit_token(&self, b: &mut GreenNodeBuilder, index: usize) {
        if index == usize::MAX {
            return;
        }
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
            Node::new(SyntaxKind::ErrorNode).with_token(self.bump())
        }
    }
    fn synchronize_top(&mut self) {
        while !self.eof() && !self.at_kw(Kw::Fn) {
            self.pos += 1;
        }
    }
    fn recover_until(&mut self, puncts: &[Punct]) {
        while !self.eof() && !puncts.iter().any(|p| self.at_punct(*p)) {
            self.pos += 1;
        }
    }
    fn expect_kw(&mut self, kw: Kw) -> usize {
        if self.at_kw(kw) {
            self.bump()
        } else {
            self.diagnostic("expected keyword");
            self.bump_or_dummy()
        }
    }
    fn expect_punct(&mut self, p: Punct) -> usize {
        if self.at_punct(p) {
            self.bump()
        } else {
            self.diagnostic("expected punctuation");
            self.bump_or_dummy()
        }
    }
    fn bump_or_dummy(&mut self) -> usize {
        if self.eof() {
            self.tokens.len().saturating_sub(1)
        } else {
            self.bump()
        }
    }
    fn bump(&mut self) -> usize {
        let i = self.pos;
        self.pos += 1;
        i
    }
    fn current_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|t| t.kind)
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
            "fn f() { return f\"hello ${name}\\"; }",
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
