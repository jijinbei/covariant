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

    fn translate(&self, _solid: &Solid, _v: Vector3) -> Solid {
        todo!("TruckKernel::translate")
    }

    fn rotate(
        &self,
        _solid: &Solid,
        _origin: Point3,
        _axis: Vector3,
        _angle_rad: f64,
    ) -> Solid {
        todo!("TruckKernel::rotate")
    }

    fn scale(&self, _solid: &Solid, _center: Point3, _factor: f64) -> Solid {
        todo!("TruckKernel::scale")
    }

    fn mirror(&self, _solid: &Solid, _origin: Point3, _normal: Vector3) -> Solid {
        todo!("TruckKernel::mirror")
    }

    fn sweep(&self, _profile: &Face, _direction: Vector3) -> Solid {
        todo!("TruckKernel::sweep")
    }

    fn revolve(
        &self,
        _profile: &Face,
        _origin: Point3,
        _axis: Vector3,
        _angle_rad: f64,
    ) -> Solid {
        todo!("TruckKernel::revolve")
    }

    fn tessellate(&self, _solid: &Solid, _tolerance: f64) -> Mesh {
        todo!("TruckKernel::tessellate")
    }

    fn export_stl(&self, _mesh: &Mesh, _path: &Path) -> GeomResult<()> {
        todo!("TruckKernel::export_stl")
    }
}
