# COVARIANT

**A functional programming language for 3D CAD design**

![Status](https://img.shields.io/badge/status-design%20phase-blue)
![Version](https://img.shields.io/badge/version-0.1--alpha-orange)

---

## Overview

COVARIANT is a new approach to 3D CAD that treats design as **compilable code** with **first-class engineering semantics**.

Unlike traditional CAD systems that record GUI operations or SCAD-like languages that generate meshes imperatively, COVARIANT:

- ✅ **Preserves design intent** through semantic representation
- ✅ **Supports incremental compilation** for fast iteration
- ✅ **Provides step-by-step debugging** of geometry construction
- ✅ **Treats engineering features** (threads, holes, fillets) as first-class values
- ✅ **Expresses mathematical geometry** naturally (curves, surfaces, sweeps)

---

## Quick Example

```cov
// Define a mounting plate with threaded holes
let plate = box(vec3(80mm, 50mm, 5mm))

let hole = threaded_hole(
  ISO_METRIC, M3, TAP,
  depth = 8mm,
  chamfer = 0.5mm
)

let model = difference(plate,
  union_many([
    move(hole, vec3(10mm, 10mm, 0)),
    move(hole, vec3(70mm, 10mm, 0)),
    move(hole, vec3(70mm, 40mm, 0)),
    move(hole, vec3(10mm, 40mm, 0))
  ])
)

export_stl("plate.stl", model)
```

---

## Core Concepts

### 1. Functional Design

Every design is a **pure expression**:
- No side effects
- Referential transparency
- Composable and reusable

### 2. Engineering Semantics

Engineering features are **not just shapes**:

```cov
// This isn't just a cylinder - it's a specification
threaded_hole(ISO_METRIC, M3, TAP, depth=10mm, chamfer=0.5mm)
```

The system knows:
- Exact thread dimensions from standards (ISO, UTS, etc.)
- Different export modes (preview/cosmetic/full threads)
- Engineering intent for downstream tools

### 3. Mathematical Geometry

Work with curves and surfaces directly:

```cov
let profile = circle2d(radius = 2mm)
let path = helix(radius = 10mm, pitch = 5mm, turns = 3)
let spring = sweep(profile, path)
```

### 4. Debuggable Evaluation

Step through your design construction:

```cov
render_debug(model, step=2)  // See exactly what happens at each step
```

---

## Project Status

**Current Phase:** Language design and specification

- [x] Core language philosophy defined
- [x] Type system designed
- [x] Geometric primitives specified
- [x] Engineering features (threaded holes) designed
- [x] Debug mechanism specified
- [ ] Parser implementation
- [ ] IR (DAG) implementation
- [ ] Geometric kernel integration
- [ ] Preview renderer
- [ ] STL export

---

## Documentation

- **[SPEC.md](SPEC.md)** - Complete language specification
- **[DESIGN.md](docs/DESIGN.md)** - Design decisions and rationale *(planned)*
- **[TUTORIAL.md](docs/TUTORIAL.md)** - Getting started guide *(planned)*

---

## Design Goals

### Primary Goals

1. **Design as Code**: Designs are expressions that can be version-controlled, tested, and composed
2. **Engineering First**: Threaded holes, tolerances, and manufacturing constraints are primitives
3. **Incremental**: Fast recompilation on design changes
4. **Debuggable**: Step-by-step visualization of construction
5. **Mathematical**: Native support for curves, surfaces, and parametric geometry

### Non-goals

- Sculpting/artistic modeling
- Direct mesh manipulation
- GUI operation recording
- Real-time physics

---

## Architecture

```
┌──────────────┐
│ Source Code  │
│   (.cov)     │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│     AST      │
└──────┬───────┘
       │
       ▼
┌──────────────┐     ┌─────────┐
│   IR (DAG)   │────▶│  Cache  │
└──────┬───────┘     └─────────┘
       │
       ▼
┌──────────────┐
│  Evaluation  │
└──────┬───────┘
       │
       ▼
┌──────────────┐
│   Geometry   │
│ (Preview/    │
│  Export)     │
└──────────────┘
```

---

## Building

**Note:** Implementation has not yet started. This is currently a design document.

When implementation begins:

```bash
cargo build --release
cargo run -- example.cov
```

---

## Contributing

This project is in the **design phase**. Contributions to the specification and design are welcome!

Areas of interest:
- Language syntax and semantics
- Geometric kernel selection/integration
- Thread standard databases
- Debug visualization approaches

---

## Acknowledgments

Inspired by:
- **OpenSCAD**: Pioneering code-based CAD
- **CadQuery/Build123d**: Python-based parametric CAD
- **Haskell**: Functional programming principles
- **ImplicitCAD**: Mathematics-first CAD
- **SolidPython**: Parametric design as code
