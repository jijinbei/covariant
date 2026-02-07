use crate::node::{IrNodeData, NodeId};

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
}
