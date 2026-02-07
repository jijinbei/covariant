use covariant_syntax::Span;

/// The kind of IR lowering error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrErrorKind {
    /// An unsupported AST construct was encountered.
    Unsupported,
}

/// An error encountered during AST → IR lowering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrError {
    pub message: String,
    pub span: Span,
    pub kind: IrErrorKind,
}

impl IrError {
    pub fn new(message: impl Into<String>, span: Span, kind: IrErrorKind) -> Self {
        Self {
            message: message.into(),
            span,
            kind,
        }
    }
}

impl std::fmt::Display for IrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{:?}] {} ({}..{})",
            self.kind, self.message, self.span.start, self.span.end
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_construction() {
        let err = IrError::new("test error", Span::new(0, 5), IrErrorKind::Unsupported);
        assert_eq!(err.message, "test error");
        assert_eq!(err.span, Span::new(0, 5));
        assert_eq!(err.kind, IrErrorKind::Unsupported);
    }

    #[test]
    fn error_display() {
        let err = IrError::new("bad node", Span::new(10, 20), IrErrorKind::Unsupported);
        let s = format!("{err}");
        assert!(s.contains("bad node"));
        assert!(s.contains("10..20"));
    }
}
