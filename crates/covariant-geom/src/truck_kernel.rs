//! `TruckKernel` — the truck-backed implementation of `GeomKernel`.

use crate::kernel::GeomKernel;
use crate::{Face, GeomResult, Mesh, Point3, Solid, Vector3};
use std::path::Path;

/// Stateless geometry kernel backed by the **truck** B-rep library.
#[derive(Debug, Clone, Copy, Default)]
pub struct TruckKernel;

impl GeomKernel for TruckKernel {
    fn box_solid(&self, _size_x: f64, _size_y: f64, _size_z: f64) -> Solid {
        todo!("TruckKernel::box_solid")
    }

    fn cylinder(&self, _radius: f64, _height: f64) -> Solid {
        todo!("TruckKernel::cylinder")
    }

    fn sphere(&self, _radius: f64) -> Solid {
        todo!("TruckKernel::sphere")
    }

    fn union(&self, _a: &Solid, _b: &Solid) -> GeomResult<Solid> {
        todo!("TruckKernel::union")
    }

    fn difference(&self, _a: &Solid, _b: &Solid) -> GeomResult<Solid> {
        todo!("TruckKernel::difference")
    }

    fn intersection(&self, _a: &Solid, _b: &Solid) -> GeomResult<Solid> {
        todo!("TruckKernel::intersection")
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
