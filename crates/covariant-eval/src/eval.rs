//! The COVARIANT evaluator — walks the IR DAG and produces runtime values.

use covariant_geom::GeomKernel;
use covariant_ir::Dag;

use crate::env::Env;
use crate::error::EvalResult;
use crate::value::Value;

/// Evaluation context holding references to the DAG, environment, and geometry kernel.
pub struct EvalCtx<'a> {
    pub dag: &'a Dag,
    pub env: Env,
    pub kernel: &'a dyn GeomKernel,
}

/// Evaluate an IR DAG, returning the value of the last root node.
pub fn eval(_dag: &Dag, _kernel: &dyn GeomKernel) -> EvalResult<Value> {
    // Implemented in commit 4.
    Ok(Value::Unit)
}
