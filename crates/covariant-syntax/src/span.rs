/// A byte-offset range in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    /// Inclusive start byte offset.
    pub start: u32,
    /// Exclusive end byte offset.
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Create a zero-length span at a position (for synthetic nodes).
    pub fn point(offset: u32) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Merge two spans into one that covers both.
    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// A value annotated with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_merge() {
        let a = Span::new(5, 10);
        let b = Span::new(8, 20);
        let merged = a.merge(b);
        assert_eq!(merged, Span::new(5, 20));
    }

    #[test]
    fn span_point() {
        let s = Span::point(42);
        assert_eq!(s.start, 42);
        assert_eq!(s.end, 42);
    }

    #[test]
    fn spanned_map() {
        let s = Spanned::new(10, Span::new(0, 2));
        let s2 = s.map(|n| n * 2);
        assert_eq!(s2.node, 20);
        assert_eq!(s2.span, Span::new(0, 2));
    }
}
