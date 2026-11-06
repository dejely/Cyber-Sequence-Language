//! Week 2 bindings live for one file execution or one REPL session.

use std::collections::HashMap;

use crate::value::Value;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }

    /// Assignment updates a binding; a miss must never create one.
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if let Some(binding) = self.values.get_mut(name) {
            *binding = value;
            true
        } else {
            false
        }
    }
}
