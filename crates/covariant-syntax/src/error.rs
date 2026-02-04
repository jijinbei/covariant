use crate::span::Span;

/// A syntax error (lexer or parser).
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub message: String,
    pub span: Span,
    pub kind: ErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Lexer: unexpected character.
    UnexpectedChar,
    /// Lexer: unterminated string literal.
    UnterminatedString,
    /// Lexer: unterminated block comment.
    UnterminatedBlockComment,
    /// Lexer: invalid number literal.
    InvalidNumber,
    /// Lexer: unknown unit suffix.
    UnknownUnit,
    /// Parser: expected a specific token.
    ExpectedToken,
    /// Parser: expected an expression.
    ExpectedExpr,
    /// Parser: expected a statement.
    ExpectedStmt,
    /// Parser: unexpected end of file.
    UnexpectedEof,
}

impl SyntaxError {
    pub fn new(message: impl Into<String>, span: Span, kind: ErrorKind) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
        }
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyntaxError {}

/// Convenience type alias.
pub type SyntaxResult<T> = Result<T, SyntaxError>;
