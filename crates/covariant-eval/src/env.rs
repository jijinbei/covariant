//! Scoped symbol table for the COVARIANT evaluator.

use std::collections::HashMap;

use crate::value::Value;

/// A scoped environment (symbol table).
///
/// Scopes are pushed/popped as blocks and function bodies are entered/exited.
/// Lookup searches from the innermost scope outward.
#[derive(Debug, Clone)]
pub struct Env {
    scopes: Vec<HashMap<String, Value>>,
}

impl Env {
    /// Create a new environment with a single empty scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Push a new empty scope (e.g., entering a block or function body).
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the innermost scope (e.g., leaving a block or function body).
    ///
    /// Panics if the last remaining scope is popped.
    pub fn pop_scope(&mut self) {
        assert!(self.scopes.len() > 1, "cannot pop the global scope");
        self.scopes.pop();
    }

    /// Define a binding in the current (innermost) scope.
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(name.into(), value);
    }

    /// Look up a name, searching from the innermost scope outward.
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_and_lookup() {
        let mut env = Env::new();
        env.define("x", Value::Int(42));
        let val = env.lookup("x").unwrap();
        assert!(matches!(val, Value::Int(42)));
    }

    #[test]
    fn inner_scope_shadows_outer() {
        let mut env = Env::new();
        env.define("x", Value::Int(1));
        env.push_scope();
        env.define("x", Value::Int(2));
        let val = env.lookup("x").unwrap();
        assert!(matches!(val, Value::Int(2)));
    }

    #[test]
    fn pop_scope_restores_outer() {
        let mut env = Env::new();
        env.define("x", Value::Int(1));
        env.push_scope();
        env.define("x", Value::Int(2));
        env.pop_scope();
        let val = env.lookup("x").unwrap();
        assert!(matches!(val, Value::Int(1)));
    }

    #[test]
    fn undefined_returns_none() {
        let env = Env::new();
        assert!(env.lookup("x").is_none());
    }

    #[test]
    fn multiple_bindings_same_scope() {
        let mut env = Env::new();
        env.define("a", Value::Int(1));
        env.define("b", Value::Float(2.0));
        env.define("c", Value::Bool(true));
        assert!(matches!(env.lookup("a"), Some(Value::Int(1))));
        assert!(matches!(env.lookup("b"), Some(Value::Float(f)) if (*f - 2.0).abs() < f64::EPSILON));
        assert!(matches!(env.lookup("c"), Some(Value::Bool(true))));
    }

    #[test]
    fn inner_scope_sees_outer() {
        let mut env = Env::new();
        env.define("x", Value::Int(42));
        env.push_scope();
        let val = env.lookup("x").unwrap();
        assert!(matches!(val, Value::Int(42)));
    }

    #[test]
    fn rebind_in_same_scope_overwrites() {
        let mut env = Env::new();
        env.define("x", Value::Int(1));
        env.define("x", Value::Int(2));
        assert!(matches!(env.lookup("x"), Some(Value::Int(2))));
    }

    #[test]
    #[should_panic(expected = "cannot pop the global scope")]
    fn pop_global_scope_panics() {
        let mut env = Env::new();
        env.pop_scope();
    }
}
