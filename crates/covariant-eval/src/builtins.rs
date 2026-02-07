//! Built-in function registry for the COVARIANT evaluator.
//!
//! Registers geometric primitives, boolean operations, transformations,
//! thread functions, utility functions, and enum constants into the environment.

use std::path::Path;
use std::sync::Arc;

use covariant_geom::{Point3, Vector3};
use covariant_thread::{
    ThreadKind, ThreadSize, ThreadSpec, ThreadStandard, get_dimensions, hole_diameter,
};

use crate::env::Env;
use crate::error::{EvalError, EvalErrorKind, EvalResult};
use crate::eval::EvalCtx;
use crate::value::{BuiltinFnPtr, Value};

// ── Helpers ──────────────────────────────────────────────────────────────

/// Register a built-in function in the environment.
fn register(env: &mut Env, name: &str, func: BuiltinFnPtr) {
    env.define(
        name,
        Value::BuiltinFn {
            name: name.to_string(),
            func,
        },
    );
}

/// Extract a numeric f64 from a value (Int, Float, or Length), with an error message.
fn expect_f64(val: &Value, arg_name: &str) -> EvalResult<f64> {
    val.as_f64().ok_or_else(|| {
        EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected numeric value for '{arg_name}', got {}", val.type_name()),
            None,
        )
    })
}

/// Extract a Length from a value.
fn expect_length(val: &Value, arg_name: &str) -> EvalResult<f64> {
    match val {
        Value::Length(l) => Ok(*l),
        Value::Int(n) => Ok(*n as f64),
        Value::Float(f) => Ok(*f),
        _ => Err(EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected Length for '{arg_name}', got {}", val.type_name()),
            None,
        )),
    }
}

/// Extract a Solid from a value.
fn expect_solid(val: &Value, arg_name: &str) -> EvalResult<covariant_geom::Solid> {
    match val {
        Value::Solid(s) => Ok(s.clone()),
        _ => Err(EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected Solid for '{arg_name}', got {}", val.type_name()),
            None,
        )),
    }
}

/// Extract a Vec3 from a value.
fn expect_vec3(val: &Value, arg_name: &str) -> EvalResult<[f64; 3]> {
    match val {
        Value::Vec3(v) => Ok(*v),
        _ => Err(EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected Vec3 for '{arg_name}', got {}", val.type_name()),
            None,
        )),
    }
}

/// Extract a String from a value.
fn expect_string(val: &Value, arg_name: &str) -> EvalResult<String> {
    match val {
        Value::String(s) => Ok(s.clone()),
        _ => Err(EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected String for '{arg_name}', got {}", val.type_name()),
            None,
        )),
    }
}

/// Extract a ThreadStandard from an EnumVariant value.
fn expect_thread_standard(val: &Value) -> EvalResult<ThreadStandard> {
    match val {
        Value::EnumVariant { type_name, variant } if type_name == "ThreadStandard" => {
            match variant.as_str() {
                "IsoMetric" => Ok(ThreadStandard::IsoMetric),
                "Uts" => Ok(ThreadStandard::Uts),
                _ => Err(EvalError::new(
                    EvalErrorKind::TypeError,
                    format!("unknown ThreadStandard variant: {variant}"),
                    None,
                )),
            }
        }
        _ => Err(EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected ThreadStandard, got {}", val.type_name()),
            None,
        )),
    }
}

/// Extract a ThreadSize from an EnumVariant value.
fn expect_thread_size(val: &Value) -> EvalResult<ThreadSize> {
    match val {
        Value::EnumVariant { type_name, variant } if type_name == "ThreadSize" => {
            for size in ThreadSize::ALL {
                if format!("{size:?}") == *variant {
                    return Ok(*size);
                }
            }
            Err(EvalError::new(
                EvalErrorKind::TypeError,
                format!("unknown ThreadSize variant: {variant}"),
                None,
            ))
        }
        _ => Err(EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected ThreadSize, got {}", val.type_name()),
            None,
        )),
    }
}

