//! The Concrete Syntax Tree (CST) and grammar definitions for Omni.
//!
//! # CST contracts (0.0.2.2)
//!
//! The tree produced through [`cst::OmniLanguage`] is lossless. See the
//! `omni_parse::parser` module documentation for the full contract; in brief:
//!
//! * Leaves exactly tile the source — no gap, no overlap, every byte covered once.
//! * Every lexer trivia region appears exactly once, with the same byte total.
//! * [`SyntaxKind::MissingToken`] is zero-width and never aliases a real token.
//! * [`SyntaxKind::ErrorNode`] is exclusively the parser's recovery tag (GRAM-0007).
//! * [`SyntaxKind::Unknown`] is reported for a raw kind with no counterpart here;
//!   it deliberately does **not** alias [`SyntaxKind::ErrorNode`], so tooling
//!   cannot read a malformed tree as a recovered one.
//! * Spans are byte offsets that land on UTF-8 character boundaries.
//!
//! [`SyntaxKind::ALL`] is indexed by raw discriminant, so the enum and `ALL`
//! must stay in lockstep; a test enforces this. Kinds are appended only —
//! inserting one would renumber the Rowan wire values.

pub mod ast;
pub mod cst;

pub use cst::{
    OmniLanguage, SyntaxElement, SyntaxElementChildren, SyntaxKind, SyntaxNode, SyntaxNodeChildren,
    SyntaxToken,
};
