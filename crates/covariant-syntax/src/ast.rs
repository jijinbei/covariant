use crate::span::{Span, Spanned};

/// A complete source file (compilation unit).
#[derive(Debug, Clone, PartialEq)]
pub struct SourceFile {
    pub stmts: Vec<Spanned<Stmt>>,
    pub span: Span,
}

// ======== Statements ========

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `let x = expr` or `let x: Type = expr`
    Let(LetStmt),
    /// `fn name(params) -> ReturnType { body }`
    FnDef(FnDef),
    /// `data Name { field: Type, ... }`
    DataDef(DataDef),
    /// `enum Name { Variant1, Variant2, ... }`
    EnumDef(EnumDef),
    /// An expression used as a statement.
    Expr(Spanned<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: Spanned<String>,
    pub ty: Option<Spanned<Type>>,
    pub value: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDef {
    pub name: Spanned<String>,
    pub params: Vec<Param>,
    pub return_ty: Option<Spanned<Type>>,
    pub body: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Spanned<String>,
    pub ty: Option<Spanned<Type>>,
    pub default: Option<Spanned<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataDef {
    pub name: Spanned<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
    pub default: Option<Spanned<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: Spanned<String>,
    pub variants: Vec<Spanned<String>>,
}

// ======== Expressions ========

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal: `42`
    IntLit(i64),
    /// Float literal: `3.14`
    FloatLit(f64),
    /// Length literal with unit: `10mm`
    LengthLit(f64, LengthUnit),
    /// Angle literal with unit: `45deg`
    AngleLit(f64, AngleUnit),
    /// Boolean: `true` or `false`
    BoolLit(bool),
    /// String: `"hello"`
    StringLit(String),
    /// Identifier: `x`, `plate`, `ISO_METRIC`
    Ident(String),

    /// Binary operation: `a + b`, `a |> b`
    BinOp {
        lhs: Box<Spanned<Expr>>,
        op: Spanned<BinOpKind>,
        rhs: Box<Spanned<Expr>>,
    },
    /// Unary operation: `!x`, `-x`
    UnaryOp {
        op: Spanned<UnaryOpKind>,
        operand: Box<Spanned<Expr>>,
    },
    /// Function call: `foo(a, b)` or `foo(depth = 10mm)`
    FnCall {
        func: Box<Spanned<Expr>>,
        args: Vec<Arg>,
    },
    /// Field access: `plate.width`
    FieldAccess {
        object: Box<Spanned<Expr>>,
        field: Spanned<String>,
    },
    /// Lambda: `|x| x + 1` or `|x, y| x + y`
    Lambda {
        params: Vec<Param>,
        body: Box<Spanned<Expr>>,
    },
    /// List literal: `[1, 2, 3]`
    List(Vec<Spanned<Expr>>),
    /// If expression: `if cond { then } else { otherwise }`
    If {
        cond: Box<Spanned<Expr>>,
        then_branch: Box<Spanned<Expr>>,
        else_branch: Option<Box<Spanned<Expr>>>,
    },
    /// Match expression: `match expr { pattern => body, ... }`
    Match {
        subject: Box<Spanned<Expr>>,
        arms: Vec<MatchArm>,
    },
    /// Data constructor: `Rectangle { width = 50mm, height = 100mm }`
    DataConstructor {
        name: Spanned<String>,
        fields: Vec<FieldInit>,
    },
    /// With-update: `r1 with { height = 200mm }`
    WithUpdate {
        base: Box<Spanned<Expr>>,
        updates: Vec<FieldInit>,
    },
    /// Block expression: `{ stmt; stmt; expr }`
    Block {
        stmts: Vec<Spanned<Stmt>>,
        tail: Option<Box<Spanned<Expr>>>,
    },
    /// Grouped expression: `(expr)` — used only during parsing,
    /// the parentheses are stripped and the inner expression is used.
    Grouped(Box<Spanned<Expr>>),
}

// ======== Supporting types ========

#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    pub name: Option<Spanned<String>>,
    pub value: Spanned<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldInit {
    pub name: Spanned<String>,
    pub value: Spanned<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Spanned<Pattern>,
    pub body: Spanned<Expr>,
    pub span: Span,
}

/// Simplified patterns for v0.1.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Identifier or variant name.
    Ident(String),
    /// Wildcard: `_`
    Wildcard,
    /// Literal pattern.
    Literal(Box<Spanned<Expr>>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOpKind {
    Add,  // +
    Sub,  // -
    Mul,  // *
    Div,  // /
    Eq,   // ==
    Neq,  // !=
    Lt,   // <
    Leq,  // <=
    Gt,   // >
    Geq,  // >=
    And,  // &&
    Or,   // ||
    Pipe, // |>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOpKind {
    Neg, // -
    Not, // !
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnit {
    Mm,
    Cm,
    M,
    In,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AngleUnit {
    Deg,
    Rad,
}

// ======== Type annotations ========

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Named type: `Length`, `Vec3`, `Solid`, user-defined, etc.
    Named(String),
    /// List type: `List[T]`
    List(Box<Spanned<Type>>),
    /// Function type: `Fn(A, B) -> C`
    Fn {
        params: Vec<Spanned<Type>>,
        ret: Box<Spanned<Type>>,
    },
}
