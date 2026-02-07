//! Instrumented evaluation that collects geometry-producing steps.

use covariant_eval::EvalError;
use covariant_geom::GeomKernel;
use covariant_ir::Dag;

use crate::trace::DebugSession;

/// Evaluate an IR DAG with debug step collection, returning a `DebugSession`.
pub fn eval_debug(
    dag: &Dag,
    kernel: &dyn GeomKernel,
    source: String,
    file_path: String,
) -> Result<DebugSession, EvalError> {
    let (_value, raw_steps) = covariant_eval::eval_debug(dag, kernel)?;
    Ok(DebugSession::new(raw_steps, source, file_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use covariant_geom::TruckKernel;
    use covariant_ir::node::{IrArg, IrNode};
    use covariant_ir::Dag;
    use covariant_syntax::Span;

    #[test]
    fn eval_debug_collects_box_step() {
        // Build a DAG that calls box(vec3(10mm, 10mm, 10mm))
        let mut dag = Dag::new();

        // vec3(10, 10, 10)
        let x = dag.insert(IrNode::FloatLit(10.0), Span::new(0, 2));
        let y = dag.insert(IrNode::FloatLit(10.0), Span::new(4, 6));
        let z = dag.insert(IrNode::FloatLit(10.0), Span::new(8, 10));
        let vec3_fn = dag.insert(IrNode::Ident("vec3".to_string()), Span::new(11, 15));
        let vec3_call = dag.insert(
            IrNode::FnCall {
                func: vec3_fn,
                args: vec![
                    IrArg { name: None, value: x, span: Span::new(0, 2) },
                    IrArg { name: None, value: y, span: Span::new(4, 6) },
                    IrArg { name: None, value: z, span: Span::new(8, 10) },
                ],
            },
            Span::new(11, 20),
        );

        // box(vec3(...))
        let box_fn = dag.insert(IrNode::Ident("box".to_string()), Span::new(21, 24));
        let box_call = dag.insert(
            IrNode::FnCall {
                func: box_fn,
                args: vec![IrArg {
                    name: None,
                    value: vec3_call,
                    span: Span::new(11, 20),
                }],
            },
            Span::new(21, 30),
        );
        dag.set_roots(vec![box_call]);

        let kernel = TruckKernel;
        let session = eval_debug(&dag, &kernel, "source".to_string(), "test.cov".to_string())
            .expect("eval_debug should succeed");

        // box() produces 1 solid step
        assert_eq!(session.step_count(), 1);
        assert_eq!(session.steps[0].span, Span::new(21, 30));
    }
}
