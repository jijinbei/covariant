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
