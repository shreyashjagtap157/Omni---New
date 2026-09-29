use omni_unicode::classify::ProhibitedKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
    pub file_id: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kw {
    Never,
    SelfKw,
    Sized,
    Addrspace,
    As,
    Async,
    Await,
    BareMetal,
    Bf16,
    Bool,
    Break,
    Byte,
    Cap,
    Catch,
    Char,
    Const,
    Continue,
    Crate,
    Dec128,
    Dec32,
    Dec64,
    Defer,
    Distributed,
    Dyn,
    Effect,
    Else,
    Enum,
    Ensure,
    Extern,
    F128,
    F16,
    F32,
    F64,
    False,
    Fn,
    For,
    Hosted,
    I128,
    I16,
    I32,
    I64,
    I8,
    If,
    Impl,
    In,
    Is,
    Isolate,
    Isize,
    Let,
    Loop,
    Macro,
    Managed,
    Match,
    Mod,
    Module,
    Move,
    Mut,
    Not,
    Opaque,
    Override,
    Package,
    Panic,
    Persistent,
    Pub,
    Pure,
    Ref,
    Relation,
    Require,
    Return,
    Script,
    SelfRef,
    Static,
    Str,
    Struct,
    Super,
    ThreadLocal,
    Trait,
    True,
    Try,
    Type,
    Typeof,
    U128,
    U16,
    U32,
    U64,
    U8,
    Unsafe,
    Use,
    Usize,
    Verified,
    Where,
    While,
    Yield,
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
    /// The apostrophe punctuation used by Edition 1 lifetime and label syntax.
    /// Character literals are recognized by the lexer before this token is emitted.
    Apostrophe,
    DotDot,
    DotDotEq,
    Question,
    QuestionDot,
    QuestionQuestion,
    At,
    Hash,
    Dollar,
    Underscore,
    Shl,
    Shr,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AmpEq,
    PipeEq,
    CaretEq,
    ShlEq,
    ShrEq,
    Tilde,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Ident,
    Int,
    Float,
    Char,
    Byte,
    String,
    RawString,
    InterpolatedString,
    Keyword(Kw),
    Punct(Punct),
    Indent,
    Dedent,
    /// A lexical error.
    ///
    /// `Error` is deliberately uniform at the *token* level so that a consumer
    /// can always reconstruct the source, but [`Token::error_reason`] carries
    /// why the token failed. SRC-0005 in particular requires that a prohibited
    /// code point be reported as a source error and never silently replaced by
    /// U+FFFD, so the distinction has to survive to the diagnostic.
    Error,
    /// Terminal token emitted after the last real token.
    ///
    /// It carries the zero-width span at end of input plus any trivia that
    /// follows the final token, which is what lets consumers reconstruct the
    /// source byte-for-byte instead of dropping the tail of the file.
    Eof,
}

/// Why a token is [`TokenKind::Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorReason {
    /// The input was not well-formed UTF-8 at this position (SRC-0001).
    MalformedUtf8,
    /// A prohibited code point appeared outside comments and literals (SRC-0005).
    ProhibitedSource {
        /// The property class, as named by SRC-0005.
        kind: ProhibitedKind,
    },
    /// A comment contained an unannotated bidi control or invisible format
    /// character, which strict mode rejects (SRC-0006).
    UnannotatedCommentSecurity,
    /// Any other lexical failure: unknown punctuation, malformed literal,
    /// unterminated comment, and so on.
    Lexical,
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
    /// Why this token is an [`TokenKind::Error`], or `None` for a valid token.
    ///
    /// The lexer is the only producer, so a non-`Error` token always carries
    /// `None`; a consumer can therefore treat `Some(_)` as "this token is an
    /// error and here is the rule it violated".
    pub error_reason: Option<ErrorReason>,
}

impl Token {
    /// A valid token.
    pub fn new(
        kind: TokenKind,
        span: Span,
        leading_trivia: Vec<Trivia>,
        trailing_trivia: Vec<Trivia>,
    ) -> Self {
        Self { kind, span, leading_trivia, trailing_trivia, error_reason: None }
    }

    /// A token that failed lexical analysis for a stated reason.
    pub fn error(
        reason: ErrorReason,
        span: Span,
        leading_trivia: Vec<Trivia>,
        trailing_trivia: Vec<Trivia>,
    ) -> Self {
        Self {
            kind: TokenKind::Error,
            span,
            leading_trivia,
            trailing_trivia,
            error_reason: Some(reason),
        }
    }

    pub fn split_shift_right(self) -> Option<(Self, Self)> {
        if self.kind != TokenKind::Punct(Punct::Shr) || self.span.end != self.span.start + 2 {
            return None;
        }
        let first = Self {
            kind: TokenKind::Punct(Punct::Gt),
            span: Span {
                start: self.span.start,
                end: self.span.start + 1,
                file_id: self.span.file_id,
            },
            leading_trivia: self.leading_trivia,
            trailing_trivia: Vec::new(),
            error_reason: None,
        };
        let second = Self {
            kind: TokenKind::Punct(Punct::Gt),
            span: Span {
                start: self.span.start + 1,
                end: self.span.end,
                file_id: self.span.file_id,
            },
            leading_trivia: Vec::new(),
            trailing_trivia: self.trailing_trivia,
            error_reason: None,
        };
        Some((first, second))
    }
}

// Trivia preservation (lossless CST) behavior lives here. No `implements` tag is
// claimed: LEX-0002 is Superseded under the Candidate-2 amendment, so the prior
// ownership tag was stale linkage and was removed by the reaper. Re-ownership
// awaits a surviving normative rule ID from the specification layer.
