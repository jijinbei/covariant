//! Thread standards database and hole geometry for COVARIANT.
//!
//! This crate provides thread dimension data for ISO Metric and UTS standards,
//! hole diameter calculations, and geometry parameters for thread generation.

pub mod dimensions;
pub mod spec;
pub mod standard;

pub use dimensions::ThreadDimensions;
pub use spec::{ThreadMode, ThreadSpec};
pub use standard::{ClearanceFit, ThreadKind, ThreadSize, ThreadStandard};
