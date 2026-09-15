use crate::cst::{SyntaxKind, SyntaxNode};

pub trait AstNode {
    fn cast(node: SyntaxNode) -> Option<Self> where Self: Sized;
    fn syntax(&self) -> &SyntaxNode;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(SyntaxNode);
impl AstNode for Name {
    fn cast(node: SyntaxNode) -> Option<Self> {
        (node.kind() == SyntaxKind::NameRef || node.kind() == SyntaxKind::Ident).then_some(Self(node))
    }
    fn syntax(&self) -> &SyntaxNode { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDef(SyntaxNode);
impl AstNode for FnDef {
    fn cast(node: SyntaxNode) -> Option<Self> { (node.kind() == SyntaxKind::FnDef).then_some(Self(node)) }
    fn syntax(&self) -> &SyntaxNode { &self.0 }
}
impl FnDef {
    pub fn name(&self) -> Option<Name> { self.syntax().children().find_map(Name::cast) }
}
