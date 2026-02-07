use covariant_syntax::ast::{
    Arg, BinOpKind, Expr, Field, FieldInit, MatchArm, Param, Stmt,
};
use covariant_syntax::{SourceFile, Span, Spanned};

use crate::dag::Dag;
use crate::error::IrError;
use crate::node::{IrArg, IrField, IrFieldInit, IrMatchArm, IrNode, IrParam, NodeId};

/// Lower a parsed source file into an IR DAG.
pub fn lower(source: &SourceFile) -> (Dag, Vec<IrError>) {
    let mut ctx = LowerCtx::new();
    let roots: Vec<NodeId> = source
        .stmts
        .iter()
        .map(|stmt| ctx.lower_stmt(&stmt.node, stmt.span))
        .collect();
    ctx.dag.set_roots(roots);
    (ctx.dag, ctx.errors)
}

struct LowerCtx {
    dag: Dag,
    errors: Vec<IrError>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            dag: Dag::new(),
            errors: Vec::new(),
        }
    }

    fn lower_stmt(&mut self, stmt: &Stmt, span: Span) -> NodeId {
        match stmt {
            Stmt::Let(let_stmt) => {
                let value = self.lower_expr(&let_stmt.value.node, let_stmt.value.span);
                self.dag.insert(
                    IrNode::Let {
                        name: let_stmt.name.clone(),
                        ty: let_stmt.ty.clone(),
                        value,
                    },
                    span,
                )
            }
            Stmt::FnDef(fn_def) => {
                let params = self.lower_params(&fn_def.params);
                let body = self.lower_expr(&fn_def.body.node, fn_def.body.span);
                self.dag.insert(
                    IrNode::FnDef {
                        name: fn_def.name.clone(),
                        params,
                        return_ty: fn_def.return_ty.clone(),
                        body,
                    },
                    span,
                )
            }
            Stmt::DataDef(data_def) => {
                let fields = self.lower_fields(&data_def.fields);
                self.dag.insert(
                    IrNode::DataDef {
                        name: data_def.name.clone(),
                        fields,
                    },
                    span,
                )
            }
            Stmt::EnumDef(enum_def) => self.dag.insert(
                IrNode::EnumDef {
                    name: enum_def.name.clone(),
                    variants: enum_def.variants.clone(),
                },
                span,
            ),
            Stmt::Expr(expr) => self.lower_expr(&expr.node, expr.span),
        }
    }

    fn lower_expr(&mut self, expr: &Expr, span: Span) -> NodeId {
        match expr {
            // -- Literals --
            Expr::IntLit(v) => self.dag.insert(IrNode::IntLit(*v), span),
            Expr::FloatLit(v) => self.dag.insert(IrNode::FloatLit(*v), span),
            Expr::LengthLit(v, u) => self.dag.insert(IrNode::LengthLit(*v, *u), span),
            Expr::AngleLit(v, u) => self.dag.insert(IrNode::AngleLit(*v, *u), span),
            Expr::BoolLit(v) => self.dag.insert(IrNode::BoolLit(*v), span),
            Expr::StringLit(v) => self.dag.insert(IrNode::StringLit(v.clone()), span),

            // -- References --
            Expr::Ident(name) => self.dag.insert(IrNode::Ident(name.clone()), span),

            // -- Operations --
            Expr::BinOp { lhs, op, rhs } => {
                if op.node == BinOpKind::Pipe {
                    self.lower_pipe(lhs, rhs, span)
                } else {
                    let lhs_id = self.lower_expr(&lhs.node, lhs.span);
                    let rhs_id = self.lower_expr(&rhs.node, rhs.span);
                    self.dag.insert(
                        IrNode::BinOp {
                            lhs: lhs_id,
                            op: op.clone(),
                            rhs: rhs_id,
                        },
                        span,
                    )
                }
            }
            Expr::UnaryOp { op, operand } => {
                let operand_id = self.lower_expr(&operand.node, operand.span);
                self.dag.insert(
                    IrNode::UnaryOp {
                        op: op.clone(),
                        operand: operand_id,
                    },
                    span,
                )
            }

            // -- Calls / access --
            Expr::FnCall { func, args } => {
                let func_id = self.lower_expr(&func.node, func.span);
                let ir_args = self.lower_args(args);
                self.dag.insert(
                    IrNode::FnCall {
                        func: func_id,
                        args: ir_args,
                    },
                    span,
                )
            }
            Expr::FieldAccess { object, field } => {
                let object_id = self.lower_expr(&object.node, object.span);
                self.dag.insert(
                    IrNode::FieldAccess {
                        object: object_id,
                        field: field.clone(),
                    },
                    span,
                )
            }

            // -- Functions --
            Expr::Lambda { params, body } => {
                let ir_params = self.lower_params(params);
                let body_id = self.lower_expr(&body.node, body.span);
                self.dag.insert(
                    IrNode::Lambda {
                        params: ir_params,
                        body: body_id,
                    },
                    span,
                )
            }

            // -- Collections --
            Expr::List(elems) => {
                let ids: Vec<NodeId> = elems
                    .iter()
                    .map(|e| self.lower_expr(&e.node, e.span))
                    .collect();
                self.dag.insert(IrNode::List(ids), span)
            }

            // -- Data --
            Expr::DataConstructor { name, fields } => {
                let ir_fields = self.lower_field_inits(fields);
                self.dag.insert(
                    IrNode::DataConstructor {
                        name: name.clone(),
                        fields: ir_fields,
                    },
                    span,
                )
            }
            Expr::WithUpdate { base, updates } => {
                let base_id = self.lower_expr(&base.node, base.span);
                let ir_updates = self.lower_field_inits(updates);
                self.dag.insert(
                    IrNode::WithUpdate {
                        base: base_id,
                        updates: ir_updates,
                    },
                    span,
                )
            }

            // -- Control flow --
            Expr::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_id = self.lower_expr(&cond.node, cond.span);
                let then_id = self.lower_expr(&then_branch.node, then_branch.span);
                let else_id = else_branch
                    .as_ref()
                    .map(|e| self.lower_expr(&e.node, e.span));
                self.dag.insert(
                    IrNode::If {
                        cond: cond_id,
                        then_branch: then_id,
                        else_branch: else_id,
                    },
                    span,
                )
            }
            Expr::Match { subject, arms } => {
                let subject_id = self.lower_expr(&subject.node, subject.span);
                let ir_arms = self.lower_match_arms(arms);
                self.dag.insert(
                    IrNode::Match {
                        subject: subject_id,
                        arms: ir_arms,
                    },
                    span,
                )
            }

            // -- Structure --
            Expr::Block { stmts, tail } => {
                let stmt_ids: Vec<NodeId> = stmts
                    .iter()
                    .map(|s| self.lower_stmt(&s.node, s.span))
                    .collect();
                let tail_id = tail.as_ref().map(|e| self.lower_expr(&e.node, e.span));
                self.dag.insert(
                    IrNode::Block {
                        stmts: stmt_ids,
                        tail: tail_id,
                    },
                    span,
                )
            }

            // -- Grouped elimination --
            Expr::Grouped(inner) => self.lower_expr(&inner.node, inner.span),
        }
    }

    /// Desugar `lhs |> rhs` into `FnCall`.
    ///
    /// - `a |> f`       → `FnCall { func: f, args: [a] }`
    /// - `a |> f(b, c)` → `FnCall { func: f, args: [a, b, c] }` (prepend)
    fn lower_pipe(
        &mut self,
        lhs: &Spanned<Expr>,
        rhs: &Spanned<Expr>,
        span: Span,
    ) -> NodeId {
        let lhs_id = self.lower_expr(&lhs.node, lhs.span);

        match &rhs.node {
            Expr::FnCall { func, args } => {
                // `a |> f(b, c)` → prepend lhs as first arg
                let func_id = self.lower_expr(&func.node, func.span);
                let lhs_arg = IrArg {
                    name: None,
                    value: lhs_id,
                    span: lhs.span,
                };
                let mut ir_args = vec![lhs_arg];
                ir_args.extend(self.lower_args(args));
                self.dag.insert(
                    IrNode::FnCall {
                        func: func_id,
                        args: ir_args,
                    },
                    span,
                )
            }
            _ => {
                // `a |> f` → `FnCall { func: f, args: [a] }`
                let func_id = self.lower_expr(&rhs.node, rhs.span);
                let lhs_arg = IrArg {
                    name: None,
                    value: lhs_id,
                    span: lhs.span,
                };
                self.dag.insert(
                    IrNode::FnCall {
                        func: func_id,
                        args: vec![lhs_arg],
                    },
                    span,
                )
            }
        }
    }

    fn lower_params(&mut self, params: &[Param]) -> Vec<IrParam> {
        params
            .iter()
            .map(|p| {
                let default = p
                    .default
                    .as_ref()
                    .map(|d| self.lower_expr(&d.node, d.span));
                IrParam {
                    name: p.name.clone(),
                    ty: p.ty.clone(),
                    default,
                    span: p.span,
                }
            })
            .collect()
    }

    fn lower_args(&mut self, args: &[Arg]) -> Vec<IrArg> {
        args.iter()
            .map(|a| {
                let value = self.lower_expr(&a.value.node, a.value.span);
                IrArg {
                    name: a.name.clone(),
                    value,
                    span: a.span,
                }
            })
            .collect()
    }

    fn lower_field_inits(&mut self, fields: &[FieldInit]) -> Vec<IrFieldInit> {
        fields
            .iter()
            .map(|f| {
                let value = self.lower_expr(&f.value.node, f.value.span);
                IrFieldInit {
                    name: f.name.clone(),
                    value,
                    span: f.span,
                }
            })
            .collect()
    }

    fn lower_match_arms(&mut self, arms: &[MatchArm]) -> Vec<IrMatchArm> {
        arms.iter()
            .map(|a| {
                let body = self.lower_expr(&a.body.node, a.body.span);
                IrMatchArm {
                    pattern: a.pattern.clone(),
                    body,
                    span: a.span,
                }
            })
            .collect()
    }

    fn lower_fields(&mut self, fields: &[Field]) -> Vec<IrField> {
        fields
            .iter()
            .map(|f| {
                let default = f
                    .default
                    .as_ref()
                    .map(|d| self.lower_expr(&d.node, d.span));
                IrField {
                    name: f.name.clone(),
                    ty: f.ty.clone(),
                    default,
                    span: f.span,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use covariant_syntax::ast::*;

    use super::*;
    use crate::node::IrNode;

    /// Helper: parse a source string and lower it.
    fn parse_and_lower(source: &str) -> (Dag, Vec<IrError>) {
        let (ast, parse_errors) = covariant_syntax::parse(source);
        assert!(
            parse_errors.is_empty(),
            "parse errors: {parse_errors:?}"
        );
        lower(&ast)
    }

    // ======== Literal tests ========

    #[test]
    fn lower_int_lit() {
        let (dag, errors) = parse_and_lower("let x = 42");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                assert_eq!(*dag.node(*value), IrNode::IntLit(42));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_float_lit() {
        let (dag, errors) = parse_and_lower("let x = 3.14");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                assert_eq!(*dag.node(*value), IrNode::FloatLit(3.14));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_length_lit() {
        let (dag, errors) = parse_and_lower("let x = 10mm");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                assert_eq!(*dag.node(*value), IrNode::LengthLit(10.0, LengthUnit::Mm));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_angle_lit() {
        let (dag, errors) = parse_and_lower("let x = 45deg");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                assert_eq!(*dag.node(*value), IrNode::AngleLit(45.0, AngleUnit::Deg));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_bool_lit() {
        let (dag, errors) = parse_and_lower("let x = true");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                assert_eq!(*dag.node(*value), IrNode::BoolLit(true));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_string_lit() {
        let (dag, errors) = parse_and_lower("let x = \"hello\"");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                assert_eq!(
                    *dag.node(*value),
                    IrNode::StringLit("hello".to_string())
                );
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Reference tests ========

    #[test]
    fn lower_ident() {
        let (dag, errors) = parse_and_lower("let x = y");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                assert_eq!(*dag.node(*value), IrNode::Ident("y".to_string()));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Operation tests ========

    #[test]
    fn lower_binop() {
        let (dag, errors) = parse_and_lower("let x = 1 + 2");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::BinOp { lhs, op, rhs } => {
                    assert_eq!(*dag.node(*lhs), IrNode::IntLit(1));
                    assert_eq!(op.node, BinOpKind::Add);
                    assert_eq!(*dag.node(*rhs), IrNode::IntLit(2));
                }
                other => panic!("expected BinOp, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_unaryop() {
        let (dag, errors) = parse_and_lower("let x = -5");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::UnaryOp { op, operand } => {
                    assert_eq!(op.node, UnaryOpKind::Neg);
                    assert_eq!(*dag.node(*operand), IrNode::IntLit(5));
                }
                other => panic!("expected UnaryOp, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Grouped elimination ========

    #[test]
    fn lower_grouped_elimination() {
        let (dag, errors) = parse_and_lower("let x = (42)");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => {
                // Grouped should be eliminated — directly an IntLit
                assert_eq!(*dag.node(*value), IrNode::IntLit(42));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Pipe desugaring ========

    #[test]
    fn lower_pipe_simple() {
        // `a |> f` → `FnCall { func: f, args: [a] }`
        let (dag, errors) = parse_and_lower("let x = a |> f");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::FnCall { func, args } => {
                    assert_eq!(*dag.node(*func), IrNode::Ident("f".to_string()));
                    assert_eq!(args.len(), 1);
                    assert_eq!(*dag.node(args[0].value), IrNode::Ident("a".to_string()));
                }
                other => panic!("expected FnCall, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_pipe_with_args() {
        // `a |> f(b)` → `FnCall { func: f, args: [a, b] }`
        let (dag, errors) = parse_and_lower("let x = a |> f(b)");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::FnCall { func, args } => {
                    assert_eq!(*dag.node(*func), IrNode::Ident("f".to_string()));
                    assert_eq!(args.len(), 2);
                    assert_eq!(*dag.node(args[0].value), IrNode::Ident("a".to_string()));
                    assert_eq!(*dag.node(args[1].value), IrNode::Ident("b".to_string()));
                }
                other => panic!("expected FnCall, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_pipe_chained() {
        // `a |> f |> g` → `g(f(a))`
        let (dag, errors) = parse_and_lower("let x = a |> f |> g");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::FnCall { func, args } => {
                    // Outer call is g(...)
                    assert_eq!(*dag.node(*func), IrNode::Ident("g".to_string()));
                    assert_eq!(args.len(), 1);
                    // Inner arg is f(a)
                    match dag.node(args[0].value) {
                        IrNode::FnCall {
                            func: inner_func,
                            args: inner_args,
                        } => {
                            assert_eq!(
                                *dag.node(*inner_func),
                                IrNode::Ident("f".to_string())
                            );
                            assert_eq!(inner_args.len(), 1);
                            assert_eq!(
                                *dag.node(inner_args[0].value),
                                IrNode::Ident("a".to_string())
                            );
                        }
                        other => panic!("expected inner FnCall, got {other:?}"),
                    }
                }
                other => panic!("expected FnCall, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== FnCall / FieldAccess ========

    #[test]
    fn lower_fncall() {
        let (dag, errors) = parse_and_lower("let x = f(1, 2)");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::FnCall { func, args } => {
                    assert_eq!(*dag.node(*func), IrNode::Ident("f".to_string()));
                    assert_eq!(args.len(), 2);
                }
                other => panic!("expected FnCall, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    #[test]
    fn lower_field_access() {
        let (dag, errors) = parse_and_lower("let x = obj.field");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::FieldAccess { object, field } => {
                    assert_eq!(*dag.node(*object), IrNode::Ident("obj".to_string()));
                    assert_eq!(field.node, "field");
                }
                other => panic!("expected FieldAccess, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== List ========

    #[test]
    fn lower_list() {
        let (dag, errors) = parse_and_lower("let x = [1, 2, 3]");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::List(elems) => {
                    assert_eq!(elems.len(), 3);
                    assert_eq!(*dag.node(elems[0]), IrNode::IntLit(1));
                    assert_eq!(*dag.node(elems[1]), IrNode::IntLit(2));
                    assert_eq!(*dag.node(elems[2]), IrNode::IntLit(3));
                }
                other => panic!("expected List, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Lambda ========

    #[test]
    fn lower_lambda() {
        let (dag, errors) = parse_and_lower("let f = |x| x");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::Lambda { params, body } => {
                    assert_eq!(params.len(), 1);
                    assert_eq!(params[0].name.node, "x");
                    assert_eq!(*dag.node(*body), IrNode::Ident("x".to_string()));
                }
                other => panic!("expected Lambda, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== If ========

    #[test]
    fn lower_if_else() {
        let (dag, errors) = parse_and_lower("let x = if true { 1 } else { 2 }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::If {
                    cond,
                    then_branch,
                    else_branch,
                } => {
                    assert_eq!(*dag.node(*cond), IrNode::BoolLit(true));
                    assert!(matches!(dag.node(*then_branch), IrNode::Block { .. }));
                    assert!(else_branch.is_some());
                }
                other => panic!("expected If, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Match ========

    #[test]
    fn lower_match() {
        let (dag, errors) = parse_and_lower("let x = match y { 1 => 10, _ => 0 }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::Match { subject, arms } => {
                    assert_eq!(*dag.node(*subject), IrNode::Ident("y".to_string()));
                    assert_eq!(arms.len(), 2);
                }
                other => panic!("expected Match, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Block ========

    #[test]
    fn lower_block() {
        let (dag, errors) = parse_and_lower("let x = { let y = 1\n y }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::Block { stmts, tail } => {
                    assert_eq!(stmts.len(), 1);
                    assert!(tail.is_some());
                }
                other => panic!("expected Block, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Data constructor ========

    #[test]
    fn lower_data_constructor() {
        let (dag, errors) =
            parse_and_lower("let r = Rectangle { width = 10mm, height = 20mm }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::DataConstructor { name, fields } => {
                    assert_eq!(name.node, "Rectangle");
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].name.node, "width");
                    assert_eq!(fields[1].name.node, "height");
                }
                other => panic!("expected DataConstructor, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== With-update ========

    #[test]
    fn lower_with_update() {
        let (dag, errors) = parse_and_lower("let r2 = r with { height = 200mm }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::Let { value, .. } => match dag.node(*value) {
                IrNode::WithUpdate { base, updates } => {
                    assert_eq!(*dag.node(*base), IrNode::Ident("r".to_string()));
                    assert_eq!(updates.len(), 1);
                    assert_eq!(updates[0].name.node, "height");
                }
                other => panic!("expected WithUpdate, got {other:?}"),
            },
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== Let ========

    #[test]
    fn lower_let() {
        let (dag, errors) = parse_and_lower("let x = 42");
        assert!(errors.is_empty());
        assert_eq!(dag.roots().len(), 1);
        match dag.node(dag.roots()[0]) {
            IrNode::Let { name, ty, value } => {
                assert_eq!(name.node, "x");
                assert!(ty.is_none());
                assert_eq!(*dag.node(*value), IrNode::IntLit(42));
            }
            other => panic!("expected Let, got {other:?}"),
        }
    }

    // ======== FnDef ========

    #[test]
    fn lower_fn_def() {
        let (dag, errors) =
            parse_and_lower("fn double(x: Int) -> Int { x * 2 }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::FnDef {
                name,
                params,
                return_ty,
                body,
            } => {
                assert_eq!(name.node, "double");
                assert_eq!(params.len(), 1);
                assert_eq!(params[0].name.node, "x");
                assert!(return_ty.is_some());
                assert!(matches!(dag.node(*body), IrNode::Block { .. }));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    // ======== DataDef ========

    #[test]
    fn lower_data_def() {
        let (dag, errors) =
            parse_and_lower("data Rectangle { width: Length, height: Length }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::DataDef { name, fields } => {
                assert_eq!(name.node, "Rectangle");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].name.node, "width");
                assert_eq!(fields[1].name.node, "height");
            }
            other => panic!("expected DataDef, got {other:?}"),
        }
    }

    // ======== EnumDef ========

    #[test]
    fn lower_enum_def() {
        let (dag, errors) = parse_and_lower("enum Color { Red, Green, Blue }");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        match dag.node(root) {
            IrNode::EnumDef { name, variants } => {
                assert_eq!(name.node, "Color");
                assert_eq!(variants.len(), 3);
                assert_eq!(variants[0].node, "Red");
                assert_eq!(variants[1].node, "Green");
                assert_eq!(variants[2].node, "Blue");
            }
            other => panic!("expected EnumDef, got {other:?}"),
        }
    }

    // ======== Span preservation ========

    #[test]
    fn span_preservation() {
        let (dag, errors) = parse_and_lower("let x = 42");
        assert!(errors.is_empty());
        let root = dag.roots()[0];
        let root_span = dag.span(root);
        // The Let statement should span at least from "let" to "42"
        assert!(root_span.end > root_span.start);
    }

    // ======== No pipe in IR ========

    #[test]
    fn no_pipe_in_ir() {
        let (dag, errors) = parse_and_lower("let x = a |> f |> g(b)");
        assert!(errors.is_empty());
        for (_id, data) in dag.iter() {
            if let IrNode::BinOp { op, .. } = &data.node {
                assert_ne!(
                    op.node,
                    BinOpKind::Pipe,
                    "Pipe should be desugared, not present in IR"
                );
            }
        }
    }
}
