//! Foundation scaffold for `omni-lex`.

pub mod layout;
pub mod scanner;
pub mod token;

#[cfg(test)]
mod security_tests;
#[cfg(test)]
mod tests;

pub use layout::{LayoutEngine, LayoutError};
pub use omni_unicode::annotation::SecurityMode;
pub use scanner::Scanner;
pub use token::{ErrorReason, Kw, Punct, Span, Token, TokenKind, Trivia, TriviaKind};
