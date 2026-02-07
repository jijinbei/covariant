//! Error types for the export crate.

use std::fmt;

/// The kind of export error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportErrorKind {
    /// The geometry kernel returned an error during tessellation or writing.
    GeomError,
    /// Mesh validation detected a fatal issue (e.g. empty mesh).
    ValidationFailed,
}

impl fmt::Display for ExportErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GeomError => write!(f, "geometry error"),
            Self::ValidationFailed => write!(f, "mesh validation failed"),
        }
    }
}

/// An export error with a kind and a human-readable message.
#[derive(Debug)]
pub struct ExportError {
    pub kind: ExportErrorKind,
    pub message: String,
}

impl ExportError {
    pub fn new(kind: ExportErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for ExportError {}

impl From<covariant_geom::GeomError> for ExportError {
    fn from(err: covariant_geom::GeomError) -> Self {
        Self::new(ExportErrorKind::GeomError, err.to_string())
    }
}

/// Convenience alias for export results.
pub type ExportResult<T> = Result<T, ExportError>;
