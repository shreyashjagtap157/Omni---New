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
    Pub,
    Use,
    Mod,
    Trait,
    Impl,
    Type,
    Where,
    For,
    In,
    While,
    Loop,
    Break,
    Continue,
    Async,
    Await,
    Try,
    Catch,
    As,
    With,
    Parallel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punct {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
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
    ColonColon,
    Semicolon,
    Arrow,
    FatArrow,
    Amp,
    AmpAmp,
    Pipe,
    PipePipe,
    PipeArrow,
    Bang,
    Dot,
    DotDot,
    DotDotEq,
    Question,
    QuestionDot,
    QuestionQuestion,
    At,
    Hash,
    Dollar,
    Underscore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    Char,
    String,
    RawString,
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

// Trivia preservation (lossless CST) behavior lives here. No `implements` tag is
// claimed: LEX-0002 is Superseded under the Candidate-2 amendment, so the prior
// ownership tag was stale linkage and was removed by the reaper. Re-ownership
// awaits a surviving normative rule ID from the specification layer.
