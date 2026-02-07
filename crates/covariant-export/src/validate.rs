//! Mesh validation and reporting.

use covariant_geom::Mesh;

/// A warning about a potential mesh issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshWarning {
    /// The mesh contains no geometry at all.
    EmptyMesh,
}

impl std::fmt::Display for MeshWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMesh => write!(f, "mesh is empty (no positions or triangles)"),
        }
    }
}

/// Summary report from mesh validation.
#[derive(Debug, Clone)]
pub struct MeshReport {
    /// Number of vertex positions.
    pub position_count: usize,
    /// Number of triangles.
    pub triangle_count: usize,
    /// Any warnings detected during validation.
    pub warnings: Vec<MeshWarning>,
}

impl MeshReport {
    /// Returns `true` if validation found no warnings.
    pub fn is_ok(&self) -> bool {
        self.warnings.is_empty()
    }
}

/// Validate a mesh and produce a report.
pub fn validate_mesh(mesh: &Mesh) -> MeshReport {
    let position_count = mesh.position_count();
    let triangle_count = mesh.triangle_count();

    let warnings = if mesh.is_empty() {
        vec![MeshWarning::EmptyMesh]
    } else {
        vec![]
    };

    MeshReport {
        position_count,
        triangle_count,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use covariant_geom::TruckKernel;
    use covariant_geom::kernel::GeomKernel;

    #[test]
    fn validate_tessellated_box_no_warnings() {
        let kernel = TruckKernel;
        let solid = kernel.box_solid(10.0, 10.0, 10.0);
        let mesh = kernel.tessellate(&solid, 0.1);
        let report = validate_mesh(&mesh);
        assert!(report.is_ok());
        assert!(report.position_count > 0);
        assert!(report.triangle_count > 0);
    }

    #[test]
    fn validate_empty_mesh_warns() {
        let mesh = covariant_geom::Mesh::empty();
        let report = validate_mesh(&mesh);
        assert!(!report.is_ok());
        assert_eq!(report.warnings, vec![MeshWarning::EmptyMesh]);
        assert_eq!(report.position_count, 0);
        assert_eq!(report.triangle_count, 0);
    }
}
