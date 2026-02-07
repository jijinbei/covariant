//! The COVARIANT evaluator — walks the IR DAG and produces runtime values.

use covariant_geom::GeomKernel;
use covariant_ir::node::{IrArg, IrNode};
use covariant_ir::{Dag, NodeId};
use covariant_syntax::ast::{BinOpKind, Pattern, UnaryOpKind};
use covariant_syntax::Span;

use crate::builtins::register_builtins;
use crate::env::Env;
use crate::error::{EvalError, EvalErrorKind, EvalResult};
use crate::units::{angle_to_rad, length_to_mm};
use crate::value::{FnParam, Value};

/// Evaluation context holding references to the DAG, environment, and geometry kernel.
pub struct EvalCtx<'a> {
    pub dag: &'a Dag,
    pub env: Env,
    pub kernel: &'a dyn GeomKernel,
    /// Registered data type definitions: type_name → field names.
    data_types: std::collections::HashMap<String, Vec<String>>,
}

/// Evaluate an IR DAG, returning the value of the last root node.
pub fn eval(dag: &Dag, kernel: &dyn GeomKernel) -> EvalResult<Value> {
    let mut env = Env::new();
    register_builtins(&mut env);

    let mut ctx = EvalCtx::new(dag, env, kernel);

    let roots = dag.roots();
    let mut last = Value::Unit;
    for &root in roots {
        last = ctx.eval_node(root)?;
    }
    Ok(last)
}

impl<'a> EvalCtx<'a> {
    /// Create a new evaluation context (for testing and builtins).
    pub fn new(dag: &'a Dag, env: Env, kernel: &'a dyn GeomKernel) -> Self {
        Self {
            dag,
            env,
            kernel,
            data_types: std::collections::HashMap::new(),
        }
    }

