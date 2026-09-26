use omni_lex::token::Punct;
use omni_lex::TokenKind;
use omni_syntax::SyntaxKind;

pub trait ExprParser {
    fn peek(&self) -> Option<TokenKind>;
    fn advance(&mut self);
    fn start_node(&mut self, kind: SyntaxKind) -> usize;
    fn finish_node(&mut self);
    fn error(&mut self, msg: String);
}
pub fn binding_power(kind: TokenKind) -> (u8, u8) {
    match kind {
        TokenKind::Punct(Punct::Eq) => (1, 1),
        TokenKind::Punct(Punct::Pipe) => (2, 3),
        TokenKind::Punct(Punct::EqEq | Punct::NotEq) => (3, 4),
        TokenKind::Punct(Punct::Lt | Punct::Le | Punct::Gt | Punct::Ge) => (4, 5),
        TokenKind::Punct(Punct::Plus | Punct::Minus) => (5, 6),
        TokenKind::Punct(Punct::Star | Punct::Slash) => (7, 8),
        _ => (0, 0),
    }
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
        TokenKind::Punct(Punct::Minus | Punct::Bang | Punct::Amp) => SyntaxKind::UnaryExpr,
        _ => SyntaxKind::ErrorNode,
    });
    let _ = mark;
    p.advance();
    p.finish_node();
    while let Some(op) = p.peek() {
        let (l, r) = binding_power(op);
        if l < min_bp || l == 0 {
            break;
        };
        p.advance();
        parse_expr_bp(p, r);
    }
}
