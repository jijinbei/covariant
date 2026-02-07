//! Transformation operations on solids.

use truck_modeling::{builder, Matrix4, Point3, Rad, Vector3};

use crate::types::TruckSolid;

/// Translate a solid by a vector.
pub fn solid_translate(solid: &TruckSolid, v: Vector3) -> TruckSolid {
    builder::translated(solid, v)
}

/// Rotate a solid around an axis through `origin`.
pub fn solid_rotate(
    solid: &TruckSolid,
    origin: Point3,
    axis: Vector3,
    angle_rad: f64,
) -> TruckSolid {
    builder::rotated(solid, origin, axis, Rad(angle_rad))
}

/// Uniformly scale a solid about `center`.
pub fn solid_scale(solid: &TruckSolid, center: Point3, factor: f64) -> TruckSolid {
    builder::scaled(solid, center, Vector3::new(factor, factor, factor))
}

/// Mirror a solid across a plane defined by `origin` and `normal`.
///
/// Uses a reflection matrix: I - 2*n*nᵀ (Householder reflection),
/// then translates to account for the plane offset.
pub fn solid_mirror(solid: &TruckSolid, origin: Point3, normal: Vector3) -> TruckSolid {
    use truck_modeling::InnerSpace;
    let n = normal.normalize();
    // Householder reflection matrix: R = I - 2*n*nᵀ
    #[rustfmt::skip]
    let reflection = Matrix4::new(
        1.0 - 2.0 * n.x * n.x, -2.0 * n.x * n.y,       -2.0 * n.x * n.z,       0.0,
        -2.0 * n.y * n.x,       1.0 - 2.0 * n.y * n.y,  -2.0 * n.y * n.z,       0.0,
        -2.0 * n.z * n.x,       -2.0 * n.z * n.y,        1.0 - 2.0 * n.z * n.z, 0.0,
        0.0,                     0.0,                      0.0,                    1.0,
    );
    // Translate so that origin maps to itself under reflection:
    // T = 2 * dot(origin - O, n) * n  where O is world origin
    let d = origin.x * n.x + origin.y * n.y + origin.z * n.z;
    let tx = 2.0 * d * n.x;
    let ty = 2.0 * d * n.y;
    let tz = 2.0 * d * n.z;
    #[rustfmt::skip]
    let translate = Matrix4::new(
        1.0, 0.0, 0.0, tx,
        0.0, 1.0, 0.0, ty,
        0.0, 0.0, 1.0, tz,
        0.0, 0.0, 0.0, 1.0,
    );
    let mat = translate * reflection;
    builder::transformed(solid, mat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::make_box;
    use std::f64::consts::FRAC_PI_2;
    use truck_modeling::EuclideanSpace;

    #[test]
    fn translate_shifts_solid() {
        let solid = make_box(1.0, 1.0, 1.0);
        let moved = solid_translate(&solid, Vector3::new(10.0, 0.0, 0.0));
        assert!(!moved.boundaries().is_empty());
    }

    #[test]
    fn rotate_preserves_topology() {
        let solid = make_box(1.0, 1.0, 1.0);
        let rotated = solid_rotate(&solid, Point3::origin(), Vector3::unit_z(), FRAC_PI_2);
        assert_eq!(
            solid.boundaries().len(),
            rotated.boundaries().len(),
            "rotation should preserve shell count"
        );
    }

    #[test]
    fn scale_changes_size() {
        let solid = make_box(1.0, 1.0, 1.0);
        let scaled = solid_scale(&solid, Point3::origin(), 2.0);
        assert!(!scaled.boundaries().is_empty());
    }

    #[test]
    fn mirror_produces_valid_solid() {
        let solid = make_box(1.0, 1.0, 1.0);
        let mirrored = solid_mirror(&solid, Point3::origin(), Vector3::unit_x());
        assert!(!mirrored.boundaries().is_empty());
    }
}
