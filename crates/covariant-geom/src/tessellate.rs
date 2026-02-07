//! Tessellation and STL export.

use std::io::BufWriter;
use std::path::Path;

use truck_meshalgo::tessellation::{MeshableShape, MeshedShape};
use truck_polymesh::PolygonMesh;

use crate::error::{GeomError, GeomErrorKind, GeomResult};
use crate::types::TruckSolid;

/// Tessellate a solid into a polygon mesh.
pub fn mesh_solid(solid: &TruckSolid, tol: f64) -> PolygonMesh {
    let meshed = solid.triangulation(tol);
    meshed.to_polygon()
}

/// Write a polygon mesh to an STL file (binary format).
pub fn write_stl(mesh: &PolygonMesh, path: &Path) -> GeomResult<()> {
    let file = std::fs::File::create(path).map_err(|e| {
        GeomError::new(
            GeomErrorKind::IoError,
            format!("failed to create STL file: {e}"),
        )
    })?;
    let mut writer = BufWriter::new(file);
    truck_polymesh::stl::write(mesh, &mut writer, truck_polymesh::stl::StlType::Binary)
        .map_err(|e| {
            GeomError::new(
                GeomErrorKind::IoError,
                format!("failed to write STL: {e}"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::make_box;

    #[test]
    fn tessellate_box_produces_mesh() {
        let solid = make_box(10.0, 10.0, 10.0);
        let mesh = mesh_solid(&solid, 0.1);
        let positions = mesh.positions();
        assert!(!positions.is_empty(), "mesh should have vertices");
    }

    #[test]
    fn export_stl_creates_file() {
        let solid = make_box(5.0, 5.0, 5.0);
        let mesh = mesh_solid(&solid, 0.1);
        let dir = std::env::temp_dir();
        let path = dir.join("covariant_test_box.stl");
        write_stl(&mesh, &path).expect("STL export should succeed");
        let meta = std::fs::metadata(&path).expect("STL file should exist");
        assert!(meta.len() > 0, "STL file should be non-empty");
        std::fs::remove_file(&path).ok();
    }
}
