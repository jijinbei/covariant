# COVARIANT Test Specifications

**Phase 1: Syntax Crate**

---

## Test Strategy

- **Unit tests**: `#[cfg(test)]` modules in each source file
- **Integration tests**: `crates/covariant-syntax/tests/` for crate-level tests
- **Fixture files**: `examples/*.cov` for end-to-end parse validation
- **Framework**: Rust built-in `#[test]` + `cargo test`

Run all tests: `cargo test --workspace`

---

## Lexer Unit Tests

Location: `crates/covariant-syntax/src/lexer.rs`

| Test | Input | Expected |
|------|-------|----------|
| single_char_operators | `+ - * /` | `Plus, Minus, Star, Slash` |
| two_char_operators | `== != <= >= && \|\| \|>` | `EqEq, BangEq, LtEq, GtEq, AmpAmp, PipePipe, PipeGt` |
| arrow_and_fat_arrow | `-> =>` | `Arrow, FatArrow` |
| delimiters | `(){}[]` | `LParen, RParen, LBrace, RBrace, LBracket, RBracket` |
| integer_literal | `42` | `IntLit` |
| float_literal | `3.14` | `FloatLit` |
| length_literals | `10mm 5cm 2.5in 1m` | `LengthLit × 4` |
| angle_literals | `45deg 0.5rad` | `AngleLit × 2` |
| string_literal | `"hello"` | `StringLit` |
| string_with_escape | `"he\"llo"` | `StringLit` |
| keywords | `let data fn enum if else match with` | respective keyword tokens |
| booleans | `true false` | `True, False` |
| identifiers | `foo ISO_METRIC M3` | `Ident × 3` |
| line_comment | `42 // comment\n10` | `IntLit, Newline, IntLit` |
| block_comment | `42 /* comment */ 10` | `IntLit, IntLit` |
| nested_block_comment | `/* outer /* inner */ still */` | (no tokens) |
| let_statement | `let x = 10mm` | `Let, Ident, Eq, LengthLit` |
| function_call | `box(vec3(80mm, 50mm, 5mm))` | `Ident, LParen, Ident, LParen, LengthLit, Comma, ...` |
| pipe_chain | `x \|> f \|> g` | `Ident, PipeGt, Ident, PipeGt, Ident` |
| lambda_delimiter | `\|x\| x + 1` | `Pipe, Ident, Pipe, Ident, Plus, IntLit` |
| number_followed_by_unknown_suffix | `10min` | `IntLit, Ident` |
| semicolons | `a; b` | `Ident, Semicolon, Ident` |
| eof_token_always_present | (empty) | `Eof` only |
| **error_unterminated_string** | `"unterminated` | 1 error: `UnterminatedString` |
| **error_unexpected_char** | `@` | 1 error: `UnexpectedChar` |
| **error_unterminated_block_comment** | `/* no end` | 1 error: `UnterminatedBlockComment` |

---

## Parser Unit Tests

Location: `crates/covariant-syntax/src/parser.rs`

### Literals

| Test | Input | Expected AST |
|------|-------|-------------|
| integer_literal | `42` | `Expr::IntLit(42)` |
| float_literal | `3.14` | `Expr::FloatLit(3.14)` |
| length_literal | `10mm` | `Expr::LengthLit(10.0, Mm)` |
| angle_literal | `45deg` | `Expr::AngleLit(45.0, Deg)` |
| bool_literal | `true` / `false` | `Expr::BoolLit(true/false)` |
| string_literal | `"hello"` | `Expr::StringLit("hello")` |

### Operators and Precedence

