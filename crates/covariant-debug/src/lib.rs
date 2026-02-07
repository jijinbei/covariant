//! Debug visualization and step-through for the COVARIANT language.
//!
//! Provides instrumented evaluation that collects geometry-producing steps,
//! and a 3D viewer for step-by-step inspection.

pub mod trace;

pub use trace::{DebugSession, DebugStep};
