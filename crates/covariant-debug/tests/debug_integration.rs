//! Integration tests for the debug step collection system.

use covariant_debug::{eval_debug, DebugSession};
use covariant_geom::{GeomKernel, TruckKernel};

/// Parse and lower source code into an IR DAG.
fn parse_and_lower(source: &str) -> covariant_ir::Dag {
    let (ast, errors) = covariant_syntax::parse(source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    let (dag, ir_errors) = covariant_ir::lower(&ast);
    assert!(ir_errors.is_empty(), "IR errors: {ir_errors:?}");
    dag
}

#[test]
fn eval_debug_collects_box_step() {
    let source = r#"let b = box(vec3(10mm, 10mm, 10mm))"#;
    let dag = parse_and_lower(source);
    let kernel = TruckKernel;
    let session = eval_debug(&dag, &kernel, source.to_string(), "test.cov".to_string())
        .expect("eval_debug should succeed");

    // box() produces one solid step
    assert_eq!(session.step_count(), 1);
    assert_eq!(session.source, source);
    assert_eq!(session.file_path, "test.cov");
}

#[test]
fn eval_debug_collects_multiple_steps() {
    let source = r#"
let a = box(vec3(10mm, 10mm, 10mm))
let b = cylinder(5mm, 20mm)
let c = move(a, vec3(20mm, 0, 0))
"#;
    let dag = parse_and_lower(source);
    let kernel = TruckKernel;
    let session = eval_debug(&dag, &kernel, source.to_string(), "test.cov".to_string())
        .expect("eval_debug should succeed");

    // box, cylinder, move → 3 geometry steps
    assert_eq!(session.step_count(), 3);

    // Each step has a valid span (non-zero range)
    for step in &session.steps {
        assert!(step.span.end > step.span.start, "step should have non-zero span");
    }
}

#[test]
fn eval_debug_trace_label_appears_in_session() {
    let source = r#"
let b = trace("my box", box(vec3(10mm, 10mm, 10mm)))
"#;
    let dag = parse_and_lower(source);
    let kernel = TruckKernel;
    let session = eval_debug(&dag, &kernel, source.to_string(), "test.cov".to_string())
        .expect("eval_debug should succeed");

    // box() produces a step (no label), then trace() passes through the solid
    // and is itself recorded as a step with the label.
    assert_eq!(session.step_count(), 2);
    // The trace call's step carries the label.
    assert!(
        session.steps.iter().any(|s| s.label.as_deref() == Some("my box")),
        "trace label should appear in at least one step"
    );
}

#[test]
fn eval_debug_non_solid_calls_not_recorded() {
    let source = r#"
let x = 1 + 2
let v = vec3(10mm, 10mm, 10mm)
"#;
    let dag = parse_and_lower(source);
    let kernel = TruckKernel;
    let session = eval_debug(&dag, &kernel, source.to_string(), "test.cov".to_string())
        .expect("eval_debug should succeed");

    // No solid-producing steps
    assert_eq!(session.step_count(), 0);
}

#[test]
fn eval_debug_step_solids_tessellate_successfully() {
    let source = r#"
let a = box(vec3(20mm, 20mm, 20mm))
let b = sphere(10mm)
"#;
    let dag = parse_and_lower(source);
    let kernel = TruckKernel;
    let session = eval_debug(&dag, &kernel, source.to_string(), "test.cov".to_string())
        .expect("eval_debug should succeed");

    assert_eq!(session.step_count(), 2);

    // Each step's solid can be tessellated.
    for step in &session.steps {
        let mesh = kernel.tessellate(&step.solid, 0.1);
        assert!(!mesh.is_empty(), "step {} should tessellate to a non-empty mesh", step.index);
    }
}

#[test]
fn eval_debug_mounting_plate_example() {
    let source = std::fs::read_to_string("../../examples/mounting_plate.cov")
        .expect("mounting_plate.cov should exist");
    let dag = parse_and_lower(&source);
    let kernel = TruckKernel;
    let session = eval_debug(
        &dag,
        &kernel,
        source.clone(),
        "examples/mounting_plate.cov".to_string(),
    )
    .expect("eval_debug on mounting_plate should succeed");

    // The mounting plate example has several geometry operations:
    // plate(box), hole(threaded_hole), 4x move, union_many, difference, export_stl creates mesh
    // At minimum: box + threaded_hole + 4 move + union_many + difference = 8 steps
    assert!(
        session.step_count() >= 7,
        "expected at least 7 geometry steps from mounting_plate, got {}",
        session.step_count()
    );
}
