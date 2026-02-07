use covariant_syntax::Span;

/// Index into the DAG arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub(crate) u32);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "n{}", self.0)
    }
}

/// An IR node with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct IrNodeData {
    pub node: IrNode,
    pub span: Span,
}

/// The IR node variants, mirroring the AST but arena-allocated.
#[derive(Debug, Clone, PartialEq)]
pub enum IrNode {
    // Placeholder — filled in commit 2
}
