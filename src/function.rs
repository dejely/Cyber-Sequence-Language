//! Anything a CSL program can call, behind one interface.
//!
//! The call expression checks "is it callable?" and "does the argument count
//! match?" in one place, then hands off through this trait, so it never
//! branches on what kind of callable it holds.

use std::fmt;
use std::rc::Rc;

use crate::environment::{Environment, SharedEnvironment};
use crate::evaluator::{Interpreter, RuntimeError};
use crate::stmt::FunctionDecl;
use crate::value::Value;

pub trait Callable: fmt::Display {
    /// How many arguments a call must supply.
    fn arity(&self) -> usize;

    /// Runs the callable on arguments already evaluated at the call site.
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError>;
}

/// Function values compare by identity: two declarations with the same text
/// are different functions, and a function always equals itself.
impl PartialEq for dyn Callable {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::addr_eq(self, other)
    }
}

/// Printing the closure would walk the scope chain, which can lead back to
/// this function, so debug output stays as short as the display form.
impl fmt::Debug for dyn Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// A function declared in CSL source.
pub struct Function {
    declaration: Rc<FunctionDecl>,
    /// The scope active where the declaration ran. Holding it here is what
    /// keeps it alive after its block or call ends.
    closure: SharedEnvironment,
}

impl Function {
    pub fn new(declaration: Rc<FunctionDecl>, closure: SharedEnvironment) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        // A fresh scope per call keeps recursive calls from sharing parameters.
        // Its parent is the closure, NOT the caller's scope: that one link is
        // what makes lookup lexical instead of dynamic.
        let scope = Environment::enclosed(Rc::clone(&self.closure));
        {
            let mut scope = scope.borrow_mut();
            for (param, argument) in self.declaration.params.iter().zip(arguments) {
                scope.define(param.lexeme(), argument);
            }
        }
        interpreter.execute_block(&self.declaration.body, scope)?;
        // `return` arrives in Week 3; until then a call always produces nil.
        Ok(Value::Nil)
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme())
    }
}
