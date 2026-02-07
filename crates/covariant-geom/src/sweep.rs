//! Sweep and revolve operations.

use truck_modeling::{builder, Point3, Rad, Vector3};

use crate::types::{TruckFace, TruckSolid};

/// Translational sweep: extrude a face along a direction vector.
pub fn solid_sweep(face: &TruckFace, direction: Vector3) -> TruckSolid {
    builder::tsweep(face, direction)
}

/// Rotational sweep (revolve): sweep a face around an axis.
pub fn solid_revolve(
    face: &TruckFace,
    origin: Point3,
    axis: Vector3,
    angle_rad: f64,
) -> TruckSolid {
    builder::rsweep(face, origin, axis, Rad(angle_rad))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    use truck_modeling::{builder, EuclideanSpace, Wire};

    /// Helper: create a square face in the XY plane.
    fn square_face(size: f64) -> TruckFace {
        let v0 = builder::vertex(Point3::new(0.0, 0.0, 0.0));
        let v1 = builder::vertex(Point3::new(size, 0.0, 0.0));
        let v2 = builder::vertex(Point3::new(size, size, 0.0));
        let v3 = builder::vertex(Point3::new(0.0, size, 0.0));
        let wire = Wire::from(vec![
            builder::line(&v0, &v1),
            builder::line(&v1, &v2),
            builder::line(&v2, &v3),
            builder::line(&v3, &v0),
        ]);
        builder::try_attach_plane(&[wire]).expect("square wire should form a plane")
    }

    #[test]
    fn sweep_face_into_solid() {
        let face = square_face(5.0);
        let solid = solid_sweep(&face, Vector3::new(0.0, 0.0, 10.0));
        assert!(!solid.boundaries().is_empty(), "sweep should produce a solid");
    }

    #[test]
    fn revolve_face_into_solid() {
        // Revolve a small square face (offset from Z axis) around Z by 2π.
        let v0 = builder::vertex(Point3::new(3.0, 0.0, 0.0));
        let v1 = builder::vertex(Point3::new(5.0, 0.0, 0.0));
        let v2 = builder::vertex(Point3::new(5.0, 0.0, 2.0));
        let v3 = builder::vertex(Point3::new(3.0, 0.0, 2.0));
        let wire = Wire::from(vec![
            builder::line(&v0, &v1),
            builder::line(&v1, &v2),
            builder::line(&v2, &v3),
            builder::line(&v3, &v0),
        ]);
        let face =
            builder::try_attach_plane(&[wire]).expect("rectangular wire should form a plane");
        let solid = solid_revolve(&face, Point3::origin(), Vector3::unit_z(), 2.0 * PI);
        assert!(
            !solid.boundaries().is_empty(),
            "revolve should produce a solid"
        );
    }
}
