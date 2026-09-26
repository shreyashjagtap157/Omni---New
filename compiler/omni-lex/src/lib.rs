//! Foundation scaffold for `omni-lex`.

pub mod layout;
pub mod scanner;
pub mod token;

#[cfg(test)]
mod tests;

pub use layout::{LayoutEngine, LayoutError};
pub use scanner::Scanner;
pub use token::{Kw, Punct, Span, Token, TokenKind, Trivia, TriviaKind};
