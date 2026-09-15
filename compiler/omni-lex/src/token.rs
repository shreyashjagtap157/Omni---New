#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub file_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kw {
    Fn,
    Let,
    Mut,
    If,
    Else,
    Return,
    Match,
    Struct,
    Enum,
    True,
    False,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punct {
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Semicolon,
    Arrow,
    Amp,
    Pipe,
    Bang,
    Dot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    Keyword(Kw),
    Punct(Punct),
    Indent,
    Dedent,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaKind {
    Whitespace,
    LineComment,
    BlockComment,
    DocComment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub leading_trivia: Vec<Trivia>,
    pub trailing_trivia: Vec<Trivia>,
}

// Semantic Linkage Tracker
// `omni-audit` parses the AST and finds this tag.
// `cfg(any())` evaluates to false, stripping it from `rustc` to prevent "unknown attribute" errors.
#[cfg(any())]
#[implements("LEX-0002")]
fn _audit_trivia_preservation() {}
