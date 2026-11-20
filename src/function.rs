//! Anything a CSL program can call, behind one interface.
//!
//! The call expression checks "is it callable?" and "does the argument count
//! match?" in one place, then hands off through this trait, so it never
//! branches on what kind of callable it holds.

use std::fmt;
use std::ops::ControlFlow;
use std::rc::Rc;

use crate::environment::{Environment, SharedEnvironment};
use crate::evaluator::{Interpreter, RuntimeError};
use crate::stmt::FunctionDecl;
use crate::token::Token;
use crate::value::Value;

pub trait Callable: fmt::Display {
    /// How many arguments a call must supply.
    fn arity(&self) -> usize;

    /// Runs the callable on arguments already evaluated at the call site.
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        call_site: &Token,
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
        _call_site: &Token,
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
        match interpreter.execute_block(&self.declaration.body, scope)? {
            ControlFlow::Break(value) => Ok(value),
            ControlFlow::Continue(()) => Ok(Value::Nil),
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme())
    }
}

/// Native functions registered in the global scope at interpreter startup.
pub(crate) enum NativeFunction {
    Len,
    TypeOf,
}

impl NativeFunction {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Len => "len",
            Self::TypeOf => "typeOf",
        }
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        1
    }

    fn call(
        &self,
        _interpreter: &mut Interpreter,
        arguments: Vec<Value>,
        call_site: &Token,
    ) -> Result<Value, RuntimeError> {
        // The evaluator verifies arity before calling any function.
        let argument = &arguments[0];
        match self {
            Self::Len => match argument {
                Value::String(text) => Ok(Value::Number(text.chars().count() as f64)),
                _ => Err(RuntimeError::new(call_site, "len expects a string.")),
            },
            Self::TypeOf => {
                let name = match argument {
                    Value::Number(_) => "number",
                    Value::String(_) => "string",
                    Value::Boolean(_) => "boolean",
                    Value::Nil => "nil",
                    Value::Callable(_) => "function",
                };
                Ok(Value::String(name.to_owned()))
            }
        }
    }
}

impl fmt::Display for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn {}>", self.name())
    }
}
