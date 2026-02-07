//! Export pipeline for COVARIANT.
//!
//! Provides quality-controlled STL export with thread mode resolution
//! and mesh validation.

pub mod error;
pub mod quality;
pub mod stl;
pub mod thread;
pub mod validate;

pub use error::{ExportError, ExportErrorKind, ExportResult};
pub use quality::{ExportOptions, Quality, StlFormat};
pub use stl::export_stl;
pub use thread::{EffectiveThreadMode, resolve_thread_mode};
pub use validate::{MeshReport, MeshWarning, validate_mesh};
