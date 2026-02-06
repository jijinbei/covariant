use covariant_thread::*;

#[test]
fn iso_end_to_end_simple() {
    let spec = ThreadSpec::new(ThreadSize::M5, ThreadKind::Internal, 10.0, 0.5);
    let dims = get_dimensions(&spec).unwrap();
    assert_eq!(dims.nominal, 5.0);

    let geom = generate_thread_geometry(&spec, ThreadMode::None).unwrap();
    match geom {
        ThreadGeometry::Simple { cylinder, chamfer } => {
            assert_eq!(cylinder.diameter, dims.tap_drill);
            assert_eq!(cylinder.depth, 10.0);
            assert!(chamfer.is_some());
        }
        _ => panic!("expected Simple"),
    }
}

#[test]
fn iso_end_to_end_full() {
    let spec = ThreadSpec::new(ThreadSize::M8, ThreadKind::Internal, 15.0, 1.0);
    let geom = generate_thread_geometry(&spec, ThreadMode::Full).unwrap();
    match geom {
        ThreadGeometry::Full {
            cylinder,
            chamfer,
            helix,
        } => {
            assert_eq!(cylinder.diameter, 6.8); // M8 tap drill
            assert_eq!(helix.major_diameter, 8.0);
            assert_eq!(helix.pitch, 1.25);
            assert!(chamfer.is_some());
        }
        _ => panic!("expected Full"),
    }
}

#[test]
fn uts_end_to_end_cosmetic() {
    let spec = ThreadSpec::new(ThreadSize::Uts1_4_20, ThreadKind::Internal, 12.0, 0.0);
    let geom = generate_thread_geometry(&spec, ThreadMode::Cosmetic).unwrap();
    match geom {
        ThreadGeometry::Cosmetic {
            annotation,
            chamfer,
            ..
        } => {
            assert!((annotation.diameter - 6.350).abs() < 0.001);
            assert!((annotation.pitch - 1.270).abs() < 0.001);
            assert!(chamfer.is_none());
        }
        _ => panic!("expected Cosmetic"),
    }
}

#[test]
fn uts_end_to_end_full() {
    let spec = ThreadSpec::new(ThreadSize::Uts1_2_13, ThreadKind::Internal, 20.0, 1.5);
    let geom = generate_thread_geometry(&spec, ThreadMode::Full).unwrap();
    match geom {
        ThreadGeometry::Full { helix, chamfer, .. } => {
            assert!((helix.major_diameter - 12.700).abs() < 0.001);
            assert!(chamfer.is_some());
        }
        _ => panic!("expected Full"),
    }
}

#[test]
fn every_iso_size_produces_valid_geometry() {
    for size in &ThreadSize::ALL[..15] {
        let spec = ThreadSpec::new(*size, ThreadKind::Internal, 10.0, 0.5);
        let geom = generate_thread_geometry(&spec, ThreadMode::Full).unwrap();
        match geom {
            ThreadGeometry::Full {
                cylinder, helix, ..
            } => {
                assert!(cylinder.diameter > 0.0, "{size}: cylinder diameter");
                assert!(helix.pitch > 0.0, "{size}: helix pitch");
                assert!(
                    helix.major_diameter > helix.minor_diameter,
                    "{size}: major > minor"
                );
            }
            _ => panic!("{size}: expected Full"),
        }
    }
}

#[test]
fn every_uts_size_produces_valid_geometry() {
    for size in &ThreadSize::ALL[15..] {
        let spec = ThreadSpec::new(*size, ThreadKind::Internal, 10.0, 0.5);
        let geom = generate_thread_geometry(&spec, ThreadMode::Full).unwrap();
        match geom {
            ThreadGeometry::Full {
                cylinder, helix, ..
            } => {
                assert!(cylinder.diameter > 0.0, "{size}: cylinder diameter");
                assert!(helix.pitch > 0.0, "{size}: helix pitch");
                assert!(
                    helix.major_diameter > helix.minor_diameter,
                    "{size}: major > minor"
                );
            }
            _ => panic!("{size}: expected Full"),
        }
    }
}

#[test]
fn all_thread_kinds_produce_geometry() {
    for kind in ThreadKind::ALL {
        let spec = ThreadSpec::new(ThreadSize::M10, *kind, 15.0, 0.0);
        let geom = generate_thread_geometry(&spec, ThreadMode::None).unwrap();
        match geom {
            ThreadGeometry::Simple { cylinder, .. } => {
                assert!(cylinder.diameter > 0.0, "{kind}: diameter");
            }
            _ => panic!("{kind}: expected Simple"),
        }
    }
}

#[test]
fn all_thread_modes_produce_different_variants() {
    let spec = ThreadSpec::new(ThreadSize::M6, ThreadKind::Internal, 10.0, 0.5);

    let simple = generate_thread_geometry(&spec, ThreadMode::None).unwrap();
    let cosmetic = generate_thread_geometry(&spec, ThreadMode::Cosmetic).unwrap();
    let full = generate_thread_geometry(&spec, ThreadMode::Full).unwrap();

    assert!(matches!(simple, ThreadGeometry::Simple { .. }));
    assert!(matches!(cosmetic, ThreadGeometry::Cosmetic { .. }));
    assert!(matches!(full, ThreadGeometry::Full { .. }));
}
