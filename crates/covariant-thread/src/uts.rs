use crate::{ThreadDimensions, ThreadSize};

/// Look up UTS (Unified Thread Standard) dimensions by size.
///
/// Returns `None` for non-UTS sizes.
/// All values converted to mm per ASME B1.1.
pub fn lookup(size: ThreadSize) -> Option<ThreadDimensions> {
    //           pitch   major   minor    tap     cl_c    cl_m    cl_f    insert
    let data: (f64, f64, f64, f64, f64, f64, f64, f64) = match size {
        // #2-56 UNC: 0.0860" major, 56 TPI
        ThreadSize::Uts2_56    => (0.4536, 2.184, 1.628, 1.8,   2.35,  2.5,   2.7,   2.9),
        // #4-40 UNC: 0.1120" major, 40 TPI
        ThreadSize::Uts4_40    => (0.635,  2.845, 2.157, 2.35,  3.1,   3.3,   3.6,   3.8),
        // #6-32 UNC: 0.1380" major, 32 TPI
        ThreadSize::Uts6_32    => (0.794,  3.505, 2.642, 2.85,  3.8,   4.0,   4.3,   4.6),
        // #8-32 UNC: 0.1640" major, 32 TPI
        ThreadSize::Uts8_32    => (0.794,  4.166, 3.302, 3.5,   4.5,   4.7,   5.0,   5.4),
        // #10-24 UNC: 0.1900" major, 24 TPI
        ThreadSize::Uts10_24   => (1.058,  4.826, 3.680, 3.9,   5.1,   5.3,   5.6,   6.1),
        // #10-32 UNF: 0.1900" major, 32 TPI
        ThreadSize::Uts10_32   => (0.794,  4.826, 3.962, 4.1,   5.1,   5.3,   5.6,   6.1),
        // 1/4"-20 UNC: 0.2500" major, 20 TPI
        ThreadSize::Uts1_4_20  => (1.270,  6.350, 4.976, 5.1,   6.6,   7.0,   7.4,   8.0),
        // 5/16"-18 UNC: 0.3125" major, 18 TPI
        ThreadSize::Uts5_16_18 => (1.411,  7.938, 6.401, 6.6,   8.3,   8.7,   9.1,   10.0),
        // 3/8"-16 UNC: 0.3750" major, 16 TPI
        ThreadSize::Uts3_8_16  => (1.588,  9.525, 7.798, 8.0,   9.9,  10.3,  10.7,   12.0),
        // 7/16"-14 UNC: 0.4375" major, 14 TPI
        ThreadSize::Uts7_16_14 => (1.814, 11.112, 9.144, 9.4,  11.5,  11.9,  12.3,   14.0),
        // 1/2"-13 UNC: 0.5000" major, 13 TPI
        ThreadSize::Uts1_2_13  => (1.954, 12.700,10.584,10.8,  13.0,  13.5,  14.0,   16.0),
        // 5/8"-11 UNC: 0.6250" major, 11 TPI
        ThreadSize::Uts5_8_11  => (2.309, 15.875,13.386,13.5,  16.3,  16.7,  17.5,   20.0),
        // 3/4"-10 UNC: 0.7500" major, 10 TPI
        ThreadSize::Uts3_4_10  => (2.540, 19.050,16.307,16.5,  19.5,  20.0,  21.0,   24.0),
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
    fn uts_10_24_dimensions() {
        let d = lookup(ThreadSize::Uts10_24).unwrap();
        assert!((d.nominal - 4.826).abs() < 0.001);
        assert!((d.pitch - 1.058).abs() < 0.001);
        assert_eq!(d.tap_drill, 3.9);
    }

    #[test]
    fn uts_1_4_20_dimensions() {
        let d = lookup(ThreadSize::Uts1_4_20).unwrap();
        assert!((d.nominal - 6.350).abs() < 0.001);
        assert!((d.pitch - 1.270).abs() < 0.001);
        assert_eq!(d.tap_drill, 5.1);
    }

    #[test]
    fn uts_1_2_13_dimensions() {
        let d = lookup(ThreadSize::Uts1_2_13).unwrap();
        assert!((d.nominal - 12.700).abs() < 0.001);
        assert!((d.pitch - 1.954).abs() < 0.001);
        assert_eq!(d.tap_drill, 10.8);
    }

    #[test]
    fn all_uts_sizes_present() {
        let uts_sizes = &ThreadSize::ALL[15..];
        for size in uts_sizes {
            assert!(
                lookup(*size).is_some(),
                "missing UTS data for {size}"
            );
        }
    }

    #[test]
    fn all_dimensions_positive() {
        let uts_sizes = &ThreadSize::ALL[15..];
        for size in uts_sizes {
            let d = lookup(*size).unwrap();
            assert!(d.nominal > 0.0, "{size}: nominal");
            assert!(d.pitch > 0.0, "{size}: pitch");
            assert!(d.minor_diameter > 0.0, "{size}: minor");
            assert!(d.tap_drill > 0.0, "{size}: tap_drill");
        }
    }

    #[test]
    fn diameter_ordering() {
        let uts_sizes = &ThreadSize::ALL[15..];
        for size in uts_sizes {
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
    fn non_uts_returns_none() {
        assert!(lookup(ThreadSize::M5).is_none());
    }
}
