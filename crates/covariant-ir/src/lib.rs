//! Intermediate representation for the COVARIANT language.
//!
//! Lowers the AST from `covariant-syntax` into an arena-allocated DAG.

pub mod dag;
pub mod error;
pub mod lower;
pub mod node;

pub use dag::Dag;
pub use error::{IrError, IrErrorKind};
pub use lower::lower;
pub use node::{IrNode, IrNodeData, NodeId};

// Re-export syntax types used in the public API.
pub use covariant_syntax::{Span, Spanned};
