//! Quality presets and export options.

use covariant_thread::ThreadMode;

/// Tessellation quality preset.
///
/// The tolerance value is the maximum chord height (distance from the true
/// surface to the tessellated approximation), in millimeters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    /// Fast preview — tolerance 0.2 mm.
    Draft,
    /// Balanced quality — tolerance 0.05 mm (default).
    Standard,
    /// High fidelity — tolerance 0.01 mm.
    Fine,
    /// User-specified tolerance in mm.
    Custom(f64),
}

impl Quality {
    /// Return the chord-height tolerance in millimeters.
    pub fn tolerance(self) -> f64 {
        match self {
            Self::Draft => 0.2,
            Self::Standard => 0.05,
            Self::Fine => 0.01,
            Self::Custom(t) => t,
        }
    }
}

/// STL output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StlFormat {
    /// Compact binary format (default).
    Binary,
    /// Human-readable ASCII format.
    Ascii,
}

/// Options controlling an STL export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExportOptions {
    /// Tessellation quality.
    pub quality: Quality,
    /// Output format.
    pub format: StlFormat,
    /// Thread rendering mode.
    pub thread_mode: ThreadMode,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            quality: Quality::Standard,
            format: StlFormat::Binary,
            thread_mode: ThreadMode::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_tolerance_values() {
        assert!((Quality::Draft.tolerance() - 0.2).abs() < f64::EPSILON);
        assert!((Quality::Standard.tolerance() - 0.05).abs() < f64::EPSILON);
        assert!((Quality::Fine.tolerance() - 0.01).abs() < f64::EPSILON);
        assert!((Quality::Custom(0.123).tolerance() - 0.123).abs() < f64::EPSILON);
    }

    #[test]
    fn export_options_default() {
        let opts = ExportOptions::default();
        assert_eq!(opts.quality, Quality::Standard);
        assert_eq!(opts.format, StlFormat::Binary);
        assert_eq!(opts.thread_mode, ThreadMode::None);
    }
}
