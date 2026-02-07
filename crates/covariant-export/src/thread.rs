//! Thread mode resolution for STL export.
//!
//! For v0.1, `Full` and `Cosmetic` thread modes fall back to `None`
//! because helical sweep geometry is not yet supported in the truck
//! kernel, and cosmetic annotations are meaningless for STL.

use covariant_thread::ThreadMode;

/// The effective thread rendering mode after resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveThreadMode {
    /// No thread geometry — plain cylindrical holes.
    None,
}

/// Resolve a requested `ThreadMode` into an effective mode for STL export.
///
/// Returns the effective mode and an optional warning message explaining
/// any fallback that occurred.
pub fn resolve_thread_mode(mode: ThreadMode) -> (EffectiveThreadMode, Option<&'static str>) {
    match mode {
        ThreadMode::None => (EffectiveThreadMode::None, None),
        ThreadMode::Cosmetic => (
            EffectiveThreadMode::None,
            Some("cosmetic thread annotations are not supported in STL; falling back to None"),
        ),
        ThreadMode::Full => (
            EffectiveThreadMode::None,
            Some("full helical thread geometry is not yet implemented; falling back to None"),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_mode_no_warning() {
        let (mode, warning) = resolve_thread_mode(ThreadMode::None);
        assert_eq!(mode, EffectiveThreadMode::None);
        assert!(warning.is_none());
    }

    #[test]
    fn cosmetic_falls_back_with_warning() {
        let (mode, warning) = resolve_thread_mode(ThreadMode::Cosmetic);
        assert_eq!(mode, EffectiveThreadMode::None);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("cosmetic"));
    }

    #[test]
    fn full_falls_back_with_warning() {
        let (mode, warning) = resolve_thread_mode(ThreadMode::Full);
        assert_eq!(mode, EffectiveThreadMode::None);
        assert!(warning.is_some());
        assert!(warning.unwrap().contains("helical"));
    }
}
