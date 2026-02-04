# CLAUDE.md

## Project Overview

COVARIANT — A functional programming language for 3D CAD design, implemented in Rust.

## Implementation Guidelines

- Follow the phases defined in `docs/ROADMAP.md` in order.
- After each implementation step, verify with `cargo build --workspace && cargo test --workspace && cargo clippy --workspace`.
- Refer to `docs/ARCHITECTURE.md` for specification and architectural details.
- If anything is unclear about the spec or implementation, ask the user immediately — do not guess.

## Coding Style

- Prefer functional programming patterns wherever possible:
  - Favor immutable data structures.
  - Use iterator combinators (`iter()`, `map()`, `filter()`, `fold()`, etc.) over imperative loops.
  - Minimize side effects; prefer pure functions.
  - Leverage `enum` + pattern matching.
  - When mutable state is necessary, keep its scope as small as possible.
- Format with `rustfmt`, lint with `clippy`.

## Git Workflow

- Keep commits small and atomic.
  - One logical change = one commit (e.g., "Define token types", "Implement lexer", "Add lexer tests" should be separate commits).
  - Each commit must build successfully.
  - Write concise commit messages describing what changed.

## Reference Project

- [Typst](https://github.com/typst/typst) — Use as a reference for multi-crate workspace structure, parser design, and IR design.