/// Extract a ThreadKind from an EnumVariant value.
fn expect_thread_kind(val: &Value) -> EvalResult<ThreadKind> {
    match val {
        Value::EnumVariant { type_name, variant } if type_name == "ThreadKind" => {
            match variant.as_str() {
                "Internal" => Ok(ThreadKind::Internal),
                "ClearanceMedium" => Ok(ThreadKind::ClearanceMedium),
                "Insert" => Ok(ThreadKind::Insert),
                "External" => Ok(ThreadKind::External),
                "ClearanceClose" => Ok(ThreadKind::ClearanceClose),
                "ClearanceFree" => Ok(ThreadKind::ClearanceFree),
                _ => Err(EvalError::new(
                    EvalErrorKind::TypeError,
                    format!("unknown ThreadKind variant: {variant}"),
                    None,
                )),
            }
        }
        _ => Err(EvalError::new(
            EvalErrorKind::TypeError,
            format!("expected ThreadKind, got {}", val.type_name()),
            None,
        )),
    }
}

fn check_arity(name: &str, args: &[Value], expected: usize) -> EvalResult<()> {
    if args.len() != expected {
        return Err(EvalError::new(
            EvalErrorKind::ArityMismatch,
            format!("{name} expects {expected} argument(s), got {}", args.len()),
            None,
        ));
    }
    Ok(())
}

// ── Registration ─────────────────────────────────────────────────────────

/// Register all built-in functions and enum constants into the environment.
pub fn register_builtins(env: &mut Env) {
    register_geometric_primitives(env);
    register_boolean_ops(env);
    register_transforms(env);
    register_thread_fn(env);
    register_utility(env);
    register_enum_constants(env);
}

fn register_geometric_primitives(env: &mut Env) {
    // box(size: Vec3) -> Solid
    register(
        env,
        "box",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("box", args, 1)?;
            let v = expect_vec3(&args[0], "size")?;
            Ok(Value::Solid(ctx.kernel.box_solid(v[0], v[1], v[2])))
        }),
    );

    // cylinder(radius: Length, height: Length) -> Solid
    register(
        env,
        "cylinder",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("cylinder", args, 2)?;
            let r = expect_length(&args[0], "radius")?;
            let h = expect_length(&args[1], "height")?;
            Ok(Value::Solid(ctx.kernel.cylinder(r, h)))
        }),
    );

    // sphere(radius: Length) -> Solid
    register(
        env,
        "sphere",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("sphere", args, 1)?;
            let r = expect_length(&args[0], "radius")?;
            Ok(Value::Solid(ctx.kernel.sphere(r)))
        }),
    );

    // vec3(x, y, z) -> Vec3
    register(
        env,
        "vec3",
        Arc::new(|args: &[Value], _ctx: &mut EvalCtx<'_>| {
            check_arity("vec3", args, 3)?;
            let x = expect_f64(&args[0], "x")?;
            let y = expect_f64(&args[1], "y")?;
            let z = expect_f64(&args[2], "z")?;
            Ok(Value::Vec3([x, y, z]))
        }),
    );
}

fn register_boolean_ops(env: &mut Env) {
    // union(a: Solid, b: Solid) -> Solid
    register(
        env,
        "union",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("union", args, 2)?;
            let a = expect_solid(&args[0], "a")?;
            let b = expect_solid(&args[1], "b")?;
            ctx.kernel.union(&a, &b).map(Value::Solid).map_err(|e| {
                EvalError::new(EvalErrorKind::GeomError, format!("union failed: {e}"), None)
            })
        }),
    );

    // difference(a: Solid, b: Solid) -> Solid
    register(
        env,
        "difference",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("difference", args, 2)?;
            let a = expect_solid(&args[0], "a")?;
            let b = expect_solid(&args[1], "b")?;
            ctx.kernel
                .difference(&a, &b)
                .map(Value::Solid)
                .map_err(|e| {
                    EvalError::new(
                        EvalErrorKind::GeomError,
                        format!("difference failed: {e}"),
                        None,
                    )
                })
        }),
    );

    // intersect(a: Solid, b: Solid) -> Solid
    register(
        env,
        "intersect",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("intersect", args, 2)?;
            let a = expect_solid(&args[0], "a")?;
            let b = expect_solid(&args[1], "b")?;
            ctx.kernel
                .intersection(&a, &b)
                .map(Value::Solid)
                .map_err(|e| {
                    EvalError::new(
                        EvalErrorKind::GeomError,
                        format!("intersect failed: {e}"),
                        None,
                    )
                })
        }),
    );

    // union_many(solids: List[Solid]) -> Solid
    register(
        env,
        "union_many",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("union_many", args, 1)?;
            let list = match &args[0] {
                Value::List(items) => items,
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::TypeError,
                        format!(
                            "union_many expects List[Solid], got {}",
                            args[0].type_name()
                        ),
                        None,
                    ))
                }
            };
            let solids: Vec<covariant_geom::Solid> = list
                .iter()
                .enumerate()
                .map(|(i, v)| expect_solid(v, &format!("solids[{i}]")))
                .collect::<EvalResult<Vec<_>>>()?;
            ctx.kernel
                .union_many(&solids)
                .map(Value::Solid)
                .map_err(|e| {
                    EvalError::new(
                        EvalErrorKind::GeomError,
                        format!("union_many failed: {e}"),
                        None,
                    )
                })
        }),
    );
}