| Test | Input | Expected AST |
|------|-------|-------------|
| binary_add | `1 + 2` | `BinOp(1, Add, 2)` |
| precedence_mul_over_add | `1 + 2 * 3` | `BinOp(1, Add, BinOp(2, Mul, 3))` |
| precedence_paren | `(1 + 2) * 3` | `BinOp(Grouped(BinOp(1, Add, 2)), Mul, 3)` |
| unary_neg | `-x` | `UnaryOp(Neg, Ident("x"))` |
| unary_not | `!flag` | `UnaryOp(Not, Ident("flag"))` |
| pipe_operator | `x \|> f \|> g` | `BinOp(BinOp(x, Pipe, f), Pipe, g)` (left-assoc) |

### Function Calls

| Test | Input | Expected AST |
|------|-------|-------------|
| function_call | `foo(1, 2)` | `FnCall { func: "foo", args: 2 }` |
| named_args | `foo(depth = 10mm, chamfer = 0.5mm)` | `FnCall { args: [named, named] }` |
| mixed_args | `f(a, b = 1)` | `FnCall { args: [positional, named] }` |
| nested_calls | `f(g(x), h(y))` | `FnCall { args: [FnCall, FnCall] }` |

### Field Access

| Test | Input | Expected AST |
|------|-------|-------------|
| field_access | `plate.width` | `FieldAccess { object: "plate", field: "width" }` |
| chained_field_and_call | `a.b(c)` | `FnCall { func: FieldAccess, args: [c] }` |

### Statements

| Test | Input | Expected AST |
|------|-------|-------------|
| let_statement | `let x = 42` | `Stmt::Let { name: "x", value: 42 }` |
| let_with_type | `let x: Int = 42` | `Stmt::Let { ty: Named("Int") }` |
| fn_def | `fn add(a: Int, b: Int) -> Int { a + b }` | `Stmt::FnDef { name: "add", params: 2 }` |
| data_def | `data Rect { w: Length, h: Length }` | `Stmt::DataDef { name: "Rect", fields: 2 }` |
| data_def_with_defaults | `data Rect { w: Length = 10mm }` | `DataDef { default: Some(...) }` |
| enum_def | `enum Color { Red, Green, Blue }` | `Stmt::EnumDef { variants: 3 }` |

### Control Flow

| Test | Input | Expected AST |
|------|-------|-------------|
| if_expression | `if true { 1 } else { 2 }` | `Expr::If { ... }` |
| if_without_else | `if flag { 1 }` | `Expr::If { else_branch: None }` |
| match_expression | `match x { 1 => "one", _ => "other" }` | `Expr::Match { arms: 2 }` |

### Lambdas and Collections

| Test | Input | Expected AST |
|------|-------|-------------|
| lambda | `\|x\| x + 1` | `Expr::Lambda { params: 1 }` |
| multi_param_lambda | `\|x, y\| x + y` | `Expr::Lambda { params: 2 }` |
| list_literal | `[1, 2, 3]` | `Expr::List(3 items)` |
| empty_list | `[]` | `Expr::List(0 items)` |

### Data Types

| Test | Input | Expected AST |
|------|-------|-------------|
| data_constructor | `Rect { w = 50mm, h = 100mm }` | `Expr::DataConstructor { fields: 2 }` |
| with_update | `r with { h = 200mm }` | `Expr::WithUpdate { ... }` |
| block_expression | `{ let x = 1; x + 2 }` | `Expr::Block { stmts: 1, tail: Some }` |

### Integration

| Test | Input | Expected |
|------|-------|----------|
| mounting_plate_example | Full SPEC.md mounting plate | 5 statements, 0 errors |

---

## Integration Tests (`.cov` fixture files)

Location: `crates/covariant-syntax/tests/parse_examples.rs`

| File | Description | Validation |
|------|-------------|------------|
| `examples/mounting_plate.cov` | Complete mounting plate from SPEC.md | 5 statements, 0 errors |
| `examples/functions.cov` | Function definitions, pipe operator, lambdas | 0 errors |
| `examples/data_types.cov` | data/enum definitions, with-update, field access | 0 errors |
| `examples/math.cov` | Numeric operations, unit literals, vector construction | 0 errors |

---

**Last Updated**: 2026-02-05
