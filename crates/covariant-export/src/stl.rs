//! STL export orchestration.
//!
//! Ties together quality control, thread mode resolution, tessellation,
//! mesh validation, and STL writing into a single entry point.

use std::path::Path;

use covariant_geom::Solid;
use covariant_geom::kernel::GeomKernel;

use crate::error::{ExportError, ExportErrorKind, ExportResult};
use crate::quality::{ExportOptions, StlFormat};
use crate::thread::resolve_thread_mode;
use crate::validate::{MeshWarning, validate_mesh};

/// Export a solid to an STL file with the given options.
///
/// Pipeline:
/// 1. Resolve thread mode (may emit a warning to stderr).
/// 2. Tessellate the solid at the requested quality.
/// 3. Validate the resulting mesh.
/// 4. Write the STL file in the requested format.
pub fn export_stl(
    kernel: &dyn GeomKernel,
    solid: &Solid,
    path: &Path,
    options: &ExportOptions,
) -> ExportResult<()> {
    // 1. Resolve thread mode
    let (_effective_mode, warning) = resolve_thread_mode(options.thread_mode);
    if let Some(msg) = warning {
        eprintln!("[export] warning: {msg}");
    }

    // 2. Tessellate
    let tolerance = options.quality.tolerance();
    let mesh = kernel.tessellate(solid, tolerance);

    // 3. Validate
    let report = validate_mesh(&mesh);
    if report.warnings.contains(&MeshWarning::EmptyMesh) {
        return Err(ExportError::new(
            ExportErrorKind::ValidationFailed,
            "tessellation produced an empty mesh",
        ));
    }

    // 4. Write STL
    match options.format {
        StlFormat::Binary => kernel.export_stl(&mesh, path)?,
        StlFormat::Ascii => kernel.export_stl_ascii(&mesh, path)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::Quality;
    use covariant_geom::TruckKernel;

    #[test]
    fn export_stl_binary_default_options() {
        let kernel = TruckKernel;
        let solid = kernel.box_solid(10.0, 10.0, 10.0);
        let dir = std::env::temp_dir();
        let path = dir.join("export_test_binary.stl");
        let opts = ExportOptions::default();
        export_stl(&kernel, &solid, &path, &opts).expect("binary STL export should succeed");
        let meta = std::fs::metadata(&path).expect("file should exist");
        assert!(meta.len() > 0);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn export_stl_ascii() {
        let kernel = TruckKernel;
        let solid = kernel.box_solid(5.0, 5.0, 5.0);
        let dir = std::env::temp_dir();
        let path = dir.join("export_test_ascii.stl");
        let opts = ExportOptions {
            format: StlFormat::Ascii,
            quality: Quality::Draft,
            ..ExportOptions::default()
        };
        export_stl(&kernel, &solid, &path, &opts).expect("ASCII STL export should succeed");
        let content = std::fs::read_to_string(&path).expect("should read file");
        assert!(content.starts_with("solid"));
        std::fs::remove_file(&path).ok();
    }
}