fn register_transforms(env: &mut Env) {
    // move(solid: Solid, v: Vec3) -> Solid
    register(
        env,
        "move",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("move", args, 2)?;
            let solid = expect_solid(&args[0], "solid")?;
            let v = expect_vec3(&args[1], "v")?;
            Ok(Value::Solid(
                ctx.kernel.translate(&solid, Vector3::new(v[0], v[1], v[2])),
            ))
        }),
    );

    // rotate(solid: Solid, axis: Vec3, angle: Angle) -> Solid
    register(
        env,
        "rotate",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("rotate", args, 3)?;
            let solid = expect_solid(&args[0], "solid")?;
            let axis = expect_vec3(&args[1], "axis")?;
            let angle = match &args[2] {
                Value::Angle(a) => *a,
                other => expect_f64(other, "angle")?,
            };
            Ok(Value::Solid(ctx.kernel.rotate(
                &solid,
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(axis[0], axis[1], axis[2]),
                angle,
            )))
        }),
    );

    // scale(solid: Solid, factor: Float) -> Solid
    register(
        env,
        "scale",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("scale", args, 2)?;
            let solid = expect_solid(&args[0], "solid")?;
            let factor = expect_f64(&args[1], "factor")?;
            Ok(Value::Solid(
                ctx.kernel
                    .scale(&solid, Point3::new(0.0, 0.0, 0.0), factor),
            ))
        }),
    );
}

fn register_thread_fn(env: &mut Env) {
    // threaded_hole(standard, size, kind, depth, chamfer) -> Solid
    register(
        env,
        "threaded_hole",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("threaded_hole", args, 5)?;
            let _standard = expect_thread_standard(&args[0])?;
            let size = expect_thread_size(&args[1])?;
            let kind = expect_thread_kind(&args[2])?;
            let depth = expect_length(&args[3], "depth")?;
            let _chamfer = expect_length(&args[4], "chamfer")?;

            let spec = ThreadSpec::new(size, kind, depth, _chamfer);
            let dims = get_dimensions(&spec).ok_or_else(|| {
                EvalError::new(
                    EvalErrorKind::Custom,
                    format!("no thread dimensions for {size}"),
                    None,
                )
            })?;
            let hole_d = hole_diameter(&dims, kind);
            let radius = hole_d / 2.0;
            Ok(Value::Solid(ctx.kernel.cylinder(radius, depth)))
        }),
    );
}

fn register_utility(env: &mut Env) {
    // trace(label: String, value) -> value
    register(
        env,
        "trace",
        Arc::new(|args: &[Value], _ctx: &mut EvalCtx<'_>| {
            check_arity("trace", args, 2)?;
            let label = expect_string(&args[0], "label")?;
            eprintln!("[trace] {label}: {:?}", args[1]);
            Ok(args[1].clone())
        }),
    );

    // export_stl(path: String, solid: Solid) -> Unit
    register(
        env,
        "export_stl",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("export_stl", args, 2)?;
            let path = expect_string(&args[0], "path")?;
            let solid = expect_solid(&args[1], "solid")?;
            let mesh = ctx
                .kernel
                .tessellate(&solid, covariant_geom::DEFAULT_TOLERANCE);
            ctx.kernel
                .export_stl(&mesh, Path::new(&path))
                .map_err(|e| {
                    EvalError::new(
                        EvalErrorKind::GeomError,
                        format!("export_stl failed: {e}"),
                        None,
                    )
                })?;
            Ok(Value::Unit)
        }),
    );

    // map(fn, list) -> list
    register(
        env,
        "map",
        Arc::new(|args: &[Value], ctx: &mut EvalCtx<'_>| {
            check_arity("map", args, 2)?;
            let func = args[0].clone();
            let list = match &args[1] {
                Value::List(items) => items.clone(),
                _ => {
                    return Err(EvalError::new(
                        EvalErrorKind::TypeError,
                        format!("map expects List as second argument, got {}", args[1].type_name()),
                        None,
                    ))
                }
            };
            let results = list
                .into_iter()
                .map(|item| ctx.call_value(&func, &[item], None))
                .collect::<EvalResult<Vec<_>>>()?;
            Ok(Value::List(results))
        }),
    );
}

