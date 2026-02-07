//! Primitive solid construction via the truck builder.

use std::f64::consts::PI;
use truck_modeling::{builder, EuclideanSpace, Point3, Rad, Vector3};

use crate::types::TruckSolid;

/// Create an axis-aligned box with the given dimensions.
///
/// The box spans from the origin to `(size_x, size_y, size_z)`.
pub fn make_box(size_x: f64, size_y: f64, size_z: f64) -> TruckSolid {
    let vertex = builder::vertex(Point3::new(0.0, 0.0, 0.0));
    let edge = builder::tsweep(&vertex, Vector3::new(size_x, 0.0, 0.0));
    let face = builder::tsweep(&edge, Vector3::new(0.0, size_y, 0.0));
    builder::tsweep(&face, Vector3::new(0.0, 0.0, size_z))
}

/// Create a cylinder along the Z axis with base at the origin.
pub fn make_cylinder(radius: f64, height: f64) -> TruckSolid {
    let vertex = builder::vertex(Point3::new(radius, 0.0, 0.0));
    let circle = builder::rsweep(
        &vertex,
        Point3::origin(),
        Vector3::unit_z(),
        Rad(2.0 * PI),
    );
    let disk = builder::try_attach_plane(&[circle]).expect("circle should form a valid plane");
    builder::tsweep(&disk, Vector3::new(0.0, 0.0, height))
}

/// Create a sphere centered at the origin.
pub fn make_sphere(radius: f64) -> TruckSolid {
    // Build a semicircular wire from south pole to north pole, then revolve
    // it into a face, then revolve that face's wire into a solid.
    //
    // Strategy: create a semicircle wire, attach a plane to get a half-disk face,
    // then rsweep the face by 2π around Z to get a solid.
    let south = builder::vertex(Point3::new(0.0, 0.0, -radius));
    let north = builder::vertex(Point3::new(0.0, 0.0, radius));
    let transit = Point3::new(radius, 0.0, 0.0);
    let arc = builder::circle_arc(&south, &north, transit);
    // Close the wire with a straight line from north back to south along Z axis.
    let line = builder::line(&north, &south);
    let wire = truck_modeling::Wire::from(vec![arc, line]);
    let half_disk =
        builder::try_attach_plane(&[wire]).expect("semicircle+line should form a plane");
    // Revolve the half-disk face 2π around the Z axis to produce a sphere solid.
    builder::rsweep(&half_disk, Point3::origin(), Vector3::unit_z(), Rad(2.0 * PI))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_solid_creates_nonempty_boundary() {
        let solid = make_box(10.0, 20.0, 30.0);
        assert!(!solid.boundaries().is_empty(), "box should have at least one shell");
    }

    #[test]
    fn cylinder_creates_nonempty_boundary() {
        let solid = make_cylinder(5.0, 15.0);
        assert!(!solid.boundaries().is_empty(), "cylinder should have at least one shell");
    }

    #[test]
    fn sphere_creates_nonempty_boundary() {
        let solid = make_sphere(8.0);
        assert!(!solid.boundaries().is_empty(), "sphere should have at least one shell");
    }
}
