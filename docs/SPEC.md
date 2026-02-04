# COVARIANT Language Specification

**Version 0.1**

A covariant, functional language for 3D design that treats design intent as first-class values.

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [Language Model](#2-language-model)
3. [Type System](#3-type-system)
4. [Geometric Primitives](#4-geometric-primitives)
5. [Transformations and Composition](#5-transformations-and-composition)
6. [Engineering Features](#6-engineering-features)
7. [Preview and Export](#7-preview-and-export)
8. [Incremental Compilation](#8-incremental-compilation)
9. [Debug Mechanism](#9-debug-mechanism)
10. [Examples](#10-examples)
11. [Future Extensions](#11-future-extensions)

---

## 1. Philosophy

### 1.1 Problem Statement

Existing 3D CAD and SCAD-like languages suffer from:

- Design intent buried in mesh representation
- Lack of engineering primitives (e.g., threaded holes)
- Opaque boolean operation failures
- Full recomputation on small changes
- Inability to naturally express mathematical geometry (curves, surfaces)

### 1.2 Core Principles

**COVARIANT is designed around these principles:**

1. **Design is an expression** - Designs are values, not procedures
2. **Meaning over shape** - Engineering intent is the primary representation
3. **Pure functional** - No side effects, referential transparency
4. **Debuggable evaluation** - Every step can be visualized
5. **Covariance** - Representation-independent, resolution-independent

### 1.3 Non-goals

- Sculpting and artistic modeling
- Imperative GUI operation recording
- Direct mesh editing
- Real-time physics simulation

---

## 2. Language Model

### 2.1 Evaluation Model

- Expression-based language
- All expressions return values
- Internal representation: **DAG (Directed Acyclic Graph)**
- Referential transparency guaranteed

### 2.2 Execution Model

```
Source Code → AST → IR (DAG) → Evaluation → Geometry
                      ↓
                   Cache
```

---

## 3. Type System

### 3.1 Basic Types

```rust
// Primitive types with units
Length      // e.g., 10mm, 2.5in
Angle       // e.g., 45deg, PI/4rad
Vec2        // 2D vector
Vec3        // 3D vector

// Geometric types
Curve       // 1D parametric curve
Surface     // 2D manifold
Solid       // 3D solid volume

// Standard types
Int
Float
Bool
String
```

### 3.2 Unit System

- `Length` and `Angle` are unit-aware types
- Unit mismatch is a compile-time error
- Implicit conversions forbidden

```cov
let a = 10mm + 5mm      // OK: 15mm
let b = 10mm + 2in      // OK: automatic unit conversion within Length
let c = 10mm + 5deg     // ERROR: incompatible types
```

### 3.3 Type Inference

- Type inference by default
- Optional type annotations
- Recommended for API boundaries and ambiguous expressions

```cov
let x = 10mm                    // Inferred: Length
let y: Vec3 = vec3(1, 2, 3)     // Explicit annotation
```

### 3.4 User-Defined Data Types

COVARIANT supports user-defined data structures (records) to group related values together.

#### Definition Syntax

```cov
data TypeName {
  field1: Type1,
  field2: Type2
}
```

#### Example: Geometric Parameters

```cov
// Define data structures
data Rectangle { width: Length, height: Length }
data Circle { radius: Length }
data HolePattern {
  positions: List[Vec2],
  diameter: Length,
  depth: Length
}

// Construction
let plate = Rectangle { width = 60mm, height = 120mm }
let mounting_holes = HolePattern {
  positions = [vec2(10mm, 10mm), vec2(50mm, 10mm)],
  diameter = 5mm,
  depth = 8mm
}
```

#### Field Access

```cov
plate.width      // 60mm
plate.height     // 120mm
```

#### Default Values

Fields can have default values, making them optional during construction:

```cov
data Rectangle {
  width: Length = 10mm,
  height: Length = 10mm
}

let r1 = Rectangle { width = 50mm }           // height = 10mm (default)
let r2 = Rectangle { width = 50mm, height = 80mm }
```

#### Immutable Update (with-syntax)

Since all values are immutable, updating a field creates a new value:

```cov
let r1 = Rectangle { width = 50mm, height = 100mm }
let r2 = r1 with { height = 200mm }    // r1 unchanged, r2 has new height

// Multiple field updates
let r3 = r1 with { width = 60mm, height = 120mm }
```

#### Nested Data Structures

```cov
data BoltHole {
  position: Vec3,
  thread: ThreadSpec
}

data MountingPlate {
  size: Rectangle,
  thickness: Length,
  holes: List[BoltHole]
}

let plate = MountingPlate {
  size = Rectangle { width = 80mm, height = 50mm },
  thickness = 5mm,
  holes = [
    BoltHole {
      position = vec3(10mm, 10mm, 0),
      thread = ThreadSpec { standard = ISO_METRIC, size = M3, kind = TAP }
    }
  ]
}
```

#### Functions with Data Types

```cov
fn make_plate(rect: Rectangle, thickness: Length) -> Solid {
  box(vec3(rect.width, rect.height, thickness))
}

fn add_hole_pattern(base: Solid, pattern: HolePattern) -> Solid {
  let holes = pattern.positions
    |> map(|pos| cylinder(radius = pattern.diameter / 2, height = pattern.depth)
                 |> move(vec3(pos.x, pos.y, 0)))
    |> union_many
  difference(base, holes)
}

// Usage
let result = make_plate(plate, 5mm)
  |> add_hole_pattern(mounting_holes)
```

#### Benefits over Variable-based Approach

| Aspect | Variables | Data Structures |
|--------|-----------|-----------------|
| Clarity | `width`, `height` ambiguous | `plate.width` explicit |
| Reuse | Copy-paste values | `let plate2 = plate` |
| Functions | `fn(w, h, d, x, y, ...)` | `fn(rect, pattern)` |
| Type safety | None | `Rectangle` ≠ `Circle` |
| Refactoring | Change all call sites | Change struct definition |

---

## 4. Geometric Primitives

### 4.1 3D Solids

```cov
box(size: Vec3) : Solid
cylinder(radius: Length, height: Length) : Solid
sphere(radius: Length) : Solid
```

### 4.2 2D Shapes (Surfaces)

```cov
circle2d(radius: Length) : Surface
rectangle2d(size: Vec2) : Surface
polygon2d(points: List[Vec2]) : Surface
```

### 4.3 1D Curves

```cov
// Basic curves
line(p0: Vec3, p1: Vec3) : Curve
circle(center: Vec3, radius: Length, normal: Vec3) : Curve
arc(center: Vec3, radius: Length, start_angle: Angle, end_angle: Angle) : Curve

// Parametric curve
curve(f: Angle -> Vec3, t_min: Angle, t_max: Angle) : Curve
```

**Important:** Curves are one-dimensional geometric objects. They cannot be used as solids directly.

---

## 5. Transformations and Composition

### 5.1 Transformations

All transformations are pure functions returning new values:

```cov
move(s: Solid, v: Vec3) : Solid
rotate(s: Solid, axis: Vec3, angle: Angle) : Solid
scale(s: Solid, factor: Float) : Solid
mirror(s: Solid, plane: Plane) : Solid
```

### 5.2 Boolean Operations

```cov
union(a: Solid, b: Solid) : Solid
difference(a: Solid, b: Solid) : Solid
intersect(a: Solid, b: Solid) : Solid

// Bulk operations
union_many(solids: List[Solid]) : Solid
```

### 5.3 Generative Operations

```cov
// Sweep a 2D profile along a 1D curve
sweep(profile: Surface, along: Curve) : Solid

// Loft between multiple 2D sections
loft(sections: List[Surface]) : Solid

// Revolve a 2D profile around an axis
revolve(profile: Surface, axis: Vec3, angle: Angle) : Solid
```

**Type constraint:** `sweep` requires a `Curve`, not a `Surface` or `Solid`.

---

## 6. Engineering Features

### 6.1 Threaded Holes (First-class Feature)

Threaded holes are **engineering specifications**, not just geometry:

```cov
threaded_hole(
  standard: ThreadStandard,   // ISO_METRIC, UTS, etc.
  size: ThreadSize,          // M3, M5, #10-24, etc.
  kind: ThreadKind,          // tap | clearance | insert
  depth: Length,
  chamfer: Length
) : Solid
```

#### Thread Standards

```cov
enum ThreadStandard {
  ISO_METRIC,
  UTS,           // Unified Thread Standard
  BSW,           // British Standard Whitworth
  METRIC_FINE
}

enum ThreadSize {
  M3, M4, M5, M6, M8, M10, M12, ...
  UTS_10_24, UTS_1_4_20, ...
}

enum ThreadKind {
  TAP,        // Threaded (tapping)
  CLEARANCE,  // Clearance hole for bolt
  INSERT      // Heat-set insert hole
}
```

#### Internal Representation

- Stores `ThreadSpec` metadata
- Preview: simplified cylinder representation
- Export: choice of thread rendering (none/cosmetic/full)

#### Example

```cov
let tap_hole = threaded_hole(
  ISO_METRIC, M3, TAP,
  depth = 10mm,
  chamfer = 0.5mm
)

let clearance_hole = threaded_hole(
  ISO_METRIC, M3, CLEARANCE,
  depth = 5mm,
  chamfer = 0.3mm
)
```

---

## 7. Preview and Export

### 7.1 Export Formats

```cov
export_stl(filename: String, solid: Solid)
export_step(filename: String, solid: Solid)   // Future
```

### 7.2 Thread Export Modes

- `NONE`: No thread geometry (fastest)
- `COSMETIC`: Annotation/metadata only (for CAD software)
- `FULL`: Complete helical thread geometry (for rendering)

---

## 8. Incremental Compilation

### 8.1 IR Structure

The intermediate representation is a DAG where:

- **Nodes** = function calls or operations
- **Edges** = data dependencies

Each node contains:
- Input hash (for cache invalidation)
- Output cache (computed geometry)
- Source span (for debugging)

### 8.2 Incremental Re-evaluation

When source code changes:

1. Compute AST diff
2. Find changed nodes in DAG
3. Re-evaluate only downstream nodes
4. Reuse cached upstream results

### 8.3 Evaluation Stages

```
Parse → Type Check → IR Build → Evaluation → Render
                                    ↑
                                  Cache
```

Preview mode can stop evaluation early for interactive feedback.

---

## 9. Debug Mechanism

### 9.1 Trace Annotations

```cov
trace(label: String, value: Solid) : Solid
```

The `trace` function:
- Does not modify the value
- Attaches metadata for debugging
- Creates a named checkpoint in the evaluation graph

### 9.2 Step-by-step Rendering

```cov
render_debug(solid: Solid, step: Int)
```

**Behavior:**
1. Linearize DAG in topological order
2. Render up to step `step`
3. Highlight current node with glow/outline
4. Highlight corresponding source code range

### 9.3 Execution Order

- Topological order (dependencies first)
- Ties broken by source code order
- High-level operations (union/difference) are single steps

### 9.4 Example Debug Session

```cov
let base = trace("base", box(vec3(50mm, 50mm, 5mm)))
let hole1 = trace("hole1", cylinder(radius=2mm, height=10mm))
let hole2 = trace("hole2", move(hole1, vec3(10mm, 0, 0)))

let result = difference(base, union(hole1, hole2))

render_debug(result, step=0)  // Shows only base
render_debug(result, step=1)  // Shows base + hole1
render_debug(result, step=2)  // Shows base + hole1 + hole2
render_debug(result, step=3)  // Shows final result
```

---

## 10. Examples

### 10.1 Simple Plate with Threaded Holes

```cov
let plate = box(vec3(80mm, 50mm, 5mm))

let hole = threaded_hole(
  ISO_METRIC, M3, TAP,
  depth = 8mm,
  chamfer = 0.5mm
)

// Create hole pattern
let holes = union_many([
  move(hole, vec3(10mm, 10mm, 0)),
  move(hole, vec3(70mm, 10mm, 0)),
  move(hole, vec3(70mm, 40mm, 0)),
  move(hole, vec3(10mm, 40mm, 0))
])

let model = difference(plate, holes)

export_stl("mounting_plate.stl", model)
```

### 10.2 Torus (Mathematical Generation)

```cov
// Define the profile (small circle)
let profile = circle2d(radius = 2mm)

// Define the path (large circle in 3D)
let path = circle(
  center = vec3(0, 0, 0),
  radius = 20mm,
  normal = vec3(0, 0, 1)
)

// Sweep to create torus
let torus = sweep(profile = profile, along = path)
```

### 10.3 Parametric Spiral

```cov
// Parametric helix curve
let helix = curve(
  f = |t| vec3(
    10mm * cos(t),
    10mm * sin(t),
    t * 2mm / (2*PI)
  ),
  t_min = 0deg,
  t_max = 720deg  // Two full turns
)

let profile = circle2d(radius = 1mm)
let spring = sweep(profile, helix)
```

---

## 11. Future Extensions

### 11.1 Planned Features (v0.2+)

- **Constraint DSL**: Geometric constraints (coincident, parallel, distance, tangent)
- **SDF-native IR**: Signed Distance Field backend for organic shapes
- **STEP export**: Proper CAD interchange format
- **G-code generation**: Direct CNC output
- **Assembly constraints**: Multi-part assemblies with mates

### 11.2 Advanced Features (v0.3+)

- **GUI = AST Editor**: Visual programming interface
- **FEM integration**: Structural analysis nodes
- **Parametric tables**: Design variants and optimization
- **Version control integration**: Semantic diff for CAD

### 11.3 Research Directions

- **Automatic feature recognition**: Extract engineering intent from meshes
- **Constraint solver integration**: Declarative geometric constraints
- **Topology optimization**: Generative design
- **Multi-material support**: Heterogeneous objects

---

## Summary

> **COVARIANT is a functional programming language for 3D CAD that:**
> - Treats design as compilable code
> - Makes engineering intent explicit
> - Provides step-by-step debuggable evaluation
> - Supports incremental compilation
> - Separates mathematical geometry from mesh representation

---

**Next Steps for Implementation:**

1. Complete IR node specification
2. Implement AST/DAG in Rust
3. Create thread specification database (ISO, UTS standards)
4. Build debug visualization UI
5. Implement basic geometric kernel (or integrate existing library)

**Design Status:** Specification phase
**Implementation:** Not started
**Target:** v0.1 proof of concept
