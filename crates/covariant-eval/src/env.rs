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
