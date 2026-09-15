//! Parser crate for Omni.
pub mod expr;
pub mod parser;
pub use parser::{Diagnostic, ParseResult, Parser};
