//! Trace event collection for debug step-through.
//!
//! `DebugStep` records a single geometry-producing evaluation step.
//! `DebugSession` collects all steps along with the source code needed
//! by the viewer.

use covariant_geom::Solid;
use covariant_ir::NodeId;
use covariant_syntax::Span;

/// A single geometry-producing step captured during evaluation.
#[derive(Debug, Clone)]
pub struct DebugStep {
    /// Zero-based index of this step in the session.
    pub index: usize,
    /// The IR node that produced this geometry.
    pub node_id: NodeId,
    /// Source span of the producing expression.
    pub span: Span,
    /// Optional label set by `trace()`.
    pub label: Option<String>,
    /// The solid produced at this step.
    pub solid: Solid,
}

/// A complete debug session: all steps plus the source needed to drive the viewer.
#[derive(Debug, Clone)]
pub struct DebugSession {
    /// Ordered list of geometry-producing steps.
    pub steps: Vec<DebugStep>,
    /// The original source code (for displaying locations).
    pub source: String,
    /// Path to the source file.
    pub file_path: String,
}

impl DebugSession {
    /// Create a new debug session from raw step data.
    pub fn new(
        raw_steps: Vec<(NodeId, Span, Option<String>, Solid)>,
        source: String,
        file_path: String,
    ) -> Self {
        let steps = raw_steps
            .into_iter()
            .enumerate()
            .map(|(i, (node_id, span, label, solid))| DebugStep {
                index: i,
                node_id,
                span,
                label,
                solid,
            })
            .collect();
        Self {
            steps,
            source,
            file_path,
        }
    }

    /// Number of geometry steps in the session.
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use covariant_ir::NodeId;
    use covariant_syntax::Span;

    fn dummy_solid() -> Solid {
        use covariant_geom::GeomKernel;
        covariant_geom::TruckKernel.box_solid(1.0, 1.0, 1.0)
    }

    #[test]
    fn debug_step_construction() {
        let step = DebugStep {
            index: 0,
            node_id: NodeId::from_raw(1),
            span: Span::new(10, 20),
            label: Some("test box".to_string()),
            solid: dummy_solid(),
        };
        assert_eq!(step.index, 0);
        assert_eq!(step.span, Span::new(10, 20));
        assert_eq!(step.label.as_deref(), Some("test box"));
    }

    #[test]
    fn debug_session_new() {
        let raw = vec![
            (
                NodeId::from_raw(1),
                Span::new(0, 10),
                Some("box".to_string()),
                dummy_solid(),
            ),
            (
                NodeId::from_raw(2),
                Span::new(11, 25),
                None,
                dummy_solid(),
            ),
        ];
        let session = DebugSession::new(raw, "source code".to_string(), "test.cov".to_string());
        assert_eq!(session.step_count(), 2);
        assert_eq!(session.steps[0].index, 0);
        assert_eq!(session.steps[1].index, 1);
        assert_eq!(session.steps[0].label.as_deref(), Some("box"));
        assert!(session.steps[1].label.is_none());
        assert_eq!(session.source, "source code");
        assert_eq!(session.file_path, "test.cov");
    }
}
