use crate::cst::{SyntaxKind, SyntaxNode};

pub trait AstNode {
    fn cast(node: SyntaxNode) -> Option<Self>
    where
        Self: Sized;
    fn syntax(&self) -> &SyntaxNode;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(SyntaxNode);
impl AstNode for Name {
    fn cast(node: SyntaxNode) -> Option<Self> {
        (node.kind() == SyntaxKind::NameRef || node.kind() == SyntaxKind::Ident)
            .then_some(Self(node))
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDef(SyntaxNode);
impl AstNode for FnDef {
    fn cast(node: SyntaxNode) -> Option<Self> {
        (node.kind() == SyntaxKind::FnDef).then_some(Self(node))
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}
impl FnDef {
    pub fn name(&self) -> Option<Name> {
        self.syntax().children().find_map(Name::cast)
    }
}

macro_rules! define_ast_nodes {
    ($(($name:ident, $kind:path)),* $(,)?) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct $name(SyntaxNode);
            impl AstNode for $name {
                fn cast(node: SyntaxNode) -> Option<Self> {
                    (node.kind() == $kind).then_some(Self(node))
                }
                fn syntax(&self) -> &SyntaxNode {
                    &self.0
                }
            }
        )*
    };
}

define_ast_nodes!(
    (StructDef, SyntaxKind::StructDef),
    (EnumDef, SyntaxKind::EnumDef),
    (TraitDef, SyntaxKind::TraitDef),
    (ImplDef, SyntaxKind::ImplDef),
    (TypeAlias, SyntaxKind::TypeAlias),
    (ConstDef, SyntaxKind::ConstDef),
    (StaticDef, SyntaxKind::StaticDef),
    (UseDecl, SyntaxKind::UseDecl),
    (ModuleDecl, SyntaxKind::ModuleDecl),
    (ExternCrateDecl, SyntaxKind::ExternCrateDecl),
    (Attribute, SyntaxKind::Attribute),
    (GenericParams, SyntaxKind::GenericParams),
    (GenericParam, SyntaxKind::GenericParam),
    (Lifetime, SyntaxKind::Lifetime),
    (Path, SyntaxKind::Path),
    (PathSegment, SyntaxKind::PathSegment),
    (Type, SyntaxKind::Type),
    (TupleType, SyntaxKind::TupleType),
    (ArrayType, SyntaxKind::ArrayType),
    (ReferenceType, SyntaxKind::ReferenceType),
    (RawPointerType, SyntaxKind::RawPointerType),
    (FunctionType, SyntaxKind::FunctionType),
    (IfExpr, SyntaxKind::IfExpr),
    (MatchExpr, SyntaxKind::MatchExpr),
    (MatchArm, SyntaxKind::MatchArm),
    (LoopExpr, SyntaxKind::LoopExpr),
    (WhileExpr, SyntaxKind::WhileExpr),
    (ForExpr, SyntaxKind::ForExpr),
    (TryExpr, SyntaxKind::TryExpr),
    (AwaitExpr, SyntaxKind::AwaitExpr),
    (AssignExpr, SyntaxKind::AssignExpr),
    (MethodCallExpr, SyntaxKind::MethodCallExpr),
    (FieldExpr, SyntaxKind::FieldExpr),
    (IndexExpr, SyntaxKind::IndexExpr),
    (RangeExpr, SyntaxKind::RangeExpr),
    (CastExpr, SyntaxKind::CastExpr),
    (PathExpr, SyntaxKind::PathExpr),
    (MacroInvocation, SyntaxKind::MacroInvocation),
    (TokenTree, SyntaxKind::TokenTree),
    (StructExpr, SyntaxKind::StructExpr),
    (ArrayExpr, SyntaxKind::ArrayExpr),
    (TupleExpr, SyntaxKind::TupleExpr),
    (ClosureExpr, SyntaxKind::ClosureExpr),
    (AsyncBlock, SyntaxKind::AsyncBlock),
    (UnsafeBlock, SyntaxKind::UnsafeBlock),
    (TryBlock, SyntaxKind::TryBlock),
    (ItemDeclStmt, SyntaxKind::ItemDeclStmt),
);

impl StructDef {
    pub fn name(&self) -> Option<Name> {
        self.syntax().children().find_map(Name::cast)
    }
}

impl EnumDef {
    pub fn variants(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.syntax().children().filter(|n| n.kind() == SyntaxKind::EnumVariant)
    }
}

impl FnDef {
    pub fn params(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.syntax()
            .children()
            .find(|n| n.kind() == SyntaxKind::ParamList)
            .into_iter()
            .flat_map(|n| n.children().filter(|c| c.kind() == SyntaxKind::Param))
    }
}
