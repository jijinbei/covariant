//! Runtime values for the COVARIANT evaluator.

use std::fmt;
use std::sync::Arc;

use covariant_ir::NodeId;

use crate::env::Env;
use crate::error::EvalResult;

/// A function parameter in a runtime closure.
#[derive(Debug, Clone)]
pub struct FnParam {
    pub name: String,
    pub default: Option<NodeId>,
}

/// Type alias for built-in function pointers.
///
/// The function receives evaluated arguments and a mutable reference to
/// the evaluator context (for accessing the kernel, dag, etc.).
pub type BuiltinFnPtr = Arc<dyn Fn(&[Value], &mut crate::eval::EvalCtx<'_>) -> EvalResult<Value> + Send + Sync>;

/// A runtime value in the COVARIANT language.
#[derive(Clone)]
pub enum Value {
    /// Integer value.
    Int(i64),
    /// Floating-point value.
    Float(f64),
    /// Length in millimeters.
    Length(f64),
    /// Angle in radians.
    Angle(f64),
    /// Boolean value.
    Bool(bool),
    /// String value.
    String(String),
    /// 3D vector (x, y, z) in mm.
    Vec3([f64; 3]),
    /// B-rep solid geometry.
    Solid(covariant_geom::Solid),
    /// Tessellated mesh.
    Mesh(covariant_geom::Mesh),
    /// Ordered list of values.
    List(Vec<Value>),
    /// User-defined function (closure).
    Function {
        params: Vec<FnParam>,
        body: NodeId,
        closure_env: Env,
    },
    /// Built-in function.
    BuiltinFn {
        name: String,
        func: BuiltinFnPtr,
    },
    /// User-defined data instance.
    Data {
        type_name: String,
        fields: Vec<(String, Value)>,
    },
    /// Enum variant.
    EnumVariant {
        type_name: String,
        variant: String,
    },
    /// The unit value (no meaningful data).
    Unit,
}

impl Value {
    /// Returns a human-readable type name for error messages.
    pub fn type_name(&self) -> &str {
        match self {
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::Length(_) => "Length",
            Self::Angle(_) => "Angle",
            Self::Bool(_) => "Bool",
            Self::String(_) => "String",
            Self::Vec3(_) => "Vec3",
            Self::Solid(_) => "Solid",
            Self::Mesh(_) => "Mesh",
            Self::List(_) => "List",
            Self::Function { .. } => "Function",
            Self::BuiltinFn { .. } => "BuiltinFn",
            Self::Data { type_name, .. } => type_name,
            Self::EnumVariant { type_name, .. } => type_name,
            Self::Unit => "Unit",
        }
    }

    /// Try to extract an f64 from numeric types (Int, Float, Length).
    ///
    /// Used for numeric coercion in arithmetic and geometry operations.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Int(n) => Some(*n as f64),
            Self::Float(f) => Some(*f),
            Self::Length(l) => Some(*l),
            Self::Angle(a) => Some(*a),
            _ => None,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(n) => write!(f, "Int({n})"),
            Self::Float(v) => write!(f, "Float({v})"),
            Self::Length(v) => write!(f, "Length({v}mm)"),
            Self::Angle(v) => write!(f, "Angle({v}rad)"),
            Self::Bool(b) => write!(f, "Bool({b})"),
            Self::String(s) => write!(f, "String({s:?})"),
            Self::Vec3([x, y, z]) => write!(f, "Vec3({x}, {y}, {z})"),
            Self::Solid(_) => write!(f, "Solid(<...>)"),
            Self::Mesh(_) => write!(f, "Mesh(<...>)"),
            Self::List(items) => write!(f, "List({items:?})"),
            Self::Function { params, body, .. } => {
                let names: Vec<_> = params.iter().map(|p| &p.name).collect();
                write!(f, "Function({names:?}, body={body})")
            }
            Self::BuiltinFn { name, .. } => write!(f, "BuiltinFn({name})"),
            Self::Data { type_name, fields } => {
                let field_names: Vec<_> = fields.iter().map(|(n, _)| n.as_str()).collect();
                write!(f, "Data({type_name} {{ {field_names:?} }})")
            }
            Self::EnumVariant {
                type_name,
                variant,
            } => write!(f, "EnumVariant({type_name}::{variant})"),
            Self::Unit => write!(f, "Unit"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_name_int() {
        assert_eq!(Value::Int(42).type_name(), "Int");
    }

    #[test]
    fn type_name_float() {
        assert_eq!(Value::Float(3.14).type_name(), "Float");
    }

    #[test]
    fn type_name_length() {
        assert_eq!(Value::Length(10.0).type_name(), "Length");
    }

    #[test]
    fn type_name_angle() {
        assert_eq!(Value::Angle(1.0).type_name(), "Angle");
    }

    #[test]
    fn type_name_bool() {
        assert_eq!(Value::Bool(true).type_name(), "Bool");
    }

    #[test]
    fn type_name_string() {
        assert_eq!(Value::String("hi".to_string()).type_name(), "String");
    }

    #[test]
    fn type_name_vec3() {
        assert_eq!(Value::Vec3([1.0, 2.0, 3.0]).type_name(), "Vec3");
    }

    #[test]
    fn type_name_list() {
        assert_eq!(Value::List(vec![]).type_name(), "List");
    }

    #[test]
    fn type_name_data() {
        let v = Value::Data {
            type_name: "Rect".to_string(),
            fields: vec![],
        };
        assert_eq!(v.type_name(), "Rect");
    }

    #[test]
    fn type_name_enum() {
        let v = Value::EnumVariant {
            type_name: "Color".to_string(),
            variant: "Red".to_string(),
        };
        assert_eq!(v.type_name(), "Color");
    }

    #[test]
    fn type_name_unit() {
        assert_eq!(Value::Unit.type_name(), "Unit");
    }

    #[test]
    fn as_f64_int() {
        assert_eq!(Value::Int(5).as_f64(), Some(5.0));
    }

    #[test]
    fn as_f64_float() {
        assert_eq!(Value::Float(3.14).as_f64(), Some(3.14));
    }

    #[test]
    fn as_f64_length() {
        assert_eq!(Value::Length(10.0).as_f64(), Some(10.0));
    }

    #[test]
    fn as_f64_angle() {
        assert_eq!(Value::Angle(1.5).as_f64(), Some(1.5));
    }

    #[test]
    fn as_f64_bool_returns_none() {
        assert_eq!(Value::Bool(true).as_f64(), None);
    }

    #[test]
    fn as_f64_string_returns_none() {
        assert_eq!(Value::String("hi".to_string()).as_f64(), None);
    }

    #[test]
    fn debug_display() {
        assert_eq!(format!("{:?}", Value::Int(42)), "Int(42)");
        assert_eq!(format!("{:?}", Value::Unit), "Unit");
        assert_eq!(format!("{:?}", Value::Bool(true)), "Bool(true)");
    }
}
