use rowan::Language;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    Ident = 0,
    Int,
    Float,
    Keyword,
    Punct,
    Indent,
    Dedent,
    Whitespace,
    LineComment,
    BlockComment,
    DocComment,
    ErrorToken,
    SourceFile,
    FnDef,
    ParamList,
    Param,
    Type,
    Block,
    LetStmt,
    ExprStmt,
    ReturnExpr,
    CallExpr,
    BinaryExpr,
    UnaryExpr,
    LiteralExpr,
    NameRef,
    ErrorNode,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

impl From<rowan::SyntaxKind> for SyntaxKind {
    fn from(kind: rowan::SyntaxKind) -> Self {
        match kind.0 {
            0 => Self::Ident,
            1 => Self::Int,
            2 => Self::Float,
            3 => Self::Keyword,
            4 => Self::Punct,
            5 => Self::Indent,
            6 => Self::Dedent,
            7 => Self::Whitespace,
            8 => Self::LineComment,
            9 => Self::BlockComment,
            10 => Self::DocComment,
            11 => Self::ErrorToken,
            12 => Self::SourceFile,
            13 => Self::FnDef,
            14 => Self::ParamList,
            15 => Self::Param,
            16 => Self::Type,
            17 => Self::Block,
            18 => Self::LetStmt,
            19 => Self::ExprStmt,
            20 => Self::ReturnExpr,
            21 => Self::CallExpr,
            22 => Self::BinaryExpr,
            23 => Self::UnaryExpr,
            24 => Self::LiteralExpr,
            25 => Self::NameRef,
            _ => Self::ErrorNode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OmniLanguage;
impl Language for OmniLanguage {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        raw.into()
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<OmniLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<OmniLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<OmniLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<OmniLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<OmniLanguage>;

#[cfg(any())]
#[implements("GRAM-0001")]
fn _audit_syntax_kinds() {}
