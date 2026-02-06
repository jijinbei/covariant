use crate::{iso_metric, uts, ClearanceFit, ThreadKind, ThreadSpec, ThreadStandard};

/// Look up thread dimensions for a given spec.
///
/// Panics for `ThreadStandard::Bsw` (not yet implemented).
/// Returns `None` if the size is not found in the standard's database.
pub fn get_dimensions(spec: &ThreadSpec) -> Option<ThreadDimensions> {
    match spec.standard {
        ThreadStandard::IsoMetric => iso_metric::lookup(spec.size),
        ThreadStandard::Uts => uts::lookup(spec.size),
        ThreadStandard::Bsw => unimplemented!("BSW thread data not yet available"),
    }
}

/// Returns the hole diameter for a given thread kind.
pub fn hole_diameter(dims: &ThreadDimensions, kind: ThreadKind) -> f64 {
    match kind {
        ThreadKind::Internal => dims.tap_drill,
        ThreadKind::External => dims.major_diameter,
        ThreadKind::ClearanceClose => dims.clearance_close,
        ThreadKind::ClearanceMedium => dims.clearance_medium,
        ThreadKind::ClearanceFree => dims.clearance_free,
        ThreadKind::Insert => dims.insert_hole,
    }
}

/// Returns the clearance hole diameter for a given fit.
pub fn clearance_hole_diameter(dims: &ThreadDimensions, fit: ClearanceFit) -> f64 {
    match fit {
        ClearanceFit::Close => dims.clearance_close,
        ClearanceFit::Medium => dims.clearance_medium,
        ClearanceFit::Free => dims.clearance_free,
    }
}

/// Chamfer output parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChamferDimensions {
    /// Outer diameter of the chamfer in mm.
    pub outer_diameter: f64,
    /// Depth of the chamfer in mm.
    pub depth: f64,
}

/// Compute chamfer dimensions for a hole.
///
/// The chamfer outer diameter is `hole_diameter + 2 * depth` (45-degree chamfer).
/// Returns `None` if `chamfer_depth` is zero or negative.
pub fn chamfer_dimensions(hole_d: f64, chamfer_depth: f64) -> Option<ChamferDimensions> {
    if chamfer_depth <= 0.0 {
        return None;
    }
    Some(ChamferDimensions {
        outer_diameter: hole_d + 2.0 * chamfer_depth,
        depth: chamfer_depth,
    })
}

/// Thread dimensions for a specific size (all values in mm).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThreadDimensions {
    /// Nominal diameter (e.g. 5.0 for M5).
    pub nominal: f64,
    /// Thread pitch in mm.
    pub pitch: f64,
    /// Major (outer) diameter in mm.
    pub major_diameter: f64,
    /// Minor (root) diameter in mm.
    pub minor_diameter: f64,
    /// Recommended tap drill diameter in mm.
    pub tap_drill: f64,
    /// Close-fit clearance hole diameter in mm.
    pub clearance_close: f64,
    /// Medium-fit clearance hole diameter in mm.
    pub clearance_medium: f64,
    /// Free-fit clearance hole diameter in mm.
    pub clearance_free: f64,
    /// Insert (helicoil) hole diameter in mm.
    pub insert_hole: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreadSize;

    #[test]
    fn get_dimensions_iso() {
        let spec = ThreadSpec::new(ThreadSize::M5, ThreadKind::Internal, 10.0, 0.5);
        let dims = get_dimensions(&spec).unwrap();
        assert_eq!(dims.nominal, 5.0);
    }

    #[test]
    fn get_dimensions_uts() {
        let spec = ThreadSpec::new(ThreadSize::Uts1_4_20, ThreadKind::Internal, 10.0, 0.0);
        let dims = get_dimensions(&spec).unwrap();
        assert!((dims.nominal - 6.350).abs() < 0.001);
    }

    #[test]
    #[should_panic(expected = "BSW")]
    fn get_dimensions_bsw_panics() {
        let spec = ThreadSpec {
            standard: ThreadStandard::Bsw,
            size: ThreadSize::M5,
            kind: ThreadKind::Internal,
            depth: 10.0,
            chamfer: 0.0,
        };
        get_dimensions(&spec);
    }

    #[test]
    fn hole_diameter_all_kinds() {
        let dims = iso_metric::lookup(ThreadSize::M5).unwrap();
        assert_eq!(hole_diameter(&dims, ThreadKind::Internal), dims.tap_drill);
        assert_eq!(hole_diameter(&dims, ThreadKind::External), dims.major_diameter);
        assert_eq!(hole_diameter(&dims, ThreadKind::ClearanceClose), dims.clearance_close);
        assert_eq!(hole_diameter(&dims, ThreadKind::ClearanceMedium), dims.clearance_medium);
        assert_eq!(hole_diameter(&dims, ThreadKind::ClearanceFree), dims.clearance_free);
        assert_eq!(hole_diameter(&dims, ThreadKind::Insert), dims.insert_hole);
    }

    #[test]
    fn clearance_hole_all_fits() {
        let dims = iso_metric::lookup(ThreadSize::M8).unwrap();
        assert_eq!(clearance_hole_diameter(&dims, ClearanceFit::Close), 8.4);
        assert_eq!(clearance_hole_diameter(&dims, ClearanceFit::Medium), 9.0);
        assert_eq!(clearance_hole_diameter(&dims, ClearanceFit::Free), 10.0);
    }

    #[test]
    fn chamfer_45_degree() {
        let ch = chamfer_dimensions(5.0, 1.0).unwrap();
        assert_eq!(ch.outer_diameter, 7.0);
        assert_eq!(ch.depth, 1.0);
    }

    #[test]
    fn chamfer_zero_returns_none() {
        assert!(chamfer_dimensions(5.0, 0.0).is_none());
    }

    #[test]
    fn chamfer_negative_returns_none() {
        assert!(chamfer_dimensions(5.0, -0.5).is_none());
    }

    #[test]
    fn dimensions_construction() {
        let dims = ThreadDimensions {
            nominal: 5.0,
            pitch: 0.8,
            major_diameter: 5.0,
            minor_diameter: 4.134,
            tap_drill: 4.2,
            clearance_close: 5.3,
            clearance_medium: 5.5,
            clearance_free: 5.8,
            insert_hole: 6.4,
        };
        assert_eq!(dims.nominal, 5.0);
        assert_eq!(dims.pitch, 0.8);
    }

    #[test]
    fn dimensions_equality() {
        let a = ThreadDimensions {
            nominal: 3.0,
            pitch: 0.5,
            major_diameter: 3.0,
            minor_diameter: 2.459,
            tap_drill: 2.5,
            clearance_close: 3.2,
            clearance_medium: 3.4,
            clearance_free: 3.6,
            insert_hole: 4.0,
        };
        let b = a;
        assert_eq!(a, b);
    }
}
