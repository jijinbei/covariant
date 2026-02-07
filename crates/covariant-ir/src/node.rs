use covariant_syntax::ast::{AngleUnit, BinOpKind, LengthUnit, Pattern, Type, UnaryOpKind};
use covariant_syntax::{Span, Spanned};

/// Index into the DAG arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub(crate) u32);

impl NodeId {
    /// Create a `NodeId` from a raw index (for test/debug use in other crates).
    pub fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "n{}", self.0)
    }
}

/// An IR node with its source span.
#[derive(Debug, Clone, PartialEq)]
pub struct IrNodeData {
    pub node: IrNode,
    pub span: Span,
}

// ======== Supporting types ========

/// A function/lambda parameter in IR form.
#[derive(Debug, Clone, PartialEq)]
pub struct IrParam {
    pub name: Spanned<String>,
    pub ty: Option<Spanned<Type>>,
    pub default: Option<NodeId>,
    pub span: Span,
}

/// A function call argument in IR form.
#[derive(Debug, Clone, PartialEq)]
pub struct IrArg {
    pub name: Option<Spanned<String>>,
    pub value: NodeId,
    pub span: Span,
}

/// A field initializer in data constructors and with-updates.
#[derive(Debug, Clone, PartialEq)]
pub struct IrFieldInit {
    pub name: Spanned<String>,
    pub value: NodeId,
    pub span: Span,
}

/// A match arm in IR form.
#[derive(Debug, Clone, PartialEq)]
pub struct IrMatchArm {
    pub pattern: Spanned<Pattern>,
    pub body: NodeId,
    pub span: Span,
}

/// A field in a data definition.
#[derive(Debug, Clone, PartialEq)]
pub struct IrField {
    pub name: Spanned<String>,
    pub ty: Spanned<Type>,
    pub default: Option<NodeId>,
    pub span: Span,
}

// ======== IR node enum ========

/// The IR node variants, mirroring the AST but arena-allocated.
///
/// `Grouped` is eliminated during lowering (unwrapped to its inner node).
/// `Pipe` (`|>`) is desugared into `FnCall`.
#[derive(Debug, Clone, PartialEq)]
pub enum IrNode {
    // -- Literals --
    IntLit(i64),
    FloatLit(f64),
    LengthLit(f64, LengthUnit),
    AngleLit(f64, AngleUnit),
    BoolLit(bool),
    StringLit(String),

    // -- References --
    Ident(String),

    // -- Operations --
    BinOp {
        lhs: NodeId,
        op: Spanned<BinOpKind>,
        rhs: NodeId,
    },
    UnaryOp {
        op: Spanned<UnaryOpKind>,
        operand: NodeId,
    },

    // -- Calls / access --
    FnCall {
        func: NodeId,
        args: Vec<IrArg>,
    },
    FieldAccess {
        object: NodeId,
        field: Spanned<String>,
    },

    // -- Functions --
    Lambda {
        params: Vec<IrParam>,
        body: NodeId,
    },

    // -- Collections --
    List(Vec<NodeId>),

    // -- Data --
    DataConstructor {
        name: Spanned<String>,
        fields: Vec<IrFieldInit>,
    },
    WithUpdate {
        base: NodeId,
        updates: Vec<IrFieldInit>,
    },

    // -- Control flow --
    If {
        cond: NodeId,
        then_branch: NodeId,
        else_branch: Option<NodeId>,
    },
    Match {
        subject: NodeId,
        arms: Vec<IrMatchArm>,
    },

    // -- Structure --
    Block {
        stmts: Vec<NodeId>,
        tail: Option<NodeId>,
    },

    // -- Statements --
    Let {
        name: Spanned<String>,
        ty: Option<Spanned<Type>>,
        value: NodeId,
    },
    FnDef {
        name: Spanned<String>,
        params: Vec<IrParam>,
        return_ty: Option<Spanned<Type>>,
        body: NodeId,
    },
    DataDef {
        name: Spanned<String>,
        fields: Vec<IrField>,
    },
    EnumDef {
        name: Spanned<String>,
        variants: Vec<Spanned<String>>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_display() {
        assert_eq!(format!("{}", NodeId(0)), "n0");
        assert_eq!(format!("{}", NodeId(42)), "n42");
    }

    #[test]
    fn node_id_ordering() {
        let a = NodeId(0);
        let b = NodeId(1);
        let c = NodeId(2);
        assert!(a < b);
        assert!(b < c);
        assert_eq!(a, NodeId(0));
    }

    #[test]
    fn node_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NodeId(0));
        set.insert(NodeId(1));
        set.insert(NodeId(0));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn ir_node_data_construction() {
        let data = IrNodeData {
            node: IrNode::IntLit(42),
            span: Span::new(0, 2),
        };
        assert_eq!(data.node, IrNode::IntLit(42));
        assert_eq!(data.span, Span::new(0, 2));
    }

    #[test]
    fn ir_param_construction() {
        let param = IrParam {
            name: Spanned::new("x".to_string(), Span::new(0, 1)),
            ty: None,
            default: None,
            span: Span::new(0, 1),
        };
        assert_eq!(param.name.node, "x");
    }

    #[test]
    fn ir_arg_construction() {
        let arg = IrArg {
            name: Some(Spanned::new("depth".to_string(), Span::new(0, 5))),
            value: NodeId(0),
            span: Span::new(0, 10),
        };
        assert!(arg.name.is_some());
    }

    #[test]
    fn ir_field_init_construction() {
        let fi = IrFieldInit {
            name: Spanned::new("width".to_string(), Span::new(0, 5)),
            value: NodeId(0),
            span: Span::new(0, 12),
        };
        assert_eq!(fi.name.node, "width");
    }

    #[test]
    fn ir_node_variants() {
        // Verify we can construct each variant category
        let _lit = IrNode::IntLit(1);
        let _flt = IrNode::FloatLit(1.0);
        let _len = IrNode::LengthLit(10.0, LengthUnit::Mm);
        let _ang = IrNode::AngleLit(45.0, AngleUnit::Deg);
        let _boo = IrNode::BoolLit(true);
        let _str = IrNode::StringLit("hi".to_string());
        let _id = IrNode::Ident("x".to_string());
        let _list = IrNode::List(vec![]);
        let _enum = IrNode::EnumDef {
            name: Spanned::new("Color".to_string(), Span::new(0, 5)),
            variants: vec![],
        };
    }
}
