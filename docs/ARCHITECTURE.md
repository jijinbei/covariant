# COVARIANT Architecture

**Multi-crate project structure**

---

## Overview

COVARIANT follows a multi-crate workspace structure inspired by Typst. This design provides clear module boundaries, enables parallel compilation, and makes testing easier.

---

## Crate Structure

```
covariant/
├── crates/
│   ├── covariant-cli/        # Command-line interface
│   ├── covariant-syntax/     # Lexer, parser, AST
│   ├── covariant-ir/         # Intermediate representation (DAG)
│   ├── covariant-eval/       # Evaluator and type checker
│   ├── covariant-geom/       # Geometry kernel abstraction
│   ├── covariant-thread/     # Thread standards database
│   ├── covariant-export/     # Export formats (STL, etc.)
│   └── covariant-debug/      # Debug visualization
├── data/
│   └── threads/              # Thread specification data (JSON)
├── examples/                 # Example .cov files
├── tests/                    # Integration tests
├── docs/                     # Documentation
└── Cargo.toml                # Workspace definition
```

---

## Dependency Graph

```
covariant-cli
    ├─→ covariant-syntax
    ├─→ covariant-eval
    ├─→ covariant-export
    └─→ covariant-debug

covariant-debug
    └─→ covariant-ir

covariant-export
    ├─→ covariant-ir
    ├─→ covariant-geom
    └─→ covariant-thread

covariant-eval
    ├─→ covariant-syntax
    ├─→ covariant-ir
    └─→ covariant-geom

covariant-ir
    └─→ covariant-syntax

covariant-geom
    └─→ covariant-thread

covariant-thread
    (no internal deps)

covariant-syntax
    (no internal deps)
```

**Dependency principles**:
- No circular dependencies
- Foundation crates have no internal dependencies
- CLI crate is the top-level integrator

---

## Crate Details

### `covariant-syntax`

**Responsibility**: Lexical analysis, parsing, AST definition

**Key APIs**:
```rust
pub fn lex(source: &str) -> Result<Vec<Token>>
pub fn parse(tokens: Vec<Token>) -> Result<Ast>

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub struct Ast {
    pub root: Expr,
    pub source: SourceId,
}
```

**Internal modules**:
- `token.rs` - Token types
- `lexer.rs` - Lexical analyzer
- `parser.rs` - Recursive descent parser
- `ast.rs` - AST node definitions
- `span.rs` - Source location tracking
- `error.rs` - Syntax error types

**Dependencies**: None (foundation crate)

---

### `covariant-thread`

**Responsibility**: Thread standards database and hole geometry

**Key APIs**:
```rust
pub struct ThreadSpec {
    pub standard: ThreadStandard,
    pub size: ThreadSize,
    pub kind: ThreadKind,
    pub depth: Length,
    pub chamfer: Length,
}

pub fn get_dimensions(spec: &ThreadSpec) -> ThreadDimensions
pub fn generate_thread_geometry(
    spec: &ThreadSpec,
    mode: ThreadMode
) -> Geometry
```

**Data files**:
- `data/threads/iso_metric.json`
- `data/threads/uts.json`
- `data/threads/bsw.json`

**Internal modules**:
- `standard.rs` - Thread standard enums
- `spec.rs` - ThreadSpec definition
- `iso_metric.rs` - ISO Metric data
- `uts.rs` - UTS data
- `dimensions.rs` - Dimension calculations
- `geometry.rs` - Thread geometry generation

**Dependencies**: None (foundation crate)

---

### `covariant-ir`

**Responsibility**: Intermediate representation and DAG construction

**Key APIs**:
```rust
pub enum IrNode {
    Primitive(PrimitiveKind),
    Boolean(BoolOp, NodeId, NodeId),
    Transform(TransformKind, NodeId),
    ThreadedHole(ThreadSpec),
    Trace(String, NodeId),
    // ...
}

pub struct Dag {
    nodes: Arena<IrNode>,
    cache: Cache,
}

pub fn lower(ast: &Ast) -> Result<Dag>
pub fn diff(old: &Dag, new: &Dag) -> DagDiff
```

