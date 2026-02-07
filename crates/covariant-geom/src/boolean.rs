//! Boolean operations on solids via truck_shapeops.

use crate::types::TruckSolid;

/// Boolean union of two solids.
pub fn solid_union(a: &TruckSolid, b: &TruckSolid, tol: f64) -> Option<TruckSolid> {
    truck_shapeops::or(a, b, tol)
}

/// Boolean intersection of two solids.
pub fn solid_intersection(a: &TruckSolid, b: &TruckSolid, tol: f64) -> Option<TruckSolid> {
    truck_shapeops::and(a, b, tol)
}

/// Boolean difference: `a` minus `b`.
///
/// Implemented as `and(a, not(b))`.
pub fn solid_difference(a: &TruckSolid, b: &TruckSolid, tol: f64) -> Option<TruckSolid> {
    let mut b_inv = b.clone();
    b_inv.not();
    truck_shapeops::and(a, &b_inv, tol)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::make_box;
    use crate::DEFAULT_TOLERANCE;
    use truck_modeling::{builder, Vector3};

    /// Make a translated copy of a solid.
    fn translated(solid: &TruckSolid, v: Vector3) -> TruckSolid {
        builder::translated(solid, v)
    }

    #[test]
    fn union_two_overlapping_boxes() {
        let a = make_box(10.0, 10.0, 10.0);
        let b = translated(&make_box(8.0, 8.0, 8.0), Vector3::new(3.0, 3.0, 3.0));
        let tol = 0.05;
        let result = solid_union(&a, &b, tol);
        assert!(result.is_some(), "union of overlapping boxes should succeed");
    }

    #[test]
    fn difference_box_minus_box() {
        let a = make_box(10.0, 10.0, 10.0);
        let b = translated(&make_box(5.0, 5.0, 20.0), Vector3::new(2.5, 2.5, -5.0));
        let tol = 0.05;
        let result = solid_difference(&a, &b, tol);
        assert!(result.is_some(), "difference should succeed");
    }

    #[test]
    fn intersection_two_overlapping_boxes() {
        let a = make_box(10.0, 10.0, 10.0);
        let b = translated(&make_box(8.0, 8.0, 8.0), Vector3::new(3.0, 3.0, 3.0));
        let tol = 0.05;
        let result = solid_intersection(&a, &b, tol);
        assert!(result.is_some(), "intersection of overlapping boxes should succeed");
    }
}
