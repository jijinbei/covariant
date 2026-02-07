//! The COVARIANT evaluator — walks the IR DAG and produces runtime values.

use covariant_geom::GeomKernel;
use covariant_ir::Dag;
use covariant_syntax::Span;

use crate::env::Env;
use crate::error::{EvalError, EvalErrorKind, EvalResult};
use crate::value::Value;

/// Evaluation context holding references to the DAG, environment, and geometry kernel.
pub struct EvalCtx<'a> {
    pub dag: &'a Dag,
    pub env: Env,
    pub kernel: &'a dyn GeomKernel,
}

impl<'a> EvalCtx<'a> {
    /// Call a value as a function with the given arguments.
    ///
    /// Used by builtins like `map` that need to invoke user-provided functions.
    pub fn call_value(
        &mut self,
        func: &Value,
        args: &[Value],
        span: Option<Span>,
    ) -> EvalResult<Value> {
        match func {
            Value::BuiltinFn { func: f, .. } => f(args, self),
            Value::Function {
                params,
                body,
                closure_env,
            } => {
                let mut call_env = closure_env.clone();
                call_env.push_scope();
                for (param, arg) in params.iter().zip(args.iter()) {
                    call_env.define(&param.name, arg.clone());
                }
                let saved_env = std::mem::replace(&mut self.env, call_env);
                let result = self.eval_node(*body);
                self.env = saved_env;
                result
            }
            _ => Err(EvalError::new(
                EvalErrorKind::NotCallable,
                format!("cannot call value of type {}", func.type_name()),
                span,
            )),
        }
    }

    /// Evaluate a single IR node by its ID. Stub — implemented in commit 4.
    pub fn eval_node(
        &mut self,
        _id: covariant_ir::NodeId,
    ) -> EvalResult<Value> {
        Ok(Value::Unit)
    }
}

/// Evaluate an IR DAG, returning the value of the last root node.
pub fn eval(_dag: &Dag, _kernel: &dyn GeomKernel) -> EvalResult<Value> {
    // Implemented in commit 4.
    Ok(Value::Unit)
}
