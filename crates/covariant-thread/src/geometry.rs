use crate::{
    chamfer_dimensions, get_dimensions, hole_diameter, ChamferDimensions, ThreadMode, ThreadSpec,
};

/// Parameters for a cylindrical hole or shaft.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CylinderParams {
    pub diameter: f64,
    pub depth: f64,
}

/// Parameters for a 45-degree chamfer at a hole entrance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChamferParams {
    pub outer_diameter: f64,
    pub inner_diameter: f64,
    pub depth: f64,
}

/// Parameters for a helical thread path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HelixParams {
    pub major_diameter: f64,
    pub minor_diameter: f64,
    pub pitch: f64,
    pub depth: f64,
}

/// Cosmetic annotation data (lightweight visual indicator).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CosmeticAnnotation {
    pub diameter: f64,
    pub depth: f64,
    pub pitch: f64,
}

/// Thread geometry output — what the geometry crate will consume.
#[derive(Debug, Clone, PartialEq)]
pub enum ThreadGeometry {
    /// Plain hole/cylinder, no thread detail.
    Simple {
        cylinder: CylinderParams,
        chamfer: Option<ChamferParams>,
    },
    /// Lightweight cosmetic annotation.
    Cosmetic {
        cylinder: CylinderParams,
        chamfer: Option<ChamferParams>,
        annotation: CosmeticAnnotation,
    },
    /// Full helical thread geometry.
    Full {
        cylinder: CylinderParams,
        chamfer: Option<ChamferParams>,
        helix: HelixParams,
    },
}

/// Generate thread geometry parameters from a spec and rendering mode.
///
/// Returns `None` if the thread size is not found in the database.
pub fn generate_thread_geometry(spec: &ThreadSpec, mode: ThreadMode) -> Option<ThreadGeometry> {
    let dims = get_dimensions(spec)?;
    let hole_d = hole_diameter(&dims, spec.kind);

    let cylinder = CylinderParams {
        diameter: hole_d,
        depth: spec.depth,
    };

    let chamfer = chamfer_dimensions(hole_d, spec.chamfer).map(
        |ChamferDimensions {
             outer_diameter,
             depth,
         }| ChamferParams {
            outer_diameter,
            inner_diameter: hole_d,
            depth,
        },
    );

    Some(match mode {
        ThreadMode::None => ThreadGeometry::Simple { cylinder, chamfer },
        ThreadMode::Cosmetic => ThreadGeometry::Cosmetic {
            cylinder,
            chamfer,
            annotation: CosmeticAnnotation {
                diameter: dims.major_diameter,
                depth: spec.depth,
                pitch: dims.pitch,
            },
        },
        ThreadMode::Full => ThreadGeometry::Full {
            cylinder,
            chamfer,
            helix: HelixParams {
                major_diameter: dims.major_diameter,
                minor_diameter: dims.minor_diameter,
                pitch: dims.pitch,
                depth: spec.depth,
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ThreadKind, ThreadSize};

    fn m5_internal_spec(chamfer: f64) -> ThreadSpec {
        ThreadSpec::new(ThreadSize::M5, ThreadKind::Internal, 10.0, chamfer)
    }

    #[test]
    fn simple_no_chamfer() {
        let geom = generate_thread_geometry(&m5_internal_spec(0.0), ThreadMode::None).unwrap();
        match geom {
            ThreadGeometry::Simple { cylinder, chamfer } => {
                assert_eq!(cylinder.diameter, 4.2); // M5 tap drill
                assert_eq!(cylinder.depth, 10.0);
                assert!(chamfer.is_none());
            }
            _ => panic!("expected Simple"),
        }
    }

    #[test]
    fn simple_with_chamfer() {
        let geom = generate_thread_geometry(&m5_internal_spec(0.5), ThreadMode::None).unwrap();
        match geom {
            ThreadGeometry::Simple { chamfer, .. } => {
                let ch = chamfer.unwrap();
                assert_eq!(ch.inner_diameter, 4.2);
                assert_eq!(ch.depth, 0.5);
                assert_eq!(ch.outer_diameter, 5.2); // 4.2 + 2*0.5
            }
            _ => panic!("expected Simple"),
        }
    }

    #[test]
    fn cosmetic_mode() {
        let geom =
            generate_thread_geometry(&m5_internal_spec(0.5), ThreadMode::Cosmetic).unwrap();
        match geom {
            ThreadGeometry::Cosmetic {
                annotation,
                cylinder,
                ..
            } => {
                assert_eq!(annotation.diameter, 5.0); // M5 major
                assert_eq!(annotation.pitch, 0.8);
                assert_eq!(cylinder.diameter, 4.2);
            }
            _ => panic!("expected Cosmetic"),
        }
    }

    #[test]
    fn full_mode() {
        let geom = generate_thread_geometry(&m5_internal_spec(0.0), ThreadMode::Full).unwrap();
        match geom {
            ThreadGeometry::Full { helix, .. } => {
                assert_eq!(helix.major_diameter, 5.0);
                assert_eq!(helix.minor_diameter, 4.134);
                assert_eq!(helix.pitch, 0.8);
                assert_eq!(helix.depth, 10.0);
            }
            _ => panic!("expected Full"),
        }
    }

    #[test]
    fn external_thread_uses_major_diameter() {
        let spec = ThreadSpec::new(ThreadSize::M5, ThreadKind::External, 8.0, 0.0);
        let geom = generate_thread_geometry(&spec, ThreadMode::None).unwrap();
        match geom {
            ThreadGeometry::Simple { cylinder, .. } => {
                assert_eq!(cylinder.diameter, 5.0); // major diameter
            }
            _ => panic!("expected Simple"),
        }
    }

    #[test]
    fn clearance_hole_geometry() {
        let spec = ThreadSpec::new(ThreadSize::M5, ThreadKind::ClearanceMedium, 12.0, 0.0);
        let geom = generate_thread_geometry(&spec, ThreadMode::None).unwrap();
        match geom {
            ThreadGeometry::Simple { cylinder, .. } => {
                assert_eq!(cylinder.diameter, 5.5); // M5 clearance medium
            }
            _ => panic!("expected Simple"),
        }
    }

    #[test]
    fn uts_thread_geometry() {
        let spec = ThreadSpec::new(ThreadSize::Uts1_4_20, ThreadKind::Internal, 15.0, 1.0);
        let geom = generate_thread_geometry(&spec, ThreadMode::Full).unwrap();
        match geom {
            ThreadGeometry::Full {
                helix, chamfer, ..
            } => {
                assert!((helix.major_diameter - 6.350).abs() < 0.001);
                assert!(chamfer.is_some());
            }
            _ => panic!("expected Full"),
        }
    }
}