**Internal modules**:
- `node.rs` - IR node types
- `dag.rs` - DAG structure
- `lower.rs` - AST → IR lowering
- `hash.rs` - Node hashing for cache keys
- `cache.rs` - Evaluation cache
- `diff.rs` - DAG diffing algorithm

**Dependencies**: `covariant-syntax`

---

### `covariant-geom`

**Responsibility**: Geometry kernel abstraction layer

**Key APIs**:
```rust
pub trait GeomKernel {
    fn box(&self, size: Vec3) -> Solid;
    fn cylinder(&self, radius: f64, height: f64) -> Solid;
    fn sphere(&self, radius: f64) -> Solid;

    fn union(&self, a: &Solid, b: &Solid) -> Result<Solid>;
    fn difference(&self, a: &Solid, b: &Solid) -> Result<Solid>;
    fn intersect(&self, a: &Solid, b: &Solid) -> Result<Solid>;

    fn sweep(&self, profile: &Surface, path: &Curve) -> Result<Solid>;
    fn loft(&self, sections: &[Surface]) -> Result<Solid>;
}

pub struct TruckKernel;   // Implementation using truck
pub struct OccKernel;     // Implementation using OpenCASCADE
```

**Kernel candidates**:
- **truck**: Pure Rust, actively developed
- **opencascade-rs**: OCCT bindings, industry standard
- **manifold**: High-performance booleans, C++ bindings

**Internal modules**:
- `kernel.rs` - GeomKernel trait
- `types.rs` - Vec3, Length, Angle, Solid, Surface, Curve
- `primitives.rs` - Primitive construction
- `boolean.rs` - Boolean operations
- `transform.rs` - Transformations
- `sweep.rs` - Sweep operations
- `truck_impl.rs` - Truck implementation (or `occ_impl.rs`)

**Dependencies**: `covariant-thread`

---

### `covariant-eval`

**Responsibility**: Type checking, type inference, evaluation

**Key APIs**:
```rust
pub struct TypeChecker {
    // ...
}

pub struct Evaluator {
    env: Env,
    geom: Box<dyn GeomKernel>,
}

pub fn type_check(dag: &Dag) -> Result<TypedDag>
pub fn eval(dag: &Dag, geom: Box<dyn GeomKernel>) -> Result<Value>
```

**Internal modules**:
- `types.rs` - Type definitions
- `infer.rs` - Type inference
- `units.rs` - Unit checking
- `env.rs` - Environment (symbol table)
- `value.rs` - Runtime value types
- `eval.rs` - Evaluator implementation
- `builtins.rs` - Built-in functions

**Dependencies**: `covariant-syntax`, `covariant-ir`, `covariant-geom`

---

### `covariant-export`

**Responsibility**: Mesh tessellation and export

**Key APIs**:
```rust
pub struct ExportOptions {
    pub quality: Quality,
    pub thread_mode: ThreadMode,
    pub tolerance: f64,
}

pub fn export_stl(
    solid: &Solid,
    path: &Path,
    options: ExportOptions
) -> Result<()>

pub fn export_step(solid: &Solid, path: &Path) -> Result<()>  // Future
```

**Internal modules**:
- `mesh.rs` - Tessellation
- `quality.rs` - Quality settings
- `stl.rs` - STL writer (binary/ASCII)
- `step.rs` - STEP writer (future)

**Dependencies**: `covariant-ir`, `covariant-geom`, `covariant-thread`

---

### `covariant-debug`

**Responsibility**: Debug visualization and step execution

**Key APIs**:
```rust
pub struct DebugSession {
    dag: Dag,
    current_step: usize,
}

pub fn render_debug(dag: &Dag, step: usize) -> DebugView
pub fn step_forward(session: &mut DebugSession)
pub fn step_backward(session: &mut DebugSession)
pub fn highlight_node(node: NodeId) -> Highlight
```

**Internal modules**:
- `trace.rs` - Trace annotations
- `stepper.rs` - Step-by-step execution
- `viewer.rs` - 3D viewer
- `render.rs` - Rendering backend
- `highlight.rs` - Node/source highlighting

