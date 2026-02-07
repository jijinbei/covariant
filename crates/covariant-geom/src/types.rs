//! Core geometry types — opaque wrappers around the truck kernel types.
//!
//! Public API never exposes truck generics directly.

use truck_polymesh::PolygonMesh;

/// A 3D point (re-exported from truck's cgmath-based types).
pub type Point3 = truck_modeling::Point3;
/// A 3D vector (re-exported from truck's cgmath-based types).
pub type Vector3 = truck_modeling::Vector3;

/// Default tolerance for boolean operations and tessellation.
///
/// truck_shapeops requires tolerance >= 1e-6. A small but safe default.
pub const DEFAULT_TOLERANCE: f64 = 0.05;

// ── Truck concrete type aliases (crate-internal) ────────────────────────

/// The concrete truck solid type used throughout this crate.
pub(crate) type TruckSolid = truck_modeling::Solid;
/// The concrete truck wire type.
pub(crate) type TruckWire = truck_modeling::Wire;
/// The concrete truck face type.
pub(crate) type TruckFace = truck_modeling::Face;
/// The concrete truck edge type.
pub(crate) type TruckEdge = truck_modeling::Edge;
/// The concrete truck vertex type.
#[allow(dead_code)]
pub(crate) type TruckVertex = truck_modeling::Vertex;

// ── Public newtype wrappers ─────────────────────────────────────────────

/// An opaque B-rep solid body.
#[derive(Debug, Clone)]
pub struct Solid(pub(crate) TruckSolid);

impl Solid {
    /// Wrap a truck solid.
    pub(crate) fn from_truck(inner: TruckSolid) -> Self {
        Self(inner)
    }

    /// Access the inner truck solid.
    pub(crate) fn inner(&self) -> &TruckSolid {
        &self.0
    }
}

/// An opaque wire (connected sequence of edges).
#[derive(Debug, Clone)]
pub struct Wire(pub(crate) TruckWire);

impl Wire {
    pub(crate) fn from_truck(inner: TruckWire) -> Self {
        Self(inner)
    }

    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &TruckWire {
        &self.0
    }
}

/// An opaque face (bounded surface).
#[derive(Debug, Clone)]
pub struct Face(pub(crate) TruckFace);

impl Face {
    pub(crate) fn from_truck(inner: TruckFace) -> Self {
        Self(inner)
    }

    pub(crate) fn inner(&self) -> &TruckFace {
        &self.0
    }
}

/// An opaque edge (bounded curve between two vertices).
#[derive(Debug, Clone)]
pub struct Edge(pub(crate) TruckEdge);

impl Edge {
    pub(crate) fn from_truck(inner: TruckEdge) -> Self {
        Self(inner)
    }

    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &TruckEdge {
        &self.0
    }
}

/// A tessellated polygon mesh, ready for STL export.
#[derive(Debug, Clone)]
pub struct Mesh(pub(crate) PolygonMesh);

impl Mesh {
    pub(crate) fn from_polygon(inner: PolygonMesh) -> Self {
        Self(inner)
    }

    pub(crate) fn inner(&self) -> &PolygonMesh {
        &self.0
    }

    /// Number of vertex positions in the mesh.
    pub fn position_count(&self) -> usize {
        self.0.positions().len()
    }

    /// Number of triangles in the mesh.
    pub fn triangle_count(&self) -> usize {
        self.0.tri_faces().len()
    }

    /// Returns `true` if the mesh contains no geometry.
    pub fn is_empty(&self) -> bool {
        self.0.positions().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tolerance_value() {
        assert!((DEFAULT_TOLERANCE - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn point3_construction() {
        let p = Point3::new(1.0, 2.0, 3.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
        assert_eq!(p.z, 3.0);
    }

    #[test]
    fn vector3_construction() {
        let v = Vector3::new(4.0, 5.0, 6.0);
        assert_eq!(v.x, 4.0);
        assert_eq!(v.y, 5.0);
        assert_eq!(v.z, 6.0);
    }

    #[test]
    fn mesh_accessors_on_tessellated_box() {
        let solid = crate::primitives::make_box(10.0, 10.0, 10.0);
        let poly = crate::tessellate::mesh_solid(&solid, 0.1);
        let mesh = Mesh::from_polygon(poly);
        assert!(mesh.position_count() > 0);
        assert!(mesh.triangle_count() > 0);
        assert!(!mesh.is_empty());
    }

    #[test]
    fn empty_mesh_is_empty() {
        let mesh = Mesh::from_polygon(PolygonMesh::default());
        assert_eq!(mesh.position_count(), 0);
        assert_eq!(mesh.triangle_count(), 0);
        assert!(mesh.is_empty());
    }
}