fn register_enum_constants(env: &mut Env) {
    // Thread standards
    env.define(
        "ISO_METRIC",
        Value::EnumVariant {
            type_name: "ThreadStandard".to_string(),
            variant: "IsoMetric".to_string(),
        },
    );
    env.define(
        "UTS",
        Value::EnumVariant {
            type_name: "ThreadStandard".to_string(),
            variant: "Uts".to_string(),
        },
    );

    // Thread sizes — ISO Metric
    for size in ThreadSize::ALL {
        let name = format!("{size:?}");
        env.define(
            &name,
            Value::EnumVariant {
                type_name: "ThreadSize".to_string(),
                variant: name.clone(),
            },
        );
    }

    // Thread kinds — mapped from SPEC names
    // TAP → Internal, CLEARANCE → ClearanceMedium, INSERT → Insert
    env.define(
        "TAP",
        Value::EnumVariant {
            type_name: "ThreadKind".to_string(),
            variant: "Internal".to_string(),
        },
    );
    env.define(
        "CLEARANCE",
        Value::EnumVariant {
            type_name: "ThreadKind".to_string(),
            variant: "ClearanceMedium".to_string(),
        },
    );
    env.define(
        "INSERT",
        Value::EnumVariant {
            type_name: "ThreadKind".to_string(),
            variant: "Insert".to_string(),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_registered() {
        let mut env = Env::new();
        register_builtins(&mut env);
        // Geometric primitives
        assert!(env.lookup("box").is_some());
        assert!(env.lookup("cylinder").is_some());
        assert!(env.lookup("sphere").is_some());
        assert!(env.lookup("vec3").is_some());
        // Booleans
        assert!(env.lookup("union").is_some());
        assert!(env.lookup("difference").is_some());
        assert!(env.lookup("intersect").is_some());
        assert!(env.lookup("union_many").is_some());
        // Transforms
        assert!(env.lookup("move").is_some());
        assert!(env.lookup("rotate").is_some());
        assert!(env.lookup("scale").is_some());
        // Thread
        assert!(env.lookup("threaded_hole").is_some());
        // Utility
        assert!(env.lookup("trace").is_some());
        assert!(env.lookup("export_stl").is_some());
        assert!(env.lookup("map").is_some());
    }

    #[test]
    fn enum_constants_registered() {
        let mut env = Env::new();
        register_builtins(&mut env);
        // Thread standards
        assert!(matches!(
            env.lookup("ISO_METRIC"),
            Some(Value::EnumVariant { type_name, variant })
            if type_name == "ThreadStandard" && variant == "IsoMetric"
        ));
        assert!(matches!(
            env.lookup("UTS"),
            Some(Value::EnumVariant { type_name, variant })
            if type_name == "ThreadStandard" && variant == "Uts"
        ));
        // Thread sizes
        assert!(env.lookup("M3").is_some());
        assert!(env.lookup("M5").is_some());
        assert!(env.lookup("M10").is_some());
        // Thread kinds
        assert!(matches!(
            env.lookup("TAP"),
            Some(Value::EnumVariant { type_name, variant })
            if type_name == "ThreadKind" && variant == "Internal"
        ));
        assert!(matches!(
            env.lookup("CLEARANCE"),
            Some(Value::EnumVariant { type_name, variant })
            if type_name == "ThreadKind" && variant == "ClearanceMedium"
        ));
        assert!(matches!(
            env.lookup("INSERT"),
            Some(Value::EnumVariant { type_name, variant })
            if type_name == "ThreadKind" && variant == "Insert"
        ));
    }

    #[test]
    fn vec3_builtin() {
        use covariant_geom::TruckKernel;
        use covariant_ir::Dag;

        let mut env = Env::new();
        register_builtins(&mut env);
        let dag = Dag::new();
        let kernel = TruckKernel;
        let mut ctx = EvalCtx {
            dag: &dag,
            env: Env::new(),
            kernel: &kernel,
        };

        let func = env.lookup("vec3").unwrap().clone();
        if let Value::BuiltinFn { func, .. } = func {
            let args = vec![Value::Length(10.0), Value::Length(20.0), Value::Length(30.0)];
            let result = func(&args, &mut ctx).unwrap();
            assert!(matches!(result, Value::Vec3([x, y, z]) if x == 10.0 && y == 20.0 && z == 30.0));
        } else {
            panic!("expected BuiltinFn");
        }
    }
}
