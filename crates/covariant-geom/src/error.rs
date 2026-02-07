//! Error types for the geometry crate.

use std::fmt;

/// The kind of geometry error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeomErrorKind {
    /// A boolean operation failed (degenerate input, tolerance issue, etc.).
    BooleanFailed,
    /// Tessellation failed to produce a valid mesh.
    TessellationFailed,
    /// The caller provided invalid parameters (zero radius, empty list, etc.).
    InvalidInput,
    /// An I/O error occurred during export.
    IoError,
}

impl fmt::Display for GeomErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BooleanFailed => write!(f, "boolean operation failed"),
            Self::TessellationFailed => write!(f, "tessellation failed"),
            Self::InvalidInput => write!(f, "invalid input"),
            Self::IoError => write!(f, "I/O error"),
        }
    }
}

/// A geometry error with a kind and a human-readable message.
#[derive(Debug)]
pub struct GeomError {
    pub kind: GeomErrorKind,
    pub message: String,
}

impl GeomError {
    pub fn new(kind: GeomErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for GeomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for GeomError {}

impl From<std::io::Error> for GeomError {
    fn from(err: std::io::Error) -> Self {
        Self::new(GeomErrorKind::IoError, err.to_string())
    }
}

/// Convenience alias for geometry results.
pub type GeomResult<T> = Result<T, GeomError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = GeomError::new(GeomErrorKind::BooleanFailed, "union produced empty solid");
        assert_eq!(
            err.to_string(),
            "boolean operation failed: union produced empty solid"
        );
    }

    #[test]
    fn error_kind_display() {
        assert_eq!(GeomErrorKind::IoError.to_string(), "I/O error");
        assert_eq!(GeomErrorKind::InvalidInput.to_string(), "invalid input");
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let geom_err: GeomError = io_err.into();
        assert_eq!(geom_err.kind, GeomErrorKind::IoError);
        assert!(geom_err.message.contains("file missing"));
    }
}
