//! Lexer, parser, and AST definitions for the COVARIANT language.

pub mod error;
pub mod span;

pub use error::{ErrorKind, SyntaxError, SyntaxResult};
pub use span::{Span, Spanned};
