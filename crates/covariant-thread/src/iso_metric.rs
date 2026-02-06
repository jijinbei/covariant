use crate::{ThreadDimensions, ThreadSize};

/// Look up ISO 261 metric thread dimensions by size.
///
/// Returns `None` for non-ISO sizes.
/// All values in mm per ISO 261 / ISO 262 / ISO 273.
pub fn lookup(size: ThreadSize) -> Option<ThreadDimensions> {
    // pitch, major_d, minor_d, tap_drill, cl_close, cl_med, cl_free, insert
    let data: (f64, f64, f64, f64, f64, f64, f64, f64) = match size {
        ThreadSize::M3 => (0.5, 3.0, 2.459, 2.5, 3.2, 3.4, 3.6, 4.0),
        ThreadSize::M4 => (0.7, 4.0, 3.242, 3.3, 4.3, 4.5, 4.8, 5.2),
        ThreadSize::M5 => (0.8, 5.0, 4.134, 4.2, 5.3, 5.5, 5.8, 6.4),
        _ => return None,
    };

    Some(ThreadDimensions {
        nominal: data.1,
        pitch: data.0,
        major_diameter: data.1,
        minor_diameter: data.2,
        tap_drill: data.3,
        clearance_close: data.4,
        clearance_medium: data.5,
        clearance_free: data.6,
        insert_hole: data.7,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn m3_dimensions() {
        let d = lookup(ThreadSize::M3).unwrap();
        assert_eq!(d.nominal, 3.0);
        assert_eq!(d.pitch, 0.5);
        assert_eq!(d.major_diameter, 3.0);
        assert_eq!(d.minor_diameter, 2.459);
        assert_eq!(d.tap_drill, 2.5);
        assert_eq!(d.clearance_close, 3.2);
        assert_eq!(d.clearance_medium, 3.4);
        assert_eq!(d.clearance_free, 3.6);
        assert_eq!(d.insert_hole, 4.0);
    }

    #[test]
    fn m4_dimensions() {
        let d = lookup(ThreadSize::M4).unwrap();
        assert_eq!(d.nominal, 4.0);
        assert_eq!(d.pitch, 0.7);
        assert_eq!(d.minor_diameter, 3.242);
        assert_eq!(d.tap_drill, 3.3);
    }

    #[test]
    fn m5_dimensions() {
        let d = lookup(ThreadSize::M5).unwrap();
        assert_eq!(d.nominal, 5.0);
        assert_eq!(d.pitch, 0.8);
        assert_eq!(d.minor_diameter, 4.134);
        assert_eq!(d.tap_drill, 4.2);
    }

    #[test]
    fn non_iso_returns_none() {
        assert!(lookup(ThreadSize::Uts1_4_20).is_none());
    }
}
