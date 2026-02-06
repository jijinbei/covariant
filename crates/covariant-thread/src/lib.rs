//! Thread standards database and hole geometry for COVARIANT.
//!
//! This crate provides thread dimension data for ISO Metric and UTS standards,
//! hole diameter calculations, and geometry parameters for thread generation.

pub mod dimensions;
pub mod geometry;
pub mod iso_metric;
pub mod spec;
pub mod standard;
pub mod uts;

pub use dimensions::{
    chamfer_dimensions, clearance_hole_diameter, get_dimensions, hole_diameter,
    ChamferDimensions, ThreadDimensions,
};
pub use geometry::{
    generate_thread_geometry, ChamferParams, CosmeticAnnotation, CylinderParams, HelixParams,
    ThreadGeometry,
};
pub use spec::{ThreadMode, ThreadSpec};
pub use standard::{ClearanceFit, ThreadKind, ThreadSize, ThreadStandard};
