//! Parser crate for Omni.
pub mod expr;
#[cfg(test)]
mod grammar_contract;
pub mod parser;
pub mod precedence;
pub use parser::{Diagnostic, FeatureUse, ParseResult, Parser};
