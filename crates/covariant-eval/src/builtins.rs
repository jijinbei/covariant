//! Built-in function registry for the COVARIANT evaluator.
//!
//! Registers geometric primitives, boolean operations, transformations,
//! thread functions, and utility functions into the environment.

use crate::env::Env;
use crate::eval::EvalCtx;

/// Register all built-in functions and enum constants into the environment.
pub fn register_builtins(_env: &mut Env, _ctx: &EvalCtx<'_>) {
    // Implemented in commit 3.
}
