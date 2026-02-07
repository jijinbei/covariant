//! Integration tests for the covariant-export crate.

use covariant_export::{ExportOptions, Quality, StlFormat, export_stl, validate_mesh};
use covariant_geom::TruckKernel;
use covariant_geom::kernel::GeomKernel;

#[test]
fn binary_stl_export_creates_nonempty_file() {
    let kernel = TruckKernel;
    let solid = kernel.box_solid(20.0, 15.0, 5.0);
    let dir = std::env::temp_dir();
    let path = dir.join("integ_binary.stl");
    let opts = ExportOptions::default();
    export_stl(&kernel, &solid, &path, &opts).expect("binary export should succeed");
    let meta = std::fs::metadata(&path).expect("file should exist");
    assert!(meta.len() > 0, "binary STL should be non-empty");
    std::fs::remove_file(&path).ok();
}

#[test]
fn ascii_stl_export_starts_with_solid() {
    let kernel = TruckKernel;
    let solid = kernel.box_solid(10.0, 10.0, 10.0);
    let dir = std::env::temp_dir();
    let path = dir.join("integ_ascii.stl");
    let opts = ExportOptions {
        format: StlFormat::Ascii,
        ..ExportOptions::default()
    };
    export_stl(&kernel, &solid, &path, &opts).expect("ASCII export should succeed");
    let content = std::fs::read_to_string(&path).expect("should read file");
    assert!(
        content.starts_with("solid"),
        "ASCII STL must start with 'solid'"
    );
    std::fs::remove_file(&path).ok();
}

#[test]
fn fine_quality_produces_more_triangles_than_draft() {
    let kernel = TruckKernel;
    let solid = kernel.sphere(10.0);

    let mesh_draft = kernel.tessellate(&solid, Quality::Draft.tolerance());
    let mesh_fine = kernel.tessellate(&solid, Quality::Fine.tolerance());

    assert!(
        mesh_fine.triangle_count() > mesh_draft.triangle_count(),
        "Fine ({}) should produce more triangles than Draft ({})",
        mesh_fine.triangle_count(),
        mesh_draft.triangle_count(),
    );
}

#[test]
fn sphere_export_and_validation_no_warnings() {
    let kernel = TruckKernel;
    let solid = kernel.sphere(8.0);
    let mesh = kernel.tessellate(&solid, Quality::Standard.tolerance());
    let report = validate_mesh(&mesh);
    assert!(report.is_ok(), "sphere mesh should have no warnings");
    assert!(report.position_count > 0);
    assert!(report.triangle_count > 0);

    // Also verify STL export works
    let dir = std::env::temp_dir();
    let path = dir.join("integ_sphere.stl");
    let opts = ExportOptions::default();
    export_stl(&kernel, &solid, &path, &opts).expect("sphere export should succeed");
    let meta = std::fs::metadata(&path).expect("file should exist");
    assert!(meta.len() > 0);
    std::fs::remove_file(&path).ok();
}
