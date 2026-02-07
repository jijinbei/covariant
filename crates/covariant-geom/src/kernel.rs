//! The `GeomKernel` trait — geometry kernel abstraction.

use crate::{Face, GeomResult, Mesh, Point3, Solid, Vector3};
use std::path::Path;

/// Abstraction over a geometry kernel that provides solid modeling operations.
///
/// `TruckKernel` is the default (and only v0.1) implementation.
pub trait GeomKernel {
    // ── Primitives ──────────────────────────────────────────────────────

    /// Create an axis-aligned box centered at the origin.
    fn box_solid(&self, size_x: f64, size_y: f64, size_z: f64) -> Solid;

    /// Create a cylinder along the Z axis, base at origin.
    fn cylinder(&self, radius: f64, height: f64) -> Solid;

    /// Create a sphere centered at the origin.
    fn sphere(&self, radius: f64) -> Solid;

    // ── Boolean operations ──────────────────────────────────────────────

    /// Boolean union of two solids.
    fn union(&self, a: &Solid, b: &Solid) -> GeomResult<Solid>;

    /// Boolean difference: `a` minus `b`.
    fn difference(&self, a: &Solid, b: &Solid) -> GeomResult<Solid>;

    /// Boolean intersection of two solids.
    fn intersection(&self, a: &Solid, b: &Solid) -> GeomResult<Solid>;

    /// Union of many solids (default: left fold).
    fn union_many(&self, solids: &[Solid]) -> GeomResult<Solid> {
        solids
            .split_first()
            .ok_or_else(|| {
                crate::GeomError::new(
                    crate::GeomErrorKind::InvalidInput,
                    "union_many requires at least one solid",
                )
            })
            .and_then(|(first, rest)| {
                rest.iter()
                    .try_fold(first.clone(), |acc, s| self.union(&acc, s))
            })
    }

    // ── Transformations ─────────────────────────────────────────────────

    /// Translate a solid by a vector.
    fn translate(&self, solid: &Solid, v: Vector3) -> Solid;

    /// Rotate a solid around an axis through `origin`.
    fn rotate(&self, solid: &Solid, origin: Point3, axis: Vector3, angle_rad: f64) -> Solid;

    /// Uniformly scale a solid about `center`.
    fn scale(&self, solid: &Solid, center: Point3, factor: f64) -> Solid;

    /// Mirror a solid across a plane defined by `origin` and `normal`.
    fn mirror(&self, solid: &Solid, origin: Point3, normal: Vector3) -> Solid;

    // ── Sweep / revolve ─────────────────────────────────────────────────

    /// Translational sweep of a face along a direction vector.
    fn sweep(&self, profile: &Face, direction: Vector3) -> Solid;

    /// Rotational sweep (revolve) of a face around an axis.
    fn revolve(
        &self,
        profile: &Face,
        origin: Point3,
        axis: Vector3,
        angle_rad: f64,
    ) -> Solid;

    // ── Tessellation / export ───────────────────────────────────────────

    /// Tessellate a solid into a triangle mesh.
    fn tessellate(&self, solid: &Solid, tolerance: f64) -> Mesh;

    /// Export a mesh to an STL file.
    fn export_stl(&self, mesh: &Mesh, path: &Path) -> GeomResult<()>;
}
