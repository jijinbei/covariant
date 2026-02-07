//! `TruckKernel` — the truck-backed implementation of `GeomKernel`.

use crate::kernel::GeomKernel;
use crate::{Face, GeomError, GeomErrorKind, GeomResult, Mesh, Point3, Solid, Vector3,
            DEFAULT_TOLERANCE};
use std::path::Path;

/// Stateless geometry kernel backed by the **truck** B-rep library.
#[derive(Debug, Clone, Copy, Default)]
pub struct TruckKernel;

impl GeomKernel for TruckKernel {
    fn box_solid(&self, size_x: f64, size_y: f64, size_z: f64) -> Solid {
        Solid::from_truck(crate::primitives::make_box(size_x, size_y, size_z))
    }

    fn cylinder(&self, radius: f64, height: f64) -> Solid {
        Solid::from_truck(crate::primitives::make_cylinder(radius, height))
    }

    fn sphere(&self, radius: f64) -> Solid {
        Solid::from_truck(crate::primitives::make_sphere(radius))
    }

    fn union(&self, a: &Solid, b: &Solid) -> GeomResult<Solid> {
        crate::boolean::solid_union(a.inner(), b.inner(), DEFAULT_TOLERANCE)
            .map(Solid::from_truck)
            .ok_or_else(|| GeomError::new(GeomErrorKind::BooleanFailed, "union failed"))
    }

    fn difference(&self, a: &Solid, b: &Solid) -> GeomResult<Solid> {
        crate::boolean::solid_difference(a.inner(), b.inner(), DEFAULT_TOLERANCE)
            .map(Solid::from_truck)
            .ok_or_else(|| GeomError::new(GeomErrorKind::BooleanFailed, "difference failed"))
    }

    fn intersection(&self, a: &Solid, b: &Solid) -> GeomResult<Solid> {
        crate::boolean::solid_intersection(a.inner(), b.inner(), DEFAULT_TOLERANCE)
            .map(Solid::from_truck)
            .ok_or_else(|| GeomError::new(GeomErrorKind::BooleanFailed, "intersection failed"))
    }

    fn translate(&self, solid: &Solid, v: Vector3) -> Solid {
        Solid::from_truck(crate::transform::solid_translate(solid.inner(), v))
    }

    fn rotate(&self, solid: &Solid, origin: Point3, axis: Vector3, angle_rad: f64) -> Solid {
        Solid::from_truck(crate::transform::solid_rotate(
            solid.inner(),
            origin,
            axis,
            angle_rad,
        ))
    }

    fn scale(&self, solid: &Solid, center: Point3, factor: f64) -> Solid {
        Solid::from_truck(crate::transform::solid_scale(solid.inner(), center, factor))
    }

    fn mirror(&self, solid: &Solid, origin: Point3, normal: Vector3) -> Solid {
        Solid::from_truck(crate::transform::solid_mirror(
            solid.inner(),
            origin,
            normal,
        ))
    }

    fn sweep(&self, profile: &Face, direction: Vector3) -> Solid {
        Solid::from_truck(crate::sweep::solid_sweep(profile.inner(), direction))
    }

    fn revolve(&self, profile: &Face, origin: Point3, axis: Vector3, angle_rad: f64) -> Solid {
        Solid::from_truck(crate::sweep::solid_revolve(
            profile.inner(),
            origin,
            axis,
            angle_rad,
        ))
    }

    fn tessellate(&self, solid: &Solid, tolerance: f64) -> Mesh {
        Mesh::from_polygon(crate::tessellate::mesh_solid(solid.inner(), tolerance))
    }

    fn export_stl(&self, mesh: &Mesh, path: &Path) -> GeomResult<()> {
        crate::tessellate::write_stl(mesh.inner(), path)
    }
}
