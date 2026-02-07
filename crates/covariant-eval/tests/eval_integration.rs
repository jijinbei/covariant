//! End-to-end integration tests: source code → parse → lower → eval.

use covariant_eval::error::EvalResult;
use covariant_eval::value::Value;
use covariant_geom::TruckKernel;

/// Parse source code, lower to IR, and evaluate with the TruckKernel.
fn eval_source(src: &str) -> EvalResult<Value> {
    let (ast, parse_errors) = covariant_syntax::parse(src);
    assert!(
        parse_errors.is_empty(),
        "parse errors: {parse_errors:?}"
    );

    let (dag, ir_errors) = covariant_ir::lower(&ast);
    assert!(ir_errors.is_empty(), "IR errors: {ir_errors:?}");

    let kernel = TruckKernel;
    covariant_eval::eval(&dag, &kernel)
}

// ── Arithmetic ───────────────────────────────────────────────────────

#[test]
fn int_arithmetic() {
    let val = eval_source("let x = 1 + 2\nx").unwrap();
    assert!(matches!(val, Value::Int(3)));
}

#[test]
fn int_subtraction() {
    let val = eval_source("let x = 10 - 3\nx").unwrap();
    assert!(matches!(val, Value::Int(7)));
}

#[test]
fn int_multiplication() {
    let val = eval_source("let x = 4 * 5\nx").unwrap();
    assert!(matches!(val, Value::Int(20)));
}

#[test]
fn int_division() {
    let val = eval_source("let x = 10 / 3\nx").unwrap();
    assert!(matches!(val, Value::Int(3)));
}

#[test]
fn length_arithmetic() {
    let val = eval_source("let x = 10mm + 5mm\nx").unwrap();
    match val {
        Value::Length(l) => assert!((l - 15.0).abs() < f64::EPSILON),
        other => panic!("expected Length, got {other:?}"),
    }
}

#[test]
fn length_unit_conversion() {
    let val = eval_source("let x = 1cm + 5mm\nx").unwrap();
    match val {
        Value::Length(l) => assert!((l - 15.0).abs() < f64::EPSILON),
        other => panic!("expected Length, got {other:?}"),
    }
}

// ── Functions ────────────────────────────────────────────────────────

#[test]
fn function_definition_and_call() {
    let val = eval_source(
        "fn double(x: Int) -> Int { x * 2 }\nlet y = double(5)\ny",
    )
    .unwrap();
    assert!(matches!(val, Value::Int(10)));
}

#[test]
fn pipe_desugared() {
    let val = eval_source(
        "fn double(x: Int) -> Int { x * 2 }\nlet x = 5 |> double\nx",
    )
    .unwrap();
    assert!(matches!(val, Value::Int(10)));
}

#[test]
fn lambda_expression() {
    let val = eval_source(
        "let inc = |x| x + 1\nlet y = inc(10)\ny",
    )
    .unwrap();
    assert!(matches!(val, Value::Int(11)));
}

// ── Geometric builtins ──────────────────────────────────────────────

#[test]
fn box_solid() {
    let val = eval_source("let s = box(vec3(10mm, 10mm, 10mm))\ns").unwrap();
    assert!(matches!(val, Value::Solid(_)));
}

#[test]
fn cylinder_solid() {
    let val = eval_source("let c = cylinder(5mm, 20mm)\nc").unwrap();
    assert!(matches!(val, Value::Solid(_)));
}

#[test]
fn sphere_solid() {
    let val = eval_source("let s = sphere(10mm)\ns").unwrap();
    assert!(matches!(val, Value::Solid(_)));
}

// ── Boolean operations on solids ────────────────────────────────────

#[test]
fn difference_solids() {
    // Offset cylinder to avoid co-planar faces (truck limitation)
    let val = eval_source(
        "let a = box(vec3(20mm, 20mm, 20mm))\n\
         let b = move(cylinder(3mm, 30mm), vec3(10mm, 10mm, -5mm))\n\
         let c = difference(a, b)\n\
         c",
    )
    .unwrap();
    assert!(matches!(val, Value::Solid(_)));
}

// ── Threaded hole ───────────────────────────────────────────────────

#[test]
fn threaded_hole_builtin() {
    let val = eval_source(
        "let h = threaded_hole(ISO_METRIC, M3, TAP, 8mm, 0.5mm)\nh",
    )
    .unwrap();
    assert!(matches!(val, Value::Solid(_)));
}

