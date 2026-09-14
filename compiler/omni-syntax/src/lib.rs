//! The Concrete Syntax Tree (CST) and grammar definitions for Omni.

pub mod ast;
pub mod cst;

pub use cst::{
    OmniLanguage, SyntaxElement, SyntaxElementChildren, SyntaxKind, SyntaxNode,
    SyntaxNodeChildren, SyntaxToken,
};
