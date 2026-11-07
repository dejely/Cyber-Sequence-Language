//! Each lexical scope owns its bindings and links to its enclosing scope.

use std::collections::HashMap;

use crate::value::Value;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn enclosed(enclosing: Environment) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn into_enclosing(self) -> Option<Self> {
        self.enclosing.map(|environment| *environment)
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.values
            .get(name)
            .cloned()
            .or_else(|| self.enclosing.as_ref().and_then(|outer| outer.get(name)))
    }

    /// Assignment updates a binding; a miss must never create one.
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if let Some(binding) = self.values.get_mut(name) {
            *binding = value;
            true
        } else {
            self.enclosing
                .as_mut()
                .is_some_and(|outer| outer.assign(name, value))
        }
    }
}
