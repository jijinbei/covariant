# COVARIANT Implementation Roadmap

**Status**: Implementation Phase
**Target**: v0.1 Proof of Concept
**Timeline**: 6-8 weeks

---

## Phase 0: Foundation ✅

**Goal**: Complete language specification and project setup

### Completed Tasks
- [x] Define core language philosophy
- [x] Design type system
- [x] Specify geometric primitives
- [x] Design threading system
- [x] Specify debug mechanism
- [x] Create SPEC.md
- [x] Create README.md
- [x] Initialize Rust project
- [x] Design multi-crate architecture

### Next Steps
- [ ] Complete IR node specification
- [ ] Define thread database schema
- [x] Create workspace structure

**Deliverables**:
- ✅ Complete specification document
- ✅ Project infrastructure design
- ✅ Workspace structure (`covariant-syntax`, `covariant-cli`)
- ⏳ IR node reference

---

## Phase 1: Syntax Crate ✅

**Goal**: Working lexer and parser
**Duration**: 1 week

### 1.1 Foundation
- [x] Create `crates/covariant-syntax`
- [x] Define token types (`token.rs`)
- [x] Define span types (`span.rs`)
- [x] Define error types (`error.rs`)

### 1.2 Lexer
- [x] Implement lexer (`lexer.rs`)
- [x] Keyword recognition
- [x] Number + unit parsing (10mm, 45deg)
- [x] String literals
- [x] Comment handling (line + nested block)
- [x] Error recovery

### 1.3 Parser
- [x] Define AST (`ast.rs`)
- [x] Expression parser (`parser.rs`) — Pratt precedence climbing
- [x] Function calls (positional + named args)
- [x] Let bindings
- [x] Type annotations
- [x] Error recovery
- [x] Span preservation
- [x] Data constructors, with-update, lambda, if/match, list, block expressions

### 1.4 Tests
- [x] Lexer unit tests (26 cases)
- [x] Parser unit tests (35 cases)
- [x] Integration tests with `.cov` fixture files (5 cases)
- [x] Test specification document (`docs/TESTING.md`)

**Deliverables**:
- ✅ Complete hand-written lexer (zero-copy, unit suffixes, nested block comments)
- ✅ Complete recursive descent parser (Pratt precedence climbing)
- ✅ Source position tracking (`Span`, `Spanned<T>`)
- ✅ 71 tests passing (66 unit + 5 integration)
- ✅ 4 example `.cov` fixture files

---

## Phase 2: Thread Crate ✅

**Goal**: Thread standards database
**Duration**: 3-4 days

### 2.1 Data Structures
- [x] Create `crates/covariant-thread`
- [x] Define ThreadSpec type
- [x] Define ThreadStandard enum
- [x] Define ThreadSize enum (15 ISO + 13 UTS)
- [x] Define ThreadKind enum
- [x] Define ClearanceFit enum

### 2.2 Standards Data
- [x] ISO Metric data (M1.6–M30, 15 sizes)
  - [x] Embedded Rust data via match arms (no JSON/serde)
  - [x] M3, M4, M5 initial data
  - [x] All 15 sizes complete
