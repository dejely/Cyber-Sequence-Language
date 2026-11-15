//! Each lexical scope owns its bindings and links to its enclosing scope.
//!
//! Scopes are shared (`Rc<RefCell<_>>`) rather than owned by their inner
//! scope, so something other than the running block can keep a scope alive.
//! A function value will hold its defining scope this way, which lets the
//! scope outlive the block or call that created it.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::value::Value;

pub type SharedEnvironment = Rc<RefCell<Environment>>;

#[derive(Default)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<SharedEnvironment>,
}

impl Environment {
    pub fn enclosed(enclosing: SharedEnvironment) -> SharedEnvironment {
        Rc::new(RefCell::new(Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned().or_else(|| {
            self.enclosing
                .as_ref()
                .and_then(|outer| outer.borrow().get(name))
        })
    }

    /// Assignment updates a binding; a miss must never create one.
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if let Some(binding) = self.values.get_mut(name) {
            *binding = value;
            true
        } else {
            self.enclosing
                .as_ref()
                .is_some_and(|outer| outer.borrow_mut().assign(name, value))
        }
    }
}
