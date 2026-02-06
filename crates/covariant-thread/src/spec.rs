use crate::{ThreadKind, ThreadSize, ThreadStandard};

/// How thread geometry should be rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThreadMode {
    /// No thread geometry — just a plain hole/cylinder.
    None,
    /// Cosmetic annotation only (lightweight).
    Cosmetic,
    /// Full helical thread geometry.
    Full,
}

/// Complete specification for a threaded hole or bolt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThreadSpec {
    pub standard: ThreadStandard,
    pub size: ThreadSize,
    pub kind: ThreadKind,
    /// Hole depth in mm.
    pub depth: f64,
    /// Chamfer depth in mm (0.0 = no chamfer).
    pub chamfer: f64,
}

impl ThreadSpec {
    /// Create a new thread spec with the given parameters.
    pub fn new(size: ThreadSize, kind: ThreadKind, depth: f64, chamfer: f64) -> Self {
        Self {
            standard: size.standard(),
            size,
            kind,
            depth,
            chamfer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_new_derives_standard() {
        let spec = ThreadSpec::new(ThreadSize::M5, ThreadKind::Internal, 10.0, 0.5);
        assert_eq!(spec.standard, ThreadStandard::IsoMetric);
        assert_eq!(spec.size, ThreadSize::M5);
        assert_eq!(spec.depth, 10.0);
        assert_eq!(spec.chamfer, 0.5);
    }

    #[test]
    fn spec_new_uts() {
        let spec = ThreadSpec::new(ThreadSize::Uts1_4_20, ThreadKind::External, 15.0, 0.0);
        assert_eq!(spec.standard, ThreadStandard::Uts);
    }

    #[test]
    fn thread_mode_equality() {
        assert_ne!(ThreadMode::None, ThreadMode::Full);
        assert_eq!(ThreadMode::Cosmetic, ThreadMode::Cosmetic);
    }
}
