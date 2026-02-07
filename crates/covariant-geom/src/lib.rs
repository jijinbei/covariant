//! Geometry kernel abstraction for COVARIANT.
//!
//! Provides primitives, boolean operations, transformations, sweep/revolve,
//! tessellation, and STL export via the **truck** B-rep kernel.

pub mod error;
pub mod types;

pub use error::{GeomError, GeomErrorKind, GeomResult};
pub use types::{Edge, Face, Mesh, Point3, Solid, Vector3, Wire, DEFAULT_TOLERANCE};
