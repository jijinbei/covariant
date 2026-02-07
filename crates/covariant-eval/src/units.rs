//! Unit conversion for length and angle literals.
//!
//! All lengths are stored internally in millimeters.
//! All angles are stored internally in radians.

use covariant_syntax::ast::{AngleUnit, LengthUnit};

/// Convert a length value from the given unit to millimeters.
pub fn length_to_mm(value: f64, unit: LengthUnit) -> f64 {
    match unit {
        LengthUnit::Mm => value,
        LengthUnit::Cm => value * 10.0,
        LengthUnit::M => value * 1000.0,
        LengthUnit::In => value * 25.4,
    }
}

/// Convert an angle value from the given unit to radians.
pub fn angle_to_rad(value: f64, unit: AngleUnit) -> f64 {
    match unit {
        AngleUnit::Deg => value * std::f64::consts::PI / 180.0,
        AngleUnit::Rad => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mm_identity() {
        assert!((length_to_mm(10.0, LengthUnit::Mm) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cm_to_mm() {
        assert!((length_to_mm(1.0, LengthUnit::Cm) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn m_to_mm() {
        assert!((length_to_mm(1.0, LengthUnit::M) - 1000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn in_to_mm() {
        assert!((length_to_mm(1.0, LengthUnit::In) - 25.4).abs() < f64::EPSILON);
    }

    #[test]
    fn deg_to_rad() {
        let rad = angle_to_rad(180.0, AngleUnit::Deg);
        assert!((rad - std::f64::consts::PI).abs() < 1e-12);
    }

    #[test]
    fn rad_identity() {
        let rad = angle_to_rad(1.0, AngleUnit::Rad);
        assert!((rad - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn deg_45() {
        let rad = angle_to_rad(45.0, AngleUnit::Deg);
        assert!((rad - std::f64::consts::FRAC_PI_4).abs() < 1e-12);
    }

    #[test]
    fn deg_90() {
        let rad = angle_to_rad(90.0, AngleUnit::Deg);
        assert!((rad - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }
}