- [x] UTS data (#2-56 through 3/4"-10, 13 sizes, all in mm)
- [ ] BSW data (future — enum variant exists, lookup panics)

### 2.3 Dimension Calculations
- [x] Tap hole diameter calculation
- [x] Clearance hole diameter calculation (close/medium/free)
- [x] Insert hole diameter calculation
- [x] Chamfer dimensions (45-degree)
- [x] Unit tests for all calculations

### 2.4 Geometry Generation
- [x] Simple representation (CylinderParams)
- [x] Cosmetic representation (CosmeticAnnotation)
- [x] Full thread geometry (HelixParams)

**Deliverables**:
- ✅ Thread standards database (embedded Rust, zero deps)
- ✅ Dimension calculation API (`get_dimensions`, `hole_diameter`, `clearance_hole_diameter`, `chamfer_dimensions`)
- ✅ Geometry generation API (`generate_thread_geometry` → `ThreadGeometry`)
- ✅ 48 unit tests + 8 integration tests passing
- ✅ `string_enum!` macro for Display/FromStr/ALL boilerplate

---

## Phase 3: IR Crate ✅

**Goal**: Intermediate representation and DAG
**Duration**: 1 week

### 3.1 IR Definition
- [x] Create `crates/covariant-ir`
- [x] Define IR node types (`node.rs`) — 20 IrNode variants, 5 supporting types
- [x] Define DAG structure (`dag.rs`) — arena-allocated with insert/get/iter
- [x] Node ID management — `NodeId(u32)` newtype
- [x] Span preservation

### 3.2 AST → IR Lowering
- [x] Implement lowering (`lower.rs`)
- [x] Pipe desugaring (`|>` → `FnCall` with prepended arg)
- [x] Grouped elimination (parentheses unwrapped)
- [x] Preserve type annotations
- [x] Preserve source spans
- [x] Error handling

### 3.3 Incremental Compilation
- [ ] Node hashing (`hash.rs`) — deferred to Phase 5 with `comemo`
- [ ] Cache system (`cache.rs`) — deferred to Phase 5
- [ ] DAG diffing (`diff.rs`) — deferred to Phase 5
- [ ] Cache invalidation — deferred to Phase 5

**Deliverables**:
- ✅ Complete IR definition (20 node variants, thin IR mirroring AST)
- ✅ AST → IR lowering with pipe desugaring and grouped elimination
- ✅ 45 unit tests + 8 integration tests passing
- ⏳ Incremental compilation deferred to Phase 5

---

## Phase 4: Geometry Crate

**Goal**: Geometry kernel integration
**Duration**: 1-2 weeks

### 4.1 Kernel Selection

**Candidates**:
- **truck**: Pure Rust, actively developed, good curve/surface support
- **opencascade-rs**: Industry standard OCCT, C++ bindings, mature
- **manifold**: High-performance booleans, C++ bindings, fast

**Selection criteria**:
- Boolean operation robustness
- Curve/surface support
- Ease of Rust integration
- License compatibility
- Community/maintenance

**Decision**: [TBD after evaluation]

### 4.2 Abstraction Layer
- [ ] Create `crates/covariant-geom`
- [ ] Define GeomKernel trait (`kernel.rs`)
- [ ] Define Vec3, Length, Angle types (`types.rs`)
- [ ] Define Solid, Surface, Curve types (`types.rs`)

### 4.3 Primitives
- [ ] Implement box
- [ ] Implement cylinder
- [ ] Implement sphere
- [ ] Implement circle2d
- [ ] Implement rectangle2d

### 4.4 Boolean Operations
- [ ] Implement union
- [ ] Implement difference
- [ ] Implement intersection
- [ ] Error handling for degenerate cases
- [ ] Tests with known-good cases

### 4.5 Transformations
- [ ] Implement move (translation)
- [ ] Implement rotate
- [ ] Implement scale
- [ ] Implement mirror

### 4.6 Advanced Shape Generation
- [ ] Define Curve type
- [ ] Define Surface type
- [ ] Implement sweep operation
- [ ] Implement loft operation
- [ ] Implement revolve operation

**Deliverables**:
- Geometry kernel abstraction
- Basic primitives
- Boolean operations
- Transformations
- Advanced shape generation
- Comprehensive tests

---

## Phase 5: Eval Crate

**Goal**: Type checking and evaluation
**Duration**: 1 week

### 5.1 Type System
- [ ] Create `crates/covariant-eval`
- [ ] Define type representation (`types.rs`)
- [ ] Implement type inference (`infer.rs`)
- [ ] Implement unit checking (`units.rs`)
- [ ] Type error messages

### 5.2 Evaluator
- [ ] Define environment/symbol table (`env.rs`)
- [ ] Define value types (`value.rs`)
- [ ] Implement evaluator (`eval.rs`)
- [ ] Function call resolution
- [ ] Error handling

### 5.3 Built-in Functions
- [ ] Geometric primitives
- [ ] Boolean operations
- [ ] Transformation functions
- [ ] threaded_hole
- [ ] trace
- [ ] with_preview_quality
- [ ] with_export_quality

**Deliverables**:
- Type checker
- Evaluator
- Built-in functions
- Clear error messages

---

## Phase 6: Export Crate

**Goal**: STL export
**Duration**: 3-4 days

### 6.1 Tessellation
- [ ] Create `crates/covariant-export`
- [ ] Define tessellation parameters
- [ ] Implement preview quality settings
- [ ] Implement export quality settings
- [ ] Quality control API

### 6.2 STL Output
- [ ] Implement binary STL writer (`stl.rs`)
- [ ] Implement ASCII STL writer
- [ ] Normal calculation
- [ ] Mesh validation
- [ ] Mesh optimization (optional)

### 6.3 Thread Hole Expansion
- [ ] No-thread mode
- [ ] Cosmetic mode
- [ ] Full thread mode
- [ ] Tests for each mode

**Deliverables**:
- STL export (binary and ASCII)
- Quality control system
- Thread rendering modes
- Valid STL output verified with external tools

---

## Phase 7: Debug Crate

**Goal**: Debug visualization
**Duration**: 1 week

### 7.1 Trace System
- [ ] Create `crates/covariant-debug`
- [ ] Implement trace annotations (`trace.rs`)
- [ ] Implement step tracking (`stepper.rs`)
- [ ] DAG linearization
- [ ] Source span mapping

### 7.2 Visualization

**Rendering backend candidates**:
- **three-d**: Simple, pure Rust, good for prototyping
- **bevy**: Full game engine, ECS architecture, more complex
- **kiss3d**: Minimal, simple API, good for basic 3D

**Decision**: Start with three-d or kiss3d for simplicity

- [ ] Choose rendering backend
- [ ] Implement basic 3D viewer (`viewer.rs`)
- [ ] Implement step-by-step rendering
- [ ] Implement current node highlighting
- [ ] Implement source code highlighting (terminal-based)
- [ ] Camera controls

**Deliverables**:
- Step execution system
- Debug visualization
- Interactive viewer
- Source highlighting

---

## Phase 8: CLI Crate

**Goal**: Usable command-line tool
**Duration**: 2-3 days

### 8.1 CLI Interface
- [x] Create `crates/covariant-cli` (skeleton)
- [ ] Argument parsing with clap
- [ ] File loading
- [ ] Error formatting for terminal
- [ ] Progress indication
- [ ] Colored output

### 8.2 Commands
- [ ] `covariant compile <file.cov>`
- [ ] `covariant export <file.cov> -o output.stl`
- [ ] `covariant debug <file.cov>`
- [ ] `covariant check <file.cov>` (type check only)

### 8.3 REPL (Optional)
- [ ] Interactive mode
- [ ] Live preview
- [ ] Command history
- [ ] Tab completion

**Deliverables**:
- Functional CLI tool
- All planned commands working
- User-friendly error messages

---

## Milestone: v0.1 Release

**Minimum Viable Product**:
- [x] Complete specification
- [x] Working parser
- [ ] Type checker
- [ ] Basic primitives (box, cylinder, sphere)
- [ ] Boolean operations (union, difference, intersection)
- [ ] Threaded holes (ISO Metric M3, M4, M5 minimum)
- [ ] STL export
- [ ] Basic debug visualization
- [ ] CLI tool

**Success Criteria**:
1. Can compile and render the mounting plate example from SPEC.md
2. Can step through evaluation with visualization
3. Can export valid STL file
4. Error messages include source locations
5. Documentation is complete

**Target Date**: 6-8 weeks from start of implementation

---

## Future Phases (v0.2+)

### Phase 9: STEP Export (v0.2)
- STEP file format
- B-rep preservation
- Thread annotations
- Interoperability with commercial CAD

### Phase 10: Constraint System (v0.2)
- Constraint DSL
- Geometric solver integration
- Assembly mates
- Over/under-constrained detection

### Phase 11: GUI (v0.3)
- Visual programming interface
- Live coding with preview
- AST editor
- Graphical debugging

### Phase 12: Analysis (v0.3)
- FEM integration
- Stress analysis
- Mass properties
- Topology optimization

---

## Development Principles

### Iteration Cycle
1. Design feature (update SPEC.md)
2. Implement in Rust
3. Write tests
4. Write examples
5. Update documentation

### Testing Strategy
- **Unit tests**: Each module has comprehensive tests
- **Integration tests**: Cross-crate functionality
- **Golden file tests**: Regression testing with snapshots
- **Example files**: Serve as integration tests

### Code Quality
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Maintain rustdoc comments for public APIs
- Keep internal docs updated

---

## Dependencies to Evaluate

### Required
- **clap** (4.x): CLI argument parsing
- **serde** (1.x): Serialization (thread database, config)
- **serde_json** (1.x): JSON parsing
- **thiserror** (1.x): Error handling

### Geometric Kernel (choose one)
- **truck** (0.4.x): Pure Rust CAD kernel
- **opencascade-rs**: OCCT bindings
- **manifold**: Fast boolean operations

### Rendering (choose one)
- **three-d** (0.16.x): Simple 3D rendering
- **kiss3d** (0.35.x): Minimal 3D engine
- **bevy** (0.12.x): Full game engine (overkill?)

### Optional
- ~~**logos** (0.13.x): Fast lexer generator~~ (not needed — hand-written lexer implemented)
- **codespan-reporting** (0.11.x): Beautiful error diagnostics
- **rustyline** (13.x): REPL with history
- **insta** (1.x): Snapshot testing

---

## Weekly Breakdown (Estimated)

**Week 1**: Syntax crate + Thread crate foundation
**Week 2**: IR crate + Thread crate completion
**Week 3**: Geometry crate (kernel integration)
**Week 4**: Eval crate
**Week 5**: Export crate + Debug crate foundation
**Week 6**: Debug crate completion + CLI
**Week 7**: Integration, testing, bug fixes
**Week 8**: Documentation, examples, polish

---

**Last Updated**: 2026-02-07
**Status**: Phase 0 + Phase 1 + Phase 2 + Phase 3 complete, ready to begin Phase 4
**Next Action**: Create `covariant-geom` crate and evaluate geometry kernel
