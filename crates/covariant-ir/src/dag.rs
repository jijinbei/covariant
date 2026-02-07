use covariant_syntax::Span;

use crate::node::{IrNode, IrNodeData, NodeId};

/// Arena-allocated DAG of IR nodes.
#[derive(Debug, Clone, Default)]
pub struct Dag {
    nodes: Vec<IrNodeData>,
    roots: Vec<NodeId>,
}

impl Dag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a node into the arena and return its `NodeId`.
    pub fn insert(&mut self, node: IrNode, span: Span) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(IrNodeData { node, span });
        id
    }

    /// Get the full `IrNodeData` for a node.
    pub fn get(&self, id: NodeId) -> &IrNodeData {
        &self.nodes[id.0 as usize]
    }

    /// Get the `IrNode` for a node.
    pub fn node(&self, id: NodeId) -> &IrNode {
        &self.get(id).node
    }

    /// Get the `Span` for a node.
    pub fn span(&self, id: NodeId) -> Span {
        self.get(id).span
    }

    /// Set the root nodes (top-level statements).
    pub fn set_roots(&mut self, roots: Vec<NodeId>) {
        self.roots = roots;
    }

    /// Get the root node IDs.
    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }

    /// Number of nodes in the arena.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Whether the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Iterate over all `(NodeId, &IrNodeData)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &IrNodeData)> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, data)| (NodeId(i as u32), data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut dag = Dag::new();
        let id = dag.insert(IrNode::IntLit(42), Span::new(0, 2));
        assert_eq!(id, NodeId(0));
        assert_eq!(*dag.node(id), IrNode::IntLit(42));
        assert_eq!(dag.span(id), Span::new(0, 2));
    }

    #[test]
    fn multiple_nodes() {
        let mut dag = Dag::new();
        let a = dag.insert(IrNode::IntLit(1), Span::new(0, 1));
        let b = dag.insert(IrNode::IntLit(2), Span::new(2, 3));
        let c = dag.insert(IrNode::IntLit(3), Span::new(4, 5));
        assert_eq!(a, NodeId(0));
        assert_eq!(b, NodeId(1));
        assert_eq!(c, NodeId(2));
        assert_eq!(dag.len(), 3);
    }

    #[test]
    fn roots() {
        let mut dag = Dag::new();
        let a = dag.insert(IrNode::IntLit(1), Span::new(0, 1));
        let b = dag.insert(IrNode::IntLit(2), Span::new(2, 3));
        dag.set_roots(vec![a, b]);
        assert_eq!(dag.roots(), &[NodeId(0), NodeId(1)]);
    }

    #[test]
    fn empty_dag() {
        let dag = Dag::new();
        assert!(dag.is_empty());
        assert_eq!(dag.len(), 0);
        assert!(dag.roots().is_empty());
    }

    #[test]
    fn iterate() {
        let mut dag = Dag::new();
        dag.insert(IrNode::BoolLit(true), Span::new(0, 4));
        dag.insert(IrNode::BoolLit(false), Span::new(5, 10));
        let items: Vec<_> = dag.iter().collect();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, NodeId(0));
        assert_eq!(items[1].0, NodeId(1));
    }

    #[test]
    fn default_is_empty() {
        let dag: Dag = Default::default();
        assert!(dag.is_empty());
    }

    #[test]
    fn get_full_data() {
        let mut dag = Dag::new();
        let id = dag.insert(IrNode::StringLit("hello".to_string()), Span::new(0, 7));
        let data = dag.get(id);
        assert_eq!(data.node, IrNode::StringLit("hello".to_string()));
        assert_eq!(data.span, Span::new(0, 7));
    }
}
