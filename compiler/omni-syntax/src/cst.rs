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
    /// A zero-width placeholder for a token the grammar required but that the
    /// input did not supply.
    ///
    /// This exists so recovery never has to fabricate source text. A missing
    /// token carries no bytes, so it can be inserted anywhere without affecting
    /// the reconstruction invariant, and it is distinguishable from both a real
    /// token and from [`SyntaxKind::ErrorNode`] recovery.
    MissingToken,
    /// A raw Rowan kind with no counterpart in this enum.
    ///
    /// Rowan's reverse conversion is total, so an unrecognized raw kind has to
    /// map somewhere. It must **not** map to [`SyntaxKind::ErrorNode`]: that kind
    /// is the parser's recovery tag (GRAM-0007), and aliasing an unknown kind
    /// onto it would let tooling read a malformed tree as a recovered one.
    Unknown,
}

impl SyntaxKind {
    /// Every kind in declaration order.
    ///
    /// `#[repr(u16)]` discriminants are the Rowan wire values, so the ordering
    /// is load-bearing and must not be reshuffled. Appending is safe; inserting
    /// is not.
    pub const ALL: &'static [SyntaxKind] = &[
        SyntaxKind::Ident,
        SyntaxKind::Int,
        SyntaxKind::Float,
        SyntaxKind::Keyword,
        SyntaxKind::Punct,
        SyntaxKind::Indent,
        SyntaxKind::Dedent,
        SyntaxKind::Whitespace,
        SyntaxKind::LineComment,
        SyntaxKind::BlockComment,
        SyntaxKind::DocComment,
        SyntaxKind::ErrorToken,
        SyntaxKind::SourceFile,
        SyntaxKind::FnDef,
        SyntaxKind::ParamList,
        SyntaxKind::Param,
        SyntaxKind::Type,
        SyntaxKind::Block,
        SyntaxKind::LetStmt,
        SyntaxKind::ExprStmt,
        SyntaxKind::ReturnExpr,
        SyntaxKind::CallExpr,
        SyntaxKind::BinaryExpr,
        SyntaxKind::UnaryExpr,
        SyntaxKind::LiteralExpr,
        SyntaxKind::NameRef,
        SyntaxKind::ErrorNode,
        SyntaxKind::MissingToken,
        SyntaxKind::Unknown,
    ];

    /// The raw Rowan discriminant for this kind.
    pub fn to_raw(self) -> rowan::SyntaxKind {
        rowan::SyntaxKind(self as u16)
    }
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        kind.to_raw()
    }
}

impl From<rowan::SyntaxKind> for SyntaxKind {
    fn from(kind: rowan::SyntaxKind) -> Self {
        SyntaxKind::ALL
            .get(kind.0 as usize)
            .copied()
            // An unknown raw kind is reported as `Unknown`, never as
            // `ErrorNode`: `ErrorNode` is the parser's recovery tag, and
            // conflating the two would make a tree look recovered when it is
            // not.
            .unwrap_or(SyntaxKind::Unknown)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every kind must round-trip through its raw discriminant unchanged.
    ///
    /// `ALL` is indexed by raw value in the reverse conversion, so a mismatch
    /// between the enum's declaration order and `ALL` would silently reinterpret
    /// kinds -- e.g. reading a `Block` as a `LetStmt`.
    #[test]
    fn every_kind_round_trips_through_its_raw_discriminant() {
        for (i, kind) in SyntaxKind::ALL.iter().enumerate() {
            assert_eq!(i, *kind as usize, "{kind:?} is at the wrong index in ALL");
            let raw = kind.to_raw();
            assert_eq!(i as u16, raw.0, "{kind:?} has the wrong raw discriminant");
            assert_eq!(*kind, SyntaxKind::from(raw), "{kind:?} did not round-trip");
            assert_eq!(*kind, OmniLanguage::kind_from_raw(raw));
            assert_eq!(raw, OmniLanguage::kind_to_raw(*kind));
        }
    }

    /// The enum and `ALL` must stay in lockstep; a kind added to one and not the
    /// other is exactly how a kind starts aliasing its neighbour.
    #[test]
    fn all_lists_every_declared_kind_exactly_once() {
        let mut seen = vec![false; SyntaxKind::ALL.len()];
        for kind in SyntaxKind::ALL {
            let idx = *kind as usize;
            assert!(idx < seen.len(), "{kind:?} has no slot in ALL");
            assert!(!seen[idx], "{kind:?} appears twice in ALL");
            seen[idx] = true;
        }
        assert!(seen.iter().all(|s| *s), "ALL has a hole: {seen:?}");
    }

    /// An unrecognized raw kind must not masquerade as a recovery node.
    ///
    /// `ErrorNode` is the parser's recovery tag (GRAM-0007); mapping an unknown
    /// kind onto it would make tooling read a malformed tree as a recovered one.
    #[test]
    fn an_unknown_raw_kind_is_not_an_error_node() {
        // `ErrorNode` is the last of the pre-existing kinds; 27/28 are the
        // appended kinds, and anything at or beyond `ALL.len()` is genuinely
        // unrecognized.
        for raw in [SyntaxKind::MissingToken as u16 + 1, 99, u16::MAX] {
            let kind = SyntaxKind::from(rowan::SyntaxKind(raw));
            assert_eq!(kind, SyntaxKind::Unknown, "raw {raw} should be Unknown");
            assert_ne!(kind, SyntaxKind::ErrorNode);
        }
        assert_ne!(SyntaxKind::MissingToken, SyntaxKind::ErrorNode);
        assert_ne!(SyntaxKind::Unknown, SyntaxKind::ErrorNode);
    }
}
