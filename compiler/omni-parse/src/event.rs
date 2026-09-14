use omni_syntax::SyntaxKind;

/// Emitted by the parser to instruct the tree builder how to construct the CST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Start a new syntax node of the given kind.
    /// orward_parent allows the parser to retroactively wrap a node inside another (crucial for left-recursive expressions).
    StartNode {
        kind: SyntaxKind,
        forward_parent: Option<usize>,
    },
    /// Consume the next token from the lexer stream and attach it to the current node.
    AddToken,
    /// Finish building the current syntax node.
    FinishNode,
    /// Emit a syntax error without crashing the parser.
    Error(String),
}
