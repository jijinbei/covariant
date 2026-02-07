//! End-to-end integration tests for covariant-geom.

use covariant_geom::{GeomKernel, Point3, TruckKernel, Vector3};

fn kernel() -> TruckKernel {
    TruckKernel
}

#[test]
fn mounting_plate_workflow() {
    let k = kernel();

    // Create a plate and translate it so it's centered on x/y.
    let plate = k.box_solid(40.0, 30.0, 5.0);
    let plate = k.translate(&plate, Vector3::new(-20.0, -15.0, 0.0));

    // Create a cylindrical hole and subtract it.
    let hole = k.cylinder(4.0, 10.0);
    let hole = k.translate(&hole, Vector3::new(0.0, 0.0, -2.5));

    let result = k.difference(&plate, &hole);
    assert!(result.is_ok(), "plate - hole should succeed");
    let result = result.unwrap();

    // Tessellate and export to STL.
    let mesh = k.tessellate(&result, 0.1);
    let dir = std::env::temp_dir();
    let path = dir.join("covariant_integration_plate.stl");
    k.export_stl(&mesh, &path)
        .expect("STL export should succeed");

    let meta = std::fs::metadata(&path).expect("STL file should exist");
    assert!(meta.len() > 0, "STL file should be non-empty");
    std::fs::remove_file(&path).ok();
}

#[test]
fn union_many_cylinders() {
    let k = kernel();

    // Create 4 small cylinders at different positions and union them.
    let offsets = [
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(10.0, 0.0, 0.0),
        Vector3::new(0.0, 10.0, 0.0),
        Vector3::new(10.0, 10.0, 0.0),
    ];

    let cylinders: Vec<_> = offsets
        .iter()
        .map(|&off| k.translate(&k.cylinder(2.0, 8.0), off))
        .collect();

    let result = k.union_many(&cylinders);
    assert!(result.is_ok(), "union_many should succeed");
}

#[test]
fn sphere_tessellation_roundtrip() {
    let k = kernel();
    let sphere = k.sphere(10.0);
    let mesh = k.tessellate(&sphere, 0.5);

    let dir = std::env::temp_dir();
    let path = dir.join("covariant_integration_sphere.stl");
    k.export_stl(&mesh, &path)
        .expect("sphere STL export should succeed");

    let meta = std::fs::metadata(&path).expect("STL file should exist");
    assert!(meta.len() > 84, "sphere STL should have more than just a header");
    std::fs::remove_file(&path).ok();
}

#[test]
fn transform_chain() {
    let k = kernel();
    let solid = k.box_solid(5.0, 5.0, 5.0);
    let moved = k.translate(&solid, Vector3::new(10.0, 0.0, 0.0));
    let rotated = k.rotate(
        &moved,
        Point3::new(0.0, 0.0, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
        std::f64::consts::FRAC_PI_4,
    );
    let scaled = k.scale(&rotated, Point3::new(0.0, 0.0, 0.0), 2.0);
    let _mirrored = k.mirror(&scaled, Point3::new(0.0, 0.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
}
