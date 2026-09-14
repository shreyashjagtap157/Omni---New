use crate::cst::SyntaxNode;

/// Base trait for all strongly-typed AST nodes wrapping raw Rowan syntax nodes.
pub trait AstNode {
    fn cast(node: SyntaxNode) -> Option<Self>
    where
        Self: Sized;
    fn syntax(&self) -> &SyntaxNode;
}

/// A name node (e.g., an identifier used in a declaration).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(SyntaxNode);

impl AstNode for Name {
    fn cast(node: SyntaxNode) -> Option<Self> {
        // In the fully expanded grammar, this would check for SyntaxKind::Name
        Some(Self(node))
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// A function definition node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDef(SyntaxNode);

impl AstNode for FnDef {
    fn cast(node: SyntaxNode) -> Option<Self> {
        Some(Self(node))
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

impl FnDef {
    /// Extracts the name of the function definition from its children.
    pub fn name(&self) -> Option<Name> {
        self.syntax().children().find_map(Name::cast)
    }
}
