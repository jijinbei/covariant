use crate::{ThreadDimensions, ThreadSize};

/// Look up ISO 261 metric thread dimensions by size.
///
/// Returns `None` for non-ISO sizes.
/// All values in mm per ISO 261 / ISO 262 / ISO 273.
pub fn lookup(size: ThreadSize) -> Option<ThreadDimensions> {
    //          pitch  major  minor   tap    cl_c   cl_m   cl_f   insert
    let data: (f64, f64, f64, f64, f64, f64, f64, f64) = match size {
        ThreadSize::M1_6 => (0.35, 1.6,  1.221, 1.25,  1.7,   1.8,   2.0,   2.1),
        ThreadSize::M2   => (0.4,  2.0,  1.567, 1.6,   2.2,   2.4,   2.6,   2.7),
        ThreadSize::M2_5 => (0.45, 2.5,  2.013, 2.05,  2.7,   2.9,   3.1,   3.3),
        ThreadSize::M3   => (0.5,  3.0,  2.459, 2.5,   3.2,   3.4,   3.6,   4.0),
        ThreadSize::M4   => (0.7,  4.0,  3.242, 3.3,   4.3,   4.5,   4.8,   5.2),
        ThreadSize::M5   => (0.8,  5.0,  4.134, 4.2,   5.3,   5.5,   5.8,   6.4),
        ThreadSize::M6   => (1.0,  6.0,  4.917, 5.0,   6.4,   6.6,   7.0,   7.6),
        ThreadSize::M8   => (1.25, 8.0,  6.647, 6.8,   8.4,   9.0,   10.0,  10.2),
        ThreadSize::M10  => (1.5,  10.0, 8.376, 8.5,   10.5,  11.0,  12.0,  12.7),
        ThreadSize::M12  => (1.75, 12.0, 10.106,10.2,  13.0,  13.5,  14.5,  15.2),
        ThreadSize::M14  => (2.0,  14.0, 11.835,12.0,  15.0,  15.5,  16.5,  17.7),
        ThreadSize::M16  => (2.0,  16.0, 13.835,14.0,  17.0,  17.5,  18.5,  20.2),
        ThreadSize::M20  => (2.5,  20.0, 17.294,17.5,  21.0,  22.0,  24.0,  25.2),
        ThreadSize::M24  => (3.0,  24.0, 20.752,21.0,  25.0,  26.0,  28.0,  30.2),
        ThreadSize::M30  => (3.5,  30.0, 26.211,26.5,  31.0,  33.0,  35.0,  37.7),
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
    fn m6_spot_check() {
        let d = lookup(ThreadSize::M6).unwrap();
        assert_eq!(d.nominal, 6.0);
        assert_eq!(d.pitch, 1.0);
        assert_eq!(d.tap_drill, 5.0);
    }

    #[test]
    fn m8_spot_check() {
        let d = lookup(ThreadSize::M8).unwrap();
        assert_eq!(d.nominal, 8.0);
        assert_eq!(d.pitch, 1.25);
        assert_eq!(d.tap_drill, 6.8);
    }

    #[test]
    fn m10_spot_check() {
        let d = lookup(ThreadSize::M10).unwrap();
        assert_eq!(d.nominal, 10.0);
        assert_eq!(d.pitch, 1.5);
        assert_eq!(d.tap_drill, 8.5);
    }

    #[test]
    fn all_iso_sizes_present() {
        let iso_sizes = &ThreadSize::ALL[..15];
        for size in iso_sizes {
            assert!(
                lookup(*size).is_some(),
                "missing ISO metric data for {size}"
            );
        }
    }

    #[test]
    fn all_dimensions_positive() {
        let iso_sizes = &ThreadSize::ALL[..15];
        for size in iso_sizes {
            let d = lookup(*size).unwrap();
            assert!(d.nominal > 0.0, "{size}: nominal");
            assert!(d.pitch > 0.0, "{size}: pitch");
            assert!(d.major_diameter > 0.0, "{size}: major");
            assert!(d.minor_diameter > 0.0, "{size}: minor");
            assert!(d.tap_drill > 0.0, "{size}: tap_drill");
            assert!(d.clearance_close > 0.0, "{size}: cl_close");
            assert!(d.clearance_medium > 0.0, "{size}: cl_med");
            assert!(d.clearance_free > 0.0, "{size}: cl_free");
            assert!(d.insert_hole > 0.0, "{size}: insert");
        }
    }

    #[test]
    fn diameter_ordering() {
        let iso_sizes = &ThreadSize::ALL[..15];
        for size in iso_sizes {
            let d = lookup(*size).unwrap();
            assert!(d.minor_diameter < d.major_diameter, "{size}: minor < major");
            assert!(d.tap_drill > d.minor_diameter, "{size}: tap > minor");
            assert!(d.clearance_close > d.major_diameter, "{size}: cl_close > major");
            assert!(d.clearance_medium >= d.clearance_close, "{size}: cl_med >= cl_close");
            assert!(d.clearance_free >= d.clearance_medium, "{size}: cl_free >= cl_med");
            assert!(d.insert_hole >= d.clearance_free, "{size}: insert >= cl_free");
        }
    }

    #[test]
    fn non_iso_returns_none() {
        assert!(lookup(ThreadSize::Uts1_4_20).is_none());
    }
}
