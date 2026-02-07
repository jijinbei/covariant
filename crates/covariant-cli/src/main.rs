use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};

/// COVARIANT — A functional programming language for 3D CAD design.
#[derive(Parser)]
#[command(name = "covariant", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile and evaluate a .cov file.
    Run {
        /// Path to the .cov source file.
        file: PathBuf,
    },
    /// Parse and check a .cov file without evaluating.
    Check {
        /// Path to the .cov source file.
        file: PathBuf,
    },
    /// Debug a .cov file with step-by-step 3D visualization.
    Debug {
        /// Path to the .cov source file.
        file: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Run { ref file } => run(file),
        Command::Check { ref file } => check(file),
        Command::Debug { ref file } => debug(file),
    };

    if let Err(msg) = result {
        eprintln!("{msg}");
        process::exit(1);
    }
}

/// Read source, parse, lower, and evaluate.
fn run(path: &Path) -> Result<(), String> {
    let source = read_source(path)?;
    let dag = parse_and_lower(&source, path)?;

    let kernel = covariant_geom::TruckKernel;
    covariant_eval::eval(&dag, &kernel).map_err(|e| format_eval_error(&e, &source, path))?;

    Ok(())
}

/// Read source, parse, and lower (no evaluation).
fn check(path: &Path) -> Result<(), String> {
    let source = read_source(path)?;
    let _ = parse_and_lower(&source, path)?;
    eprintln!("ok: {}", path.display());
    Ok(())
}

/// Read source, parse, lower, evaluate with debug step collection, then launch viewer.
fn debug(path: &Path) -> Result<(), String> {
    let source = read_source(path)?;
    let dag = parse_and_lower(&source, path)?;

    let kernel = covariant_geom::TruckKernel;
    let file_path = path.display().to_string();
    let session =
        covariant_debug::eval_debug(&dag, &kernel, source.clone(), file_path)
            .map_err(|e| format_eval_error(&e, &source, path))?;

    eprintln!(
        "Debug session: {} geometry step(s) collected from {}",
        session.step_count(),
        path.display(),
    );
    for step in &session.steps {
        let label = step.label.as_deref().unwrap_or("(unlabeled)");
        eprintln!("  Step {}: {}", step.index + 1, label);
    }

    covariant_debug::launch_viewer(&session, &kernel);
    Ok(())
}

fn read_source(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("error: cannot read '{}': {e}", path.display()))
}

fn parse_and_lower(source: &str, path: &Path) -> Result<covariant_ir::Dag, String> {
    let (ast, parse_errors) = covariant_syntax::parse(source);
    if !parse_errors.is_empty() {
        let mut msg = String::new();
        for err in &parse_errors {
            msg.push_str(&format_syntax_error(err, source, path));
            msg.push('\n');
        }
        return Err(msg);
    }

    let (dag, ir_errors) = covariant_ir::lower(&ast);
    if !ir_errors.is_empty() {
        let mut msg = String::new();
        for err in &ir_errors {
            msg.push_str(&format!(
                "error[IR]: {} (at {}:{})\n",
                err,
                path.display(),
                "?"
            ));
        }
        return Err(msg);
    }

    Ok(dag)
}

/// Format a syntax error with source location.
fn format_syntax_error(
    err: &covariant_syntax::SyntaxError,
    source: &str,
    path: &Path,
) -> String {
    let (line, col) = offset_to_line_col(source, err.span.start);
    format!(
        "error[Syntax]: {} (at {}:{line}:{col})",
        err.message,
        path.display(),
    )
}

/// Format an eval error with source location.
fn format_eval_error(
    err: &covariant_eval::EvalError,
    source: &str,
    path: &Path,
) -> String {
    match err.span {
        Some(span) => {
            let (line, col) = offset_to_line_col(source, span.start);
            format!(
                "error[{}]: {} (at {}:{line}:{col})",
                err.kind,
                err.message,
                path.display(),
            )
        }
        None => format!("error[{}]: {}", err.kind, err.message),
    }
}

/// Convert a byte offset to 1-based (line, column).
fn offset_to_line_col(source: &str, offset: u32) -> (usize, usize) {
    let offset = offset as usize;
    let mut line = 1;
    let mut col = 1;
    for (i, ch) in source.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}
