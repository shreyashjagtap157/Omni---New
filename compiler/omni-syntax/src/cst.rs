use rowan::Language;

/// All grammatical nodes and tokens for the Omni language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum SyntaxKind {
    // --- Lexical Tokens (from omni-lex) ---
    Ident = 0,
    Int,
    Float,
    Keyword,
    Punct,
    Indent,
    Dedent,

    // --- Trivia ---
    Whitespace,
    LineComment,
    BlockComment,
    DocComment,

    ErrorToken, // Lexical error

    // --- Parser Nodes (to be expanded in grammar) ---
    SourceFile,
    ErrorNode, // Syntax error node
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

// Convert Rowan's generic u16 back to our typed enum.
// Completely safe, no 	ransmute required.
impl From<rowan::SyntaxKind> for SyntaxKind {
    fn from(kind: rowan::SyntaxKind) -> Self {
        match kind.0 {
            0 => SyntaxKind::Ident,
            1 => SyntaxKind::Int,
            2 => SyntaxKind::Float,
            3 => SyntaxKind::Keyword,
            4 => SyntaxKind::Punct,
            5 => SyntaxKind::Indent,
            6 => SyntaxKind::Dedent,
            7 => SyntaxKind::Whitespace,
            8 => SyntaxKind::LineComment,
            9 => SyntaxKind::BlockComment,
            10 => SyntaxKind::DocComment,
            11 => SyntaxKind::ErrorToken,
            12 => SyntaxKind::SourceFile,
            13 => SyntaxKind::ErrorNode,
            _ => SyntaxKind::ErrorNode,
        }
    }
}

/// The Rowan Language definition for Omni.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OmniLanguage;

impl Language for OmniLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        Self::Kind::from(raw)
    }

    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind::from(kind)
    }
}

// Type aliases for the green/red tree nodes exposed by Rowan
pub type SyntaxNode = rowan::SyntaxNode<OmniLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<OmniLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<OmniLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<OmniLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<OmniLanguage>;

// Semantic linkage declaration for the syntax definition.
// GRAM-0001 (Syntax Nodes) will be defined and validated as we build the parser.
#[cfg(any())]
#[implements("GRAM-0001")]
fn _audit_syntax_kinds() {}
