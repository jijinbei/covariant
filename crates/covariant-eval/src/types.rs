//! Type representation for future static type checking.
//!
//! Not used by the evaluator in v0.1 (dynamic typing), but defined
//! here so the type system can be built incrementally.

use std::fmt;

/// Runtime/static type representation.
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int,
    Float,
    Length,
    Angle,
    Bool,
    String,
    Vec3,
    Solid,
    Mesh,
    List(Box<Ty>),
    Fn { params: Vec<Ty>, ret: Box<Ty> },
    Data(String),
    Enum(String),
    Unit,
}

impl fmt::Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::Length => write!(f, "Length"),
            Self::Angle => write!(f, "Angle"),
            Self::Bool => write!(f, "Bool"),
            Self::String => write!(f, "String"),
            Self::Vec3 => write!(f, "Vec3"),
            Self::Solid => write!(f, "Solid"),
            Self::Mesh => write!(f, "Mesh"),
            Self::List(inner) => write!(f, "List[{inner}]"),
            Self::Fn { params, ret } => {
                write!(f, "Fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ") -> {ret}")
            }
            Self::Data(name) => write!(f, "{name}"),
            Self::Enum(name) => write!(f, "{name}"),
            Self::Unit => write!(f, "Unit"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_simple_types() {
        assert_eq!(format!("{}", Ty::Int), "Int");
        assert_eq!(format!("{}", Ty::Float), "Float");
        assert_eq!(format!("{}", Ty::Length), "Length");
        assert_eq!(format!("{}", Ty::Angle), "Angle");
        assert_eq!(format!("{}", Ty::Bool), "Bool");
        assert_eq!(format!("{}", Ty::String), "String");
        assert_eq!(format!("{}", Ty::Vec3), "Vec3");
        assert_eq!(format!("{}", Ty::Solid), "Solid");
        assert_eq!(format!("{}", Ty::Mesh), "Mesh");
        assert_eq!(format!("{}", Ty::Unit), "Unit");
    }

    #[test]
    fn display_list_type() {
        let ty = Ty::List(Box::new(Ty::Int));
        assert_eq!(format!("{ty}"), "List[Int]");
    }

    #[test]
    fn display_fn_type() {
        let ty = Ty::Fn {
            params: vec![Ty::Int, Ty::Float],
            ret: Box::new(Ty::Bool),
        };
        assert_eq!(format!("{ty}"), "Fn(Int, Float) -> Bool");
    }

    #[test]
    fn display_data_type() {
        let ty = Ty::Data("Rectangle".to_string());
        assert_eq!(format!("{ty}"), "Rectangle");
    }

    #[test]
    fn display_enum_type() {
        let ty = Ty::Enum("Color".to_string());
        assert_eq!(format!("{ty}"), "Color");
    }
}