    /// Evaluate a single IR node by its ID.
    pub fn eval_node(&mut self, id: NodeId) -> EvalResult<Value> {
        let node = self.dag.node(id).clone();
        let span = self.dag.span(id);

        match node {
            // ── Literals ─────────────────────────────────────────────
            IrNode::IntLit(n) => Ok(Value::Int(n)),
            IrNode::FloatLit(f) => Ok(Value::Float(f)),
            IrNode::LengthLit(val, unit) => Ok(Value::Length(length_to_mm(val, unit))),
            IrNode::AngleLit(val, unit) => Ok(Value::Angle(angle_to_rad(val, unit))),
            IrNode::BoolLit(b) => Ok(Value::Bool(b)),
            IrNode::StringLit(s) => Ok(Value::String(s)),

            // ── References ───────────────────────────────────────────
            IrNode::Ident(name) => self
                .env
                .lookup(&name)
                .cloned()
                .ok_or_else(|| {
                    EvalError::new(
                        EvalErrorKind::UndefinedName,
                        format!("undefined name '{name}'"),
                        Some(span),
                    )
                }),

            // ── Binary operations ────────────────────────────────────
            IrNode::BinOp { lhs, op, rhs } => {
                let l = self.eval_node(lhs)?;
                let r = self.eval_node(rhs)?;
                self.eval_binop(l, op.node, r, span)
            }

            // ── Unary operations ─────────────────────────────────────
            IrNode::UnaryOp { op, operand } => {
                let val = self.eval_node(operand)?;
                self.eval_unaryop(op.node, val, span)
            }

            // ── Function call ────────────────────────────────────────
            IrNode::FnCall { func, args } => {
                let func_val = self.eval_node(func)?;
                self.eval_call(func_val, &args, span)
            }

            // ── Field access ─────────────────────────────────────────
            IrNode::FieldAccess { object, field } => {
                let obj = self.eval_node(object)?;
                match obj {
                    Value::Data { fields, .. } => fields
                        .iter()
                        .find(|(n, _)| *n == field.node)
                        .map(|(_, v)| v.clone())
                        .ok_or_else(|| {
                            EvalError::new(
                                EvalErrorKind::FieldNotFound,
                                format!("field '{}' not found", field.node),
                                Some(field.span),
                            )
                        }),
                    _ => Err(EvalError::new(
                        EvalErrorKind::TypeError,
                        format!(
                            "field access on non-data type {}",
                            obj.type_name()
                        ),
                        Some(span),
                    )),
                }
            }

            // ── Lambda ───────────────────────────────────────────────
            IrNode::Lambda { params, body } => {
                let fn_params = params
                    .iter()
                    .map(|p| FnParam {
                        name: p.name.node.clone(),
                        default: p.default,
                    })
                    .collect();
                Ok(Value::Function {
                    params: fn_params,
                    body,
                    closure_env: self.env.clone(),
                })
            }

            // ── List ─────────────────────────────────────────────────
            IrNode::List(elements) => {
                let items = elements
                    .iter()
                    .map(|&e| self.eval_node(e))
                    .collect::<EvalResult<Vec<_>>>()?;
                Ok(Value::List(items))
            }

            // ── Data constructor ─────────────────────────────────────
            IrNode::DataConstructor { name, fields } => {
                let field_values = fields
                    .iter()
                    .map(|f| {
                        let val = self.eval_node(f.value)?;
                        Ok((f.name.node.clone(), val))
                    })
                    .collect::<EvalResult<Vec<_>>>()?;
                Ok(Value::Data {
                    type_name: name.node,
                    fields: field_values,
                })
            }

            // ── With-update ──────────────────────────────────────────
            IrNode::WithUpdate { base, updates } => {
                let base_val = self.eval_node(base)?;
                match base_val {
                    Value::Data {
                        type_name,
                        mut fields,
                    } => {
                        for upd in &updates {
                            let val = self.eval_node(upd.value)?;
                            if let Some(entry) = fields.iter_mut().find(|(n, _)| *n == upd.name.node) {
                                entry.1 = val;
                            } else {
                                return Err(EvalError::new(
                                    EvalErrorKind::FieldNotFound,
                                    format!(
                                        "field '{}' not found in {}",
                                        upd.name.node, type_name
                                    ),
                                    Some(upd.span),
                                ));
                            }
                        }
                        Ok(Value::Data { type_name, fields })
                    }
                    _ => Err(EvalError::new(
                        EvalErrorKind::TypeError,
                        format!(
                            "with-update on non-data type {}",
                            base_val.type_name()
                        ),
                        Some(span),
                    )),
                }
            }

            // ── If expression ────────────────────────────────────────
            IrNode::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_node(cond)?;
                match cond_val {
                    Value::Bool(true) => self.eval_node(then_branch),
                    Value::Bool(false) => match else_branch {
                        Some(eb) => self.eval_node(eb),
                        None => Ok(Value::Unit),
                    },
                    _ => Err(EvalError::new(
                        EvalErrorKind::TypeError,
                        format!(
                            "if condition must be Bool, got {}",
                            cond_val.type_name()
                        ),
                        Some(span),
                    )),
                }
            }

            // ── Match expression ─────────────────────────────────────
            IrNode::Match { subject, arms } => {
                let subj = self.eval_node(subject)?;
                for arm in &arms {
                    if let Some(bindings) = self.match_pattern(&arm.pattern.node, &subj) {
                        self.env.push_scope();
                        for (name, val) in bindings {
                            self.env.define(name, val);
                        }
                        let result = self.eval_node(arm.body);
                        self.env.pop_scope();
                        return result;
                    }
                }
                Err(EvalError::new(
                    EvalErrorKind::PatternMismatch,
                    "no pattern matched",
                    Some(span),
                ))
            }

            // ── Block ────────────────────────────────────────────────
            IrNode::Block { stmts, tail } => {
                self.env.push_scope();
                for &stmt in &stmts {
                    self.eval_node(stmt)?;
                }
                let result = match tail {
                    Some(t) => self.eval_node(t),
                    None => Ok(Value::Unit),
                };
                self.env.pop_scope();
                result
            }

            // ── Let binding ──────────────────────────────────────────
            IrNode::Let { name, value, .. } => {
                let val = self.eval_node(value)?;
                self.env.define(name.node, val);
                Ok(Value::Unit)
            }

            // ── Function definition ──────────────────────────────────
            IrNode::FnDef {
                name, params, body, ..
            } => {
                let fn_params = params
                    .iter()
                    .map(|p| FnParam {
                        name: p.name.node.clone(),
                        default: p.default,
                    })
                    .collect();
                let func = Value::Function {
                    params: fn_params,
                    body,
                    closure_env: self.env.clone(),
                };
                self.env.define(name.node, func);
                Ok(Value::Unit)
            }

            // ── Data definition ──────────────────────────────────────
            IrNode::DataDef { name, fields } => {
                let field_names: Vec<String> =
                    fields.iter().map(|f| f.name.node.clone()).collect();
                self.data_types.insert(name.node, field_names);
                Ok(Value::Unit)
            }

            // ── Enum definition ──────────────────────────────────────
            IrNode::EnumDef { name, variants } => {
                for v in &variants {
                    self.env.define(
                        &v.node,
                        Value::EnumVariant {
                            type_name: name.node.clone(),
                            variant: v.node.clone(),
                        },
                    );
                }
                Ok(Value::Unit)
            }
        }
    }

    /// Evaluate a binary operation.
    fn eval_binop(
        &self,
        lhs: Value,
        op: BinOpKind,
        rhs: Value,
        span: Span,
    ) -> EvalResult<Value> {
        match op {
            // ── Arithmetic ───────────────────────────────────────
            BinOpKind::Add => match (&lhs, &rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                (Value::Length(a), Value::Length(b)) => Ok(Value::Length(a + b)),
                (Value::Angle(a), Value::Angle(b)) => Ok(Value::Angle(a + b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + *b as f64)),
                (Value::String(a), Value::String(b)) => {
                    Ok(Value::String(format!("{a}{b}")))
                }
                _ => Err(self.type_error_binop("+", &lhs, &rhs, span)),
            },
            BinOpKind::Sub => match (&lhs, &rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                (Value::Length(a), Value::Length(b)) => Ok(Value::Length(a - b)),
                (Value::Angle(a), Value::Angle(b)) => Ok(Value::Angle(a - b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
                _ => Err(self.type_error_binop("-", &lhs, &rhs, span)),
            },
            BinOpKind::Mul => match (&lhs, &rhs) {
                (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                (Value::Length(a), Value::Float(b)) => Ok(Value::Length(a * b)),
                (Value::Float(a), Value::Length(b)) => Ok(Value::Length(a * b)),
                (Value::Length(a), Value::Int(b)) => Ok(Value::Length(a * *b as f64)),
                (Value::Int(a), Value::Length(b)) => Ok(Value::Length(*a as f64 * b)),
                (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * *b as f64)),
                _ => Err(self.type_error_binop("*", &lhs, &rhs, span)),
            },
            BinOpKind::Div => {
                // Check division by zero for all numeric types.
                match (&lhs, &rhs) {
                    (Value::Int(_), Value::Int(0)) | (Value::Float(_), Value::Int(0)) => {
                        return Err(EvalError::new(
                            EvalErrorKind::DivisionByZero,
                            "division by zero",
                            Some(span),
                        ));
                    }
                    (_, Value::Float(b)) if *b == 0.0 => {
                        return Err(EvalError::new(
                            EvalErrorKind::DivisionByZero,
                            "division by zero",
                            Some(span),
                        ));
                    }
                    (_, Value::Length(b)) if *b == 0.0 => {
                        return Err(EvalError::new(
                            EvalErrorKind::DivisionByZero,
                            "division by zero",
                            Some(span),
                        ));
                    }
                    _ => {}
                }
                match (&lhs, &rhs) {
                    (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                    (Value::Length(a), Value::Float(b)) => Ok(Value::Length(a / b)),
                    (Value::Length(a), Value::Int(b)) => Ok(Value::Length(a / *b as f64)),
                    (Value::Length(a), Value::Length(b)) => Ok(Value::Float(a / b)),
                    (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                    (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / *b as f64)),
                    _ => Err(self.type_error_binop("/", &lhs, &rhs, span)),
                }
            }

            // ── Comparison ───────────────────────────────────────
            BinOpKind::Eq => Ok(Value::Bool(self.values_equal(&lhs, &rhs))),
            BinOpKind::Neq => Ok(Value::Bool(!self.values_equal(&lhs, &rhs))),
            BinOpKind::Lt => self.compare_values(&lhs, &rhs, span).map(|c| Value::Bool(c < 0)),
            BinOpKind::Leq => self.compare_values(&lhs, &rhs, span).map(|c| Value::Bool(c <= 0)),
            BinOpKind::Gt => self.compare_values(&lhs, &rhs, span).map(|c| Value::Bool(c > 0)),
            BinOpKind::Geq => self.compare_values(&lhs, &rhs, span).map(|c| Value::Bool(c >= 0)),

            // ── Logical ──────────────────────────────────────────
            BinOpKind::And => match (&lhs, &rhs) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                _ => Err(self.type_error_binop("&&", &lhs, &rhs, span)),
            },
            BinOpKind::Or => match (&lhs, &rhs) {
                (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                _ => Err(self.type_error_binop("||", &lhs, &rhs, span)),
            },

            // Pipe is desugared by the IR lowering — should not appear.
            BinOpKind::Pipe => Err(EvalError::new(
                EvalErrorKind::Custom,
                "pipe operator should have been desugared",
                Some(span),
            )),
        }
    }

    /// Evaluate a unary operation.
    fn eval_unaryop(
        &self,
        op: UnaryOpKind,
        val: Value,
        span: Span,
    ) -> EvalResult<Value> {
        match op {
            UnaryOpKind::Neg => match val {
                Value::Int(n) => Ok(Value::Int(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                Value::Length(l) => Ok(Value::Length(-l)),
                Value::Angle(a) => Ok(Value::Angle(-a)),
                _ => Err(EvalError::new(
                    EvalErrorKind::TypeError,
                    format!("cannot negate {}", val.type_name()),
                    Some(span),
                )),
            },
            UnaryOpKind::Not => match val {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(EvalError::new(
                    EvalErrorKind::TypeError,
                    format!("cannot apply ! to {}", val.type_name()),
                    Some(span),
                )),
            },
        }
    }

    /// Evaluate a function call.
    fn eval_call(
        &mut self,
        func_val: Value,
        ir_args: &[IrArg],
        span: Span,
    ) -> EvalResult<Value> {
        match func_val {
            Value::BuiltinFn { name, func } => {
                let args = self.eval_args_positional(ir_args)?;
                func(&args, self).map_err(|mut e| {
                    if e.span.is_none() {
                        e.span = Some(span);
                    }
                    e.message = format!("{name}: {}", e.message);
                    e
                })
            }
            Value::Function {
                params,
                body,
                closure_env,
            } => {
                let args = self.resolve_args(&params, ir_args, span)?;
                let mut call_env = closure_env;
                call_env.push_scope();
                for (param, val) in params.iter().zip(args.iter()) {
                    call_env.define(&param.name, val.clone());
                }
                let saved_env = std::mem::replace(&mut self.env, call_env);
                let result = self.eval_node(body);
                self.env = saved_env;
                result
            }
            _ => Err(EvalError::new(
                EvalErrorKind::NotCallable,
                format!("cannot call value of type {}", func_val.type_name()),
                Some(span),
            )),
        }
    }

    /// Call a value as a function with already-evaluated arguments.
    ///
    /// Used by builtins like `map` that need to invoke user-provided functions.
    pub fn call_value(
        &mut self,
        func: &Value,
        args: &[Value],
        span: Option<Span>,
    ) -> EvalResult<Value> {
        match func {
            Value::BuiltinFn { func: f, .. } => f(args, self),
            Value::Function {
                params,
                body,
                closure_env,
            } => {
                let mut call_env = closure_env.clone();
                call_env.push_scope();
                for (param, arg) in params.iter().zip(args.iter()) {
                    call_env.define(&param.name, arg.clone());
                }
                let saved_env = std::mem::replace(&mut self.env, call_env);
                let result = self.eval_node(*body);
                self.env = saved_env;
                result
            }
            _ => Err(EvalError::new(
                EvalErrorKind::NotCallable,
                format!("cannot call value of type {}", func.type_name()),
                span,
            )),
        }
    }

    /// Evaluate all arguments positionally (for builtins).
    fn eval_args_positional(&mut self, args: &[IrArg]) -> EvalResult<Vec<Value>> {
        args.iter()
            .map(|arg| self.eval_node(arg.value))
            .collect()
    }

    /// Resolve arguments to match function parameters, handling named args + defaults.
    fn resolve_args(
        &mut self,
        params: &[FnParam],
        ir_args: &[IrArg],
        span: Span,
    ) -> EvalResult<Vec<Value>> {
        let mut result = vec![None; params.len()];

        // First pass: positional args.
        let mut positional_idx = 0;
        for arg in ir_args {
            if let Some(ref name_spanned) = arg.name {
                // Named argument — find the param.
                let param_idx = params
                    .iter()
                    .position(|p| p.name == name_spanned.node)
                    .ok_or_else(|| {
                        EvalError::new(
                            EvalErrorKind::ArityMismatch,
                            format!("unknown parameter '{}'", name_spanned.node),
                            Some(name_spanned.span),
                        )
                    })?;
                let val = self.eval_node(arg.value)?;
                result[param_idx] = Some(val);
            } else {
                // Positional argument.
                if positional_idx >= params.len() {
                    return Err(EvalError::new(
                        EvalErrorKind::ArityMismatch,
                        format!(
                            "too many arguments: expected {}, got at least {}",
                            params.len(),
                            positional_idx + 1,
                        ),
                        Some(span),
                    ));
                }
                let val = self.eval_node(arg.value)?;
                result[positional_idx] = Some(val);
                positional_idx += 1;
            }
        }

        // Second pass: fill in defaults for missing args.
        for (i, param) in params.iter().enumerate() {
            if result[i].is_none() {
                if let Some(default_node) = param.default {
                    let val = self.eval_node(default_node)?;
                    result[i] = Some(val);
                } else {
                    return Err(EvalError::new(
                        EvalErrorKind::ArityMismatch,
                        format!("missing argument '{}'", param.name),
                        Some(span),
                    ));
                }
            }
        }

        Ok(result.into_iter().map(|v| v.unwrap()).collect())
    }

    /// Try to match a pattern against a value, returning bindings on success.
    fn match_pattern(&self, pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(vec![]),
            Pattern::Ident(name) => {
                // If the name matches an enum variant in scope, compare values.
                if let Some(env_val) = self.env.lookup(name)
                    && let Value::EnumVariant { .. } = env_val
                {
                    if self.values_equal(env_val, value) {
                        return Some(vec![]);
                    }
                    return None;
                }
                // Otherwise, bind the name.
                Some(vec![(name.clone(), value.clone())])
            }
            Pattern::Literal(lit_expr) => {
                // Evaluate the literal pattern by matching the AST expression.
                // Since patterns come from the parser, they should be simple literals.
                let lit_val = self.eval_pattern_literal(lit_expr)?;
                if self.values_equal(&lit_val, value) {
                    Some(vec![])
                } else {
                    None
                }
            }
        }
    }

    /// Evaluate a literal expression within a pattern.
    fn eval_pattern_literal(
        &self,
        expr: &covariant_syntax::Spanned<covariant_syntax::ast::Expr>,
    ) -> Option<Value> {
        use covariant_syntax::ast::Expr;
        match &expr.node {
            Expr::IntLit(n) => Some(Value::Int(*n)),
            Expr::FloatLit(f) => Some(Value::Float(*f)),
            Expr::BoolLit(b) => Some(Value::Bool(*b)),
            Expr::StringLit(s) => Some(Value::String(s.clone())),
            Expr::LengthLit(v, u) => Some(Value::Length(length_to_mm(*v, *u))),
            Expr::AngleLit(v, u) => Some(Value::Angle(angle_to_rad(*v, *u))),
            _ => None,
        }
    }

    /// Compare two values for equality.
    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => x == y,
            (Value::Float(x), Value::Float(y)) => x == y,
            (Value::Length(x), Value::Length(y)) => x == y,
            (Value::Angle(x), Value::Angle(y)) => x == y,
            (Value::Bool(x), Value::Bool(y)) => x == y,
            (Value::String(x), Value::String(y)) => x == y,
            (Value::Vec3(x), Value::Vec3(y)) => x == y,
            (Value::Unit, Value::Unit) => true,
            (
                Value::EnumVariant {
                    type_name: tn1,
                    variant: v1,
                },
                Value::EnumVariant {
                    type_name: tn2,
                    variant: v2,
                },
            ) => tn1 == tn2 && v1 == v2,
            _ => false,
        }
    }

    /// Compare two values, returning -1, 0, or 1.
    fn compare_values(&self, a: &Value, b: &Value, span: Span) -> EvalResult<i32> {
        match (a, b) {
            (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y) as i32),
            (Value::Float(x), Value::Float(y)) => {
                Ok(x.partial_cmp(y).map(|o| o as i32).unwrap_or(0))
            }
            (Value::Length(x), Value::Length(y)) => {
                Ok(x.partial_cmp(y).map(|o| o as i32).unwrap_or(0))
            }
            (Value::Angle(x), Value::Angle(y)) => {
                Ok(x.partial_cmp(y).map(|o| o as i32).unwrap_or(0))
            }
            (Value::Int(x), Value::Float(y)) => {
                let xf = *x as f64;
                Ok(xf.partial_cmp(y).map(|o| o as i32).unwrap_or(0))
            }
            (Value::Float(x), Value::Int(y)) => {
                let yf = *y as f64;
                Ok(x.partial_cmp(&yf).map(|o| o as i32).unwrap_or(0))
            }
            _ => Err(EvalError::new(
                EvalErrorKind::TypeError,
                format!(
                    "cannot compare {} and {}",
                    a.type_name(),
                    b.type_name()
                ),
                Some(span),
            )),
        }
    }

    /// Produce a type error for a binary operation.
    fn type_error_binop(&self, op: &str, lhs: &Value, rhs: &Value, span: Span) -> EvalError {
        EvalError::new(
            EvalErrorKind::TypeError,
            format!(
                "cannot apply '{op}' to {} and {}",
                lhs.type_name(),
                rhs.type_name()
            ),
            Some(span),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use covariant_geom::TruckKernel;
    use covariant_ir::node::IrNode;
    use covariant_syntax::Span;

    fn make_dag_with_roots(nodes: Vec<(IrNode, Span)>) -> Dag {
        let mut dag = Dag::new();
        let mut ids = Vec::new();
        for (node, span) in nodes {
            ids.push(dag.insert(node, span));
        }
        dag.set_roots(ids.clone());
        dag
    }

    fn eval_single(node: IrNode) -> EvalResult<Value> {
        let dag = make_dag_with_roots(vec![(node, Span::new(0, 1))]);
        let kernel = TruckKernel;
        eval(&dag, &kernel)
    }

    #[test]
    fn eval_int_lit() {
        let val = eval_single(IrNode::IntLit(42)).unwrap();
        assert!(matches!(val, Value::Int(42)));
    }

    #[test]
    fn eval_float_lit() {
        let val = eval_single(IrNode::FloatLit(3.14)).unwrap();
        assert!(matches!(val, Value::Float(f) if (f - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn eval_length_lit() {
        use covariant_syntax::ast::LengthUnit;
        let val = eval_single(IrNode::LengthLit(1.0, LengthUnit::Cm)).unwrap();
        assert!(matches!(val, Value::Length(l) if (l - 10.0).abs() < f64::EPSILON));
    }

    #[test]
    fn eval_bool_lit() {
        let val = eval_single(IrNode::BoolLit(true)).unwrap();
        assert!(matches!(val, Value::Bool(true)));
    }

    #[test]
    fn eval_string_lit() {
        let val = eval_single(IrNode::StringLit("hello".to_string())).unwrap();
        assert!(matches!(val, Value::String(ref s) if s == "hello"));
    }

    #[test]
    fn eval_addition_int() {
        use covariant_syntax::Spanned;
        let mut dag = Dag::new();
        let a = dag.insert(IrNode::IntLit(1), Span::new(0, 1));
        let b = dag.insert(IrNode::IntLit(2), Span::new(2, 3));
        let add = dag.insert(
            IrNode::BinOp {
                lhs: a,
                op: Spanned::new(BinOpKind::Add, Span::new(1, 2)),
                rhs: b,
            },
            Span::new(0, 3),
        );
        dag.set_roots(vec![add]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Int(3)));
    }

    #[test]
    fn eval_let_and_ident() {
        use covariant_syntax::Spanned;
        let mut dag = Dag::new();
        let val_node = dag.insert(IrNode::IntLit(42), Span::new(8, 10));
        let let_node = dag.insert(
            IrNode::Let {
                name: Spanned::new("x".to_string(), Span::new(4, 5)),
                ty: None,
                value: val_node,
            },
            Span::new(0, 10),
        );
        let ident = dag.insert(IrNode::Ident("x".to_string()), Span::new(11, 12));
        dag.set_roots(vec![let_node, ident]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Int(42)));
    }

    #[test]
    fn eval_fn_def_and_call() {
        use covariant_ir::node::{IrArg, IrParam};
        use covariant_syntax::Spanned;

        let mut dag = Dag::new();
        // fn double(x) { x * 2 }
        let param_x = dag.insert(IrNode::Ident("x".to_string()), Span::new(20, 21));
        let two = dag.insert(IrNode::IntLit(2), Span::new(24, 25));
        let mul = dag.insert(
            IrNode::BinOp {
                lhs: param_x,
                op: Spanned::new(BinOpKind::Mul, Span::new(22, 23)),
                rhs: two,
            },
            Span::new(20, 25),
        );
        let fn_def = dag.insert(
            IrNode::FnDef {
                name: Spanned::new("double".to_string(), Span::new(3, 9)),
                params: vec![IrParam {
                    name: Spanned::new("x".to_string(), Span::new(10, 11)),
                    ty: None,
                    default: None,
                    span: Span::new(10, 11),
                }],
                return_ty: None,
                body: mul,
            },
            Span::new(0, 27),
        );

        // double(5)
        let func_ref = dag.insert(IrNode::Ident("double".to_string()), Span::new(28, 34));
        let five = dag.insert(IrNode::IntLit(5), Span::new(35, 36));
        let call = dag.insert(
            IrNode::FnCall {
                func: func_ref,
                args: vec![IrArg {
                    name: None,
                    value: five,
                    span: Span::new(35, 36),
                }],
            },
            Span::new(28, 37),
        );

        dag.set_roots(vec![fn_def, call]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Int(10)));
    }

    #[test]
    fn eval_lambda() {
        use covariant_ir::node::{IrArg, IrParam};
        use covariant_syntax::Spanned;

        let mut dag = Dag::new();
        // let inc = |x| x + 1
        let x_ref = dag.insert(IrNode::Ident("x".to_string()), Span::new(15, 16));
        let one = dag.insert(IrNode::IntLit(1), Span::new(19, 20));
        let add = dag.insert(
            IrNode::BinOp {
                lhs: x_ref,
                op: Spanned::new(BinOpKind::Add, Span::new(17, 18)),
                rhs: one,
            },
            Span::new(15, 20),
        );
        let lambda = dag.insert(
            IrNode::Lambda {
                params: vec![IrParam {
                    name: Spanned::new("x".to_string(), Span::new(12, 13)),
                    ty: None,
                    default: None,
                    span: Span::new(12, 13),
                }],
                body: add,
            },
            Span::new(11, 20),
        );
        let let_inc = dag.insert(
            IrNode::Let {
                name: Spanned::new("inc".to_string(), Span::new(4, 7)),
                ty: None,
                value: lambda,
            },
            Span::new(0, 20),
        );
        // inc(10)
        let inc_ref = dag.insert(IrNode::Ident("inc".to_string()), Span::new(21, 24));
        let ten = dag.insert(IrNode::IntLit(10), Span::new(25, 27));
        let call = dag.insert(
            IrNode::FnCall {
                func: inc_ref,
                args: vec![IrArg {
                    name: None,
                    value: ten,
                    span: Span::new(25, 27),
                }],
            },
            Span::new(21, 28),
        );

        dag.set_roots(vec![let_inc, call]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Int(11)));
    }

    #[test]
    fn eval_if_true() {
        let mut dag = Dag::new();
        let cond = dag.insert(IrNode::BoolLit(true), Span::new(3, 7));
        let then = dag.insert(IrNode::IntLit(1), Span::new(10, 11));
        let else_ = dag.insert(IrNode::IntLit(2), Span::new(19, 20));
        let if_node = dag.insert(
            IrNode::If {
                cond,
                then_branch: then,
                else_branch: Some(else_),
            },
            Span::new(0, 21),
        );
        dag.set_roots(vec![if_node]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Int(1)));
    }

    #[test]
    fn eval_if_false() {
        let mut dag = Dag::new();
        let cond = dag.insert(IrNode::BoolLit(false), Span::new(3, 8));
        let then = dag.insert(IrNode::IntLit(1), Span::new(11, 12));
        let else_ = dag.insert(IrNode::IntLit(2), Span::new(20, 21));
        let if_node = dag.insert(
            IrNode::If {
                cond,
                then_branch: then,
                else_branch: Some(else_),
            },
            Span::new(0, 22),
        );
        dag.set_roots(vec![if_node]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Int(2)));
    }

    #[test]
    fn eval_block_scoping() {
        use covariant_syntax::Spanned;
        let mut dag = Dag::new();
        // let x = 1
        let one = dag.insert(IrNode::IntLit(1), Span::new(8, 9));
        let let_x = dag.insert(
            IrNode::Let {
                name: Spanned::new("x".to_string(), Span::new(4, 5)),
                ty: None,
                value: one,
            },
            Span::new(0, 9),
        );
        // { let x = 2; x }
        let two = dag.insert(IrNode::IntLit(2), Span::new(20, 21));
        let let_inner = dag.insert(
            IrNode::Let {
                name: Spanned::new("x".to_string(), Span::new(16, 17)),
                ty: None,
                value: two,
            },
            Span::new(12, 21),
        );
        let x_ref_inner = dag.insert(IrNode::Ident("x".to_string()), Span::new(23, 24));
        let block = dag.insert(
            IrNode::Block {
                stmts: vec![let_inner],
                tail: Some(x_ref_inner),
            },
            Span::new(10, 25),
        );
        // After block, x should still be 1
        let x_ref_outer = dag.insert(IrNode::Ident("x".to_string()), Span::new(26, 27));

        dag.set_roots(vec![let_x, block, x_ref_outer]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        // Last root is x_ref_outer = 1
        assert!(matches!(val, Value::Int(1)));
    }

    #[test]
    fn eval_data_constructor_and_field() {
        use covariant_ir::node::{IrField, IrFieldInit};
        use covariant_syntax::Spanned;
        use covariant_syntax::ast::Type;

        let mut dag = Dag::new();
        // data Rect { width: Length, height: Length }
        let data_def = dag.insert(
            IrNode::DataDef {
                name: Spanned::new("Rect".to_string(), Span::new(5, 9)),
                fields: vec![
                    IrField {
                        name: Spanned::new("width".to_string(), Span::new(12, 17)),
                        ty: Spanned::new(Type::Named("Length".to_string()), Span::new(19, 25)),
                        default: None,
                        span: Span::new(12, 25),
                    },
                    IrField {
                        name: Spanned::new("height".to_string(), Span::new(27, 33)),
                        ty: Spanned::new(Type::Named("Length".to_string()), Span::new(35, 41)),
                        default: None,
                        span: Span::new(27, 41),
                    },
                ],
            },
            Span::new(0, 43),
        );

        // let r = Rect { width = 10mm, height = 20mm }
        let w_val = dag.insert(
            IrNode::LengthLit(10.0, covariant_syntax::ast::LengthUnit::Mm),
            Span::new(60, 64),
        );
        let h_val = dag.insert(
            IrNode::LengthLit(20.0, covariant_syntax::ast::LengthUnit::Mm),
            Span::new(75, 79),
        );
        let ctor = dag.insert(
            IrNode::DataConstructor {
                name: Spanned::new("Rect".to_string(), Span::new(52, 56)),
                fields: vec![
                    IrFieldInit {
                        name: Spanned::new("width".to_string(), Span::new(59, 64)),
                        value: w_val,
                        span: Span::new(59, 68),
                    },
                    IrFieldInit {
                        name: Spanned::new("height".to_string(), Span::new(70, 76)),
                        value: h_val,
                        span: Span::new(70, 79),
                    },
                ],
            },
            Span::new(52, 81),
        );
        let let_r = dag.insert(
            IrNode::Let {
                name: Spanned::new("r".to_string(), Span::new(48, 49)),
                ty: None,
                value: ctor,
            },
            Span::new(44, 81),
        );

        // r.width
        let r_ref = dag.insert(IrNode::Ident("r".to_string()), Span::new(82, 83));
        let field_access = dag.insert(
            IrNode::FieldAccess {
                object: r_ref,
                field: Spanned::new("width".to_string(), Span::new(84, 89)),
            },
            Span::new(82, 89),
        );

        dag.set_roots(vec![data_def, let_r, field_access]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Length(l) if (l - 10.0).abs() < f64::EPSILON));
    }

    #[test]
    fn eval_unary_neg() {
        use covariant_syntax::Spanned;

        let mut dag = Dag::new();
        let five = dag.insert(IrNode::IntLit(5), Span::new(1, 2));
        let neg = dag.insert(
            IrNode::UnaryOp {
                op: Spanned::new(UnaryOpKind::Neg, Span::new(0, 1)),
                operand: five,
            },
            Span::new(0, 2),
        );
        dag.set_roots(vec![neg]);
        let kernel = TruckKernel;
        let val = eval(&dag, &kernel).unwrap();
        assert!(matches!(val, Value::Int(-5)));
    }

    #[test]
    fn eval_undefined_name_error() {
        let dag = make_dag_with_roots(vec![(
            IrNode::Ident("nonexistent".to_string()),
            Span::new(0, 11),
        )]);
        let kernel = TruckKernel;
        let err = eval(&dag, &kernel).unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::UndefinedName);
    }

    #[test]
    fn eval_division_by_zero() {
        use covariant_syntax::Spanned;

        let mut dag = Dag::new();
        let a = dag.insert(IrNode::IntLit(10), Span::new(0, 2));
        let b = dag.insert(IrNode::IntLit(0), Span::new(5, 6));
        let div = dag.insert(
            IrNode::BinOp {
                lhs: a,
                op: Spanned::new(BinOpKind::Div, Span::new(3, 4)),
                rhs: b,
            },
            Span::new(0, 6),
        );
        dag.set_roots(vec![div]);
        let kernel = TruckKernel;
        let err = eval(&dag, &kernel).unwrap_err();
        assert_eq!(err.kind, EvalErrorKind::DivisionByZero);
    }
}
