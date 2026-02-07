//! Evaluation errors with source location tracking.

use covariant_syntax::Span;
use std::fmt;

/// The result type for evaluation operations.
pub type EvalResult<T> = Result<T, EvalError>;

/// An evaluation error with source location and kind.
#[derive(Debug, Clone)]
pub struct EvalError {
    pub message: String,
    pub span: Option<Span>,
    pub kind: EvalErrorKind,
}

impl EvalError {
    pub fn new(kind: EvalErrorKind, message: impl Into<String>, span: Option<Span>) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.span {
            Some(span) => write!(f, "[{:?}] {}: {}", span, self.kind, self.message),
            None => write!(f, "{}: {}", self.kind, self.message),
        }
    }
}

impl std::error::Error for EvalError {}

/// Categories of evaluation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalErrorKind {
    /// Type mismatch at runtime.
    TypeError,
    /// Reference to an undefined name.
    UndefinedName,
    /// Wrong number of arguments.
    ArityMismatch,
    /// Field not found on a data value.
    FieldNotFound,
    /// Division by zero.
    DivisionByZero,
    /// Error from the geometry kernel.
    GeomError,
    /// Attempted to call a non-callable value.
    NotCallable,
    /// No pattern matched in a match expression.
    PatternMismatch,
    /// General-purpose error.
    Custom,
}

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::TypeError => "TypeError",
            Self::UndefinedName => "UndefinedName",
            Self::ArityMismatch => "ArityMismatch",
            Self::FieldNotFound => "FieldNotFound",
            Self::DivisionByZero => "DivisionByZero",
            Self::GeomError => "GeomError",
            Self::NotCallable => "NotCallable",
            Self::PatternMismatch => "PatternMismatch",
            Self::Custom => "Error",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_construction() {
        let err = EvalError::new(
            EvalErrorKind::TypeError,
            "expected Int, got Float",
            Some(Span::new(0, 5)),
        );
        assert_eq!(err.kind, EvalErrorKind::TypeError);
        assert_eq!(err.message, "expected Int, got Float");
        assert_eq!(err.span, Some(Span::new(0, 5)));
    }

    #[test]
    fn error_display_with_span() {
        let err = EvalError::new(
            EvalErrorKind::UndefinedName,
            "unknown variable 'x'",
            Some(Span::new(10, 11)),
        );
        let s = format!("{err}");
        assert!(s.contains("UndefinedName"));
        assert!(s.contains("unknown variable 'x'"));
    }

    #[test]
    fn error_display_without_span() {
        let err = EvalError::new(EvalErrorKind::DivisionByZero, "division by zero", None);
        let s = format!("{err}");
        assert!(s.contains("DivisionByZero"));
        assert!(s.contains("division by zero"));
    }

    #[test]
    fn error_kind_display() {
        assert_eq!(format!("{}", EvalErrorKind::TypeError), "TypeError");
        assert_eq!(format!("{}", EvalErrorKind::GeomError), "GeomError");
        assert_eq!(format!("{}", EvalErrorKind::Custom), "Error");
    }
}
