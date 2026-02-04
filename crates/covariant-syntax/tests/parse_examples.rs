use std::fs;
use std::path::Path;

use covariant_syntax::ast::Stmt;

/// Parse a .cov file and assert zero errors.
fn parse_file(path: &Path) -> covariant_syntax::SourceFile {
    let source = fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let (ast, errors) = covariant_syntax::parse(&source);
    assert!(
        errors.is_empty(),
        "errors in {}:\n{}",
        path.display(),
        errors
            .iter()
            .map(|e| format!("  [{:?}] {}", e.kind, e.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
    ast
}

fn examples_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
}

#[test]
fn parse_mounting_plate() {
    let ast = parse_file(&examples_dir().join("mounting_plate.cov"));
    // 4 let bindings + 1 expression statement
    assert_eq!(ast.stmts.len(), 5, "expected 5 statements in mounting_plate.cov");

    // First 4 should be Let statements
    for (i, stmt) in ast.stmts.iter().take(4).enumerate() {
        assert!(
            matches!(stmt.node, Stmt::Let(_)),
            "statement {i} should be Let, got {:?}",
            stmt.node
        );
    }

    // Last should be an expression statement (export_stl call)
    assert!(
        matches!(ast.stmts[4].node, Stmt::Expr(_)),
        "last statement should be Expr"
    );
}

#[test]
fn parse_functions() {
    let ast = parse_file(&examples_dir().join("functions.cov"));
    assert!(
        ast.stmts.len() >= 3,
        "expected at least 3 statements in functions.cov, got {}",
        ast.stmts.len()
    );

    // First statements should be function definitions
    let fn_count = ast
        .stmts
        .iter()
        .filter(|s| matches!(s.node, Stmt::FnDef(_)))
        .count();
    assert!(fn_count >= 3, "expected at least 3 fn definitions, got {fn_count}");
}

#[test]
fn parse_data_types() {
    let ast = parse_file(&examples_dir().join("data_types.cov"));
    assert!(
        !ast.stmts.is_empty(),
        "data_types.cov should have statements"
    );

    // Should contain DataDef and EnumDef statements
    let data_count = ast
        .stmts
        .iter()
        .filter(|s| matches!(s.node, Stmt::DataDef(_)))
        .count();
    let enum_count = ast
        .stmts
        .iter()
        .filter(|s| matches!(s.node, Stmt::EnumDef(_)))
        .count();
    assert!(data_count >= 2, "expected at least 2 data definitions, got {data_count}");
    assert!(enum_count >= 1, "expected at least 1 enum definition, got {enum_count}");
}

#[test]
fn parse_math() {
    let ast = parse_file(&examples_dir().join("math.cov"));
    assert!(
        ast.stmts.len() >= 10,
        "expected at least 10 statements in math.cov, got {}",
        ast.stmts.len()
    );

    // All should be Let statements
    let let_count = ast
        .stmts
        .iter()
        .filter(|s| matches!(s.node, Stmt::Let(_)))
        .count();
    assert_eq!(let_count, ast.stmts.len(), "all math.cov statements should be let bindings");
}

#[test]
fn all_examples_parse_without_errors() {
    let dir = examples_dir();
    let entries: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("failed to read examples dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "cov")
        })
        .collect();

    assert!(
        !entries.is_empty(),
        "no .cov files found in {}",
        dir.display()
    );

    for entry in entries {
        let path = entry.path();
        parse_file(&path); // will panic on errors
    }
}
