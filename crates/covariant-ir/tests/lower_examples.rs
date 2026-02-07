use std::fs;
use std::path::Path;

use covariant_ir::node::IrNode;
use covariant_ir::{Dag, IrError};
use covariant_syntax::ast::BinOpKind;

/// Parse and lower a .cov file, asserting zero errors at both stages.
fn parse_and_lower_file(path: &Path) -> (Dag, Vec<IrError>) {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let (ast, parse_errors) = covariant_syntax::parse(&source);
    assert!(
        parse_errors.is_empty(),
        "parse errors in {}:\n{}",
        path.display(),
        parse_errors
            .iter()
            .map(|e| format!("  [{:?}] {}", e.kind, e.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let (dag, lower_errors) = covariant_ir::lower(&ast);
    assert!(
        lower_errors.is_empty(),
        "lower errors in {}:\n{}",
        path.display(),
        lower_errors
            .iter()
            .map(|e| format!("  [{:?}] {}", e.kind, e.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    (dag, lower_errors)
}

fn examples_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
}

// ======== Per-file tests ========

#[test]
fn lower_mounting_plate() {
    let (dag, _) = parse_and_lower_file(&examples_dir().join("mounting_plate.cov"));
    // 4 let bindings + 1 expression statement (export_stl)
    assert_eq!(dag.roots().len(), 5, "mounting_plate.cov should have 5 roots");

    // First 4 are Let, last is FnCall (export_stl)
    for root in dag.roots().iter().take(4) {
        assert!(
            matches!(dag.node(*root), IrNode::Let { .. }),
            "expected Let root, got {:?}",
            dag.node(*root)
        );
    }
    assert!(
        matches!(dag.node(dag.roots()[4]), IrNode::FnCall { .. }),
        "last root should be FnCall (export_stl)"
    );
}

#[test]
fn lower_functions() {
    let (dag, _) = parse_and_lower_file(&examples_dir().join("functions.cov"));

    // 3 fn defs + 4 let bindings = 7
    assert_eq!(dag.roots().len(), 7, "functions.cov should have 7 roots");

    // First 3 are FnDef
    for root in dag.roots().iter().take(3) {
        assert!(
            matches!(dag.node(*root), IrNode::FnDef { .. }),
            "expected FnDef root, got {:?}",
            dag.node(*root)
        );
    }

    // Remaining are Let
    for root in dag.roots().iter().skip(3) {
        assert!(
            matches!(dag.node(*root), IrNode::Let { .. }),
            "expected Let root, got {:?}",
            dag.node(*root)
        );
    }
}

#[test]
fn lower_math() {
    let (dag, _) = parse_and_lower_file(&examples_dir().join("math.cov"));

    // All roots should be Let
    for root in dag.roots() {
        assert!(
            matches!(dag.node(*root), IrNode::Let { .. }),
            "expected Let root, got {:?}",
            dag.node(*root)
        );
    }
    // 23 let bindings
    assert_eq!(dag.roots().len(), 23, "math.cov should have 23 roots");
}

#[test]
fn lower_data_types() {
    let (dag, _) = parse_and_lower_file(&examples_dir().join("data_types.cov"));

    // Count by kind
    let data_count = dag
        .roots()
        .iter()
        .filter(|r| matches!(dag.node(**r), IrNode::DataDef { .. }))
        .count();
    let enum_count = dag
        .roots()
        .iter()
        .filter(|r| matches!(dag.node(**r), IrNode::EnumDef { .. }))
        .count();
    let let_count = dag
        .roots()
        .iter()
        .filter(|r| matches!(dag.node(**r), IrNode::Let { .. }))
        .count();

    assert!(data_count >= 2, "expected >= 2 DataDef, got {data_count}");
    assert!(enum_count >= 1, "expected >= 1 EnumDef, got {enum_count}");
    assert!(let_count >= 5, "expected >= 5 Let, got {let_count}");

    // Total: 3 data defs + 1 enum + 6 lets = 10
    assert_eq!(dag.roots().len(), 10, "data_types.cov should have 10 roots");
}

// ======== Cross-cutting tests ========

#[test]
fn no_pipe_in_any_example() {
    let dir = examples_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cov"))
        .collect();

    for entry in entries {
        let path = entry.path();
        let (dag, _) = parse_and_lower_file(&path);
        for (_id, data) in dag.iter() {
            if let IrNode::BinOp { op, .. } = &data.node {
                assert_ne!(
                    op.node,
                    BinOpKind::Pipe,
                    "Pipe found in IR of {}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn all_examples_lower_without_errors() {
    let dir = examples_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cov"))
        .collect();

    assert!(!entries.is_empty(), "no .cov files found");

    for entry in entries {
        let path = entry.path();
        parse_and_lower_file(&path);
    }
}

#[test]
fn dag_is_non_empty_for_all_examples() {
    let dir = examples_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cov"))
        .collect();

    for entry in entries {
        let path = entry.path();
        let (dag, _) = parse_and_lower_file(&path);
        assert!(
            !dag.is_empty(),
            "DAG should not be empty for {}",
            path.display()
        );
        assert!(
            !dag.roots().is_empty(),
            "roots should not be empty for {}",
            path.display()
        );
    }
}

#[test]
fn spans_are_valid_for_all_roots() {
    let dir = examples_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cov"))
        .collect();

    for entry in entries {
        let path = entry.path();
        let (dag, _) = parse_and_lower_file(&path);
        for root in dag.roots() {
            let span = dag.span(*root);
            assert!(
                span.end >= span.start,
                "invalid span for root in {}: {}..{}",
                path.display(),
                span.start,
                span.end
            );
        }
    }
}
