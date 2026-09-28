//! Parser crate for Omni.
pub mod expr;
#[cfg(test)]
mod grammar_contract;
pub mod parser;
pub use parser::{Diagnostic, ParseResult, Parser};
