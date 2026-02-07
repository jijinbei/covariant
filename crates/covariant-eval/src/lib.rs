//! Evaluator for the COVARIANT language.
//!
//! Walks the IR DAG from `covariant-ir` and produces runtime values,
//! including geometry via `covariant-geom`.

pub mod builtins;
pub mod env;
pub mod error;
pub mod eval;
pub mod types;
pub mod units;
pub mod value;

pub use env::Env;
pub use error::{EvalError, EvalErrorKind, EvalResult};
pub use eval::{eval, eval_debug, RawDebugStep};
pub use types::Ty;
pub use value::Value;
