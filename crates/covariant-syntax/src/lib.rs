//! Lexer, parser, and AST definitions for the COVARIANT language.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

pub use ast::SourceFile;
pub use error::{ErrorKind, SyntaxError, SyntaxResult};
pub use span::{Span, Spanned};
pub use token::{SyntaxKind, Token};

/// Convenience: lex and parse source code in one step.
pub fn parse(source: &str) -> (SourceFile, Vec<SyntaxError>) {
    let (tokens, mut lex_errors) = lexer::lex(source);
    let (ast, parse_errors) = parser::parse(source, tokens);
    lex_errors.extend(parse_errors);
    (ast, lex_errors)
}
