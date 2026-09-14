//! Foundation scaffold for `omni-lex`.

pub mod layout;
pub mod scanner;
pub mod token;

pub use layout::{LayoutEngine, LayoutError};
pub use scanner::Scanner;
pub use token::{Span, Token, TokenKind, Trivia, TriviaKind};