**Dependencies**: `covariant-ir`, rendering library (three-d/bevy/kiss3d)

---

### `covariant-cli`

**Responsibility**: Command-line interface

**Key APIs**:
```rust
pub fn main() -> Result<()>
pub fn run_file(path: &Path) -> Result<()>
pub fn run_repl() -> Result<()>
```

**Commands**:
```bash
covariant compile <file.cov>
covariant export <file.cov> -o output.stl
covariant debug <file.cov>
covariant check <file.cov>  # Type check only
```

**Internal modules**:
- `main.rs` - Entry point
- `cli.rs` - CLI argument parsing
- `repl.rs` - Interactive REPL

**Dependencies**: All other crates, `clap`

---

## Compilation Flow

```
.cov source file
    │
    ▼
┌─────────────────────┐
│  covariant-syntax   │
│  lex() → parse()    │
└──────────┬──────────┘
           │ AST
           ▼
┌─────────────────────┐
│   covariant-ir      │
│     lower()         │
└──────────┬──────────┘
           │ DAG
           ▼
┌─────────────────────┐    ┌─────────┐
│  covariant-eval     │───▶│  Cache  │
│  type_check()       │    └─────────┘
│  eval()             │
└──────────┬──────────┘
           │ Geometry (Solid)
           ▼
┌─────────────────────┐
│ covariant-export    │
│  tessellate()       │
│  export_stl()       │
└─────────────────────┘
           │
           ▼
        output.stl
```

---

## Benefits of Multi-crate Structure

### 1. Parallel Compilation
Independent crates can be compiled in parallel, reducing overall build time.

### 2. Clear API Boundaries
Each crate has an explicit public API defined in `lib.rs`. Internal implementation details are truly private.

### 3. Incremental Compilation
Only changed crates and their dependents need to recompile.

### 4. Reusability
External projects can depend on individual crates:
- `covariant-thread`: Thread database library
- `covariant-geom`: CAD kernel abstraction
- `covariant-syntax`: Parser for tooling

### 5. Easier Testing
- Unit tests per crate
- Integration tests between crates
- Easier to mock dependencies

### 6. Better Documentation
Each crate generates its own rustdoc with clear dependency information.

---

## Development Workflow

### Build Commands

```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p covariant-syntax

# Test specific crate
cargo test -p covariant-eval

# Generate docs for specific crate
cargo doc -p covariant-geom --open

# Run CLI
cargo run --bin covariant-cli -- example.cov
```

### Development Cycle

1. **Make changes** to a crate
2. **Run tests** for that crate: `cargo test -p <crate>`
3. **Run integration tests**: `cargo test --workspace`
4. **Check dependents**: Ensure downstream crates still compile
5. **Update docs**: Keep rustdoc comments current

---

## Implementation Order

**Recommendation**: Implement in dependency order (bottom-up)

1. **Foundation crates** (no dependencies):
   - `covariant-syntax`
   - `covariant-thread`

2. **Core crates**:
   - `covariant-ir` (depends on syntax)
   - `covariant-geom` (depends on thread)

3. **High-level crates**:
   - `covariant-eval` (depends on syntax, ir, geom)
   - `covariant-export` (depends on ir, geom, thread)
   - `covariant-debug` (depends on ir)

4. **Integration crate**:
   - `covariant-cli` (depends on all)

---

## Workspace Configuration

### Root `Cargo.toml`

```toml
[workspace]
members = [
    "crates/covariant-cli",
    "crates/covariant-syntax",
    "crates/covariant-ir",
    "crates/covariant-eval",
    "crates/covariant-geom",
    "crates/covariant-thread",
    "crates/covariant-export",
    "crates/covariant-debug",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/covariant"
authors = ["Your Name <you@example.com>"]

[workspace.dependencies]
# Shared dependencies
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
```

### Example Crate `Cargo.toml`

```toml
# crates/covariant-syntax/Cargo.toml
[package]
name = "covariant-syntax"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
logos = "0.13"
codespan-reporting = "0.11"
```

---

**Status**: Design complete, ready for implementation
**Last Updated**: 2026-01-30