// ── Control flow ────────────────────────────────────────────────────

#[test]
fn if_expression() {
    let val = eval_source("if true { 42 } else { 0 }").unwrap();
    assert!(matches!(val, Value::Int(42)));
}

#[test]
fn if_false_branch() {
    let val = eval_source("if false { 42 } else { 0 }").unwrap();
    assert!(matches!(val, Value::Int(0)));
}

// ── Data constructors ───────────────────────────────────────────────

#[test]
fn data_constructor_and_field_access() {
    let val = eval_source(
        "data Rect { width: Length, height: Length }\n\
         let r = Rect { width = 50mm, height = 100mm }\n\
         r.width",
    )
    .unwrap();
    match val {
        Value::Length(l) => assert!((l - 50.0).abs() < f64::EPSILON),
        other => panic!("expected Length, got {other:?}"),
    }
}

#[test]
fn with_update() {
    let val = eval_source(
        "data Rect { width: Length, height: Length }\n\
         let r1 = Rect { width = 50mm, height = 100mm }\n\
         let r2 = r1 with { height = 200mm }\n\
         r2.height",
    )
    .unwrap();
    match val {
        Value::Length(l) => assert!((l - 200.0).abs() < f64::EPSILON),
        other => panic!("expected Length, got {other:?}"),
    }
}

// ── Lists ───────────────────────────────────────────────────────────

#[test]
fn list_literal() {
    let val = eval_source("[1, 2, 3]").unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], Value::Int(1)));
            assert!(matches!(items[1], Value::Int(2)));
            assert!(matches!(items[2], Value::Int(3)));
        }
        other => panic!("expected List, got {other:?}"),
    }
}

// ── Boolean comparisons ─────────────────────────────────────────────

#[test]
fn comparison_lt() {
    let val = eval_source("1 < 2").unwrap();
    assert!(matches!(val, Value::Bool(true)));
}

#[test]
fn comparison_eq() {
    let val = eval_source("42 == 42").unwrap();
    assert!(matches!(val, Value::Bool(true)));
}

// ── String operations ───────────────────────────────────────────────

#[test]
fn string_concat() {
    let val = eval_source("\"hello\" + \" world\"").unwrap();
    assert!(matches!(val, Value::String(ref s) if s == "hello world"));
}

// ── Export STL ──────────────────────────────────────────────────────

#[test]
fn export_stl_writes_file() {
    let path = "/tmp/covariant_test_export.stl";
    // Clean up any leftover file
    let _ = std::fs::remove_file(path);

    let val = eval_source(&format!(
        "let s = box(vec3(10mm, 10mm, 10mm))\nexport_stl(\"{path}\", s)"
    ))
    .unwrap();
    assert!(matches!(val, Value::Unit));
    assert!(
        std::path::Path::new(path).exists(),
        "STL file was not created"
    );

    // Verify file has content
    let metadata = std::fs::metadata(path).unwrap();
    assert!(metadata.len() > 0, "STL file is empty");

    // Clean up
    let _ = std::fs::remove_file(path);
}

// ── Enum definitions ────────────────────────────────────────────────

#[test]
fn enum_definition_and_match() {
    let val = eval_source(
        "enum Color { Red, Green, Blue }\n\
         let c = Red\n\
         match c {\n\
           Red => 1,\n\
           Green => 2,\n\
           _ => 3\n\
         }",
    )
    .unwrap();
    assert!(matches!(val, Value::Int(1)));
}

// ── Block scoping ───────────────────────────────────────────────────

#[test]
fn block_scoping() {
    let val = eval_source(
        "let x = 1\n\
         let y = {\n\
           let x = 2\n\
           x\n\
         }\n\
         x + y",
    )
    .unwrap();
    // x is 1, y (from block) is 2, sum is 3
    assert!(matches!(val, Value::Int(3)));
}

// ── Negation ────────────────────────────────────────────────────────

#[test]
fn unary_negation() {
    let val = eval_source("let x = -5\nx").unwrap();
    assert!(matches!(val, Value::Int(-5)));
}

#[test]
fn logical_not() {
    let val = eval_source("!false").unwrap();
    assert!(matches!(val, Value::Bool(true)));
}
