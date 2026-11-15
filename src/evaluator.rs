//! Tree-walking evaluator: a post-order match over expression nodes.

use std::fmt;
use std::rc::Rc;

use crate::environment::{Environment, SharedEnvironment};
use crate::expr::Expr;
use crate::function::Function;
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};
use crate::value::Value;

/// A failure detected while evaluating; the token supplies the line number.
#[derive(Debug)]
pub struct RuntimeError {
    operator: Token,
    message: String,
}

impl RuntimeError {
    pub(crate) fn new(operator: &Token, message: impl Into<String>) -> Self {
        Self {
            operator: operator.clone(),
            message: message.into(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[line {}] Runtime error at '{}': {}",
            self.operator.line(),
            self.operator.lexeme(),
            self.message
        )
    }
}

/// Owns the bindings shared by every statement and expression in a session.
#[derive(Default)]
pub struct Interpreter {
    environment: SharedEnvironment,
}

impl Interpreter {
    /// Evaluate left before right; `?` propagates the first runtime error.
    pub fn evaluate(&mut self, expression: &Expr) -> Result<Value, RuntimeError> {
        match expression {
            Expr::Literal(literal) => Ok(match literal {
                Literal::Number(value) => Value::Number(*value),
                Literal::String(value) => Value::String(value.clone()),
                Literal::Boolean(value) => Value::Boolean(*value),
                Literal::Null => Value::Nil,
            }),
            Expr::Variable(name) => self.environment.borrow().get(name.lexeme()).ok_or_else(|| {
                RuntimeError::new(name, format!("Undefined variable '{}'.", name.lexeme()))
            }),
            Expr::Assign { name, value } => {
                let value = self.evaluate(value)?;
                if !self
                    .environment
                    .borrow_mut()
                    .assign(name.lexeme(), value.clone())
                {
                    return Err(RuntimeError::new(
                        name,
                        format!("Undefined variable '{}'.", name.lexeme()),
                    ));
                }
                Ok(value)
            }
            Expr::Grouping(inner) => self.evaluate(inner),
            Expr::Unary { operator, right } => {
                let right = self.evaluate(right)?;
                match operator.token_type() {
                    TokenType::Minus => number(-check_number_operand(operator, &right)?, operator),
                    TokenType::Bang => Ok(Value::Boolean(!right.is_truthy())),
                    _ => Err(RuntimeError::new(operator, "Unsupported unary operator.")),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // Left before right is a language decision, observable once operands have effects.
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                binary(operator, left, right)
            }
            // Callee first, then arguments left to right, all in the caller's
            // scope. Both checks live here, so every kind of callable gets them.
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(callee)?;
                let arguments = arguments
                    .iter()
                    .map(|argument| self.evaluate(argument))
                    .collect::<Result<Vec<_>, _>>()?;
                let Value::Callable(function) = callee else {
                    return Err(RuntimeError::new(paren, "Can only call functions."));
                };
                if arguments.len() != function.arity() {
                    return Err(RuntimeError::new(
                        paren,
                        format!(
                            "Expected {} arguments but got {}.",
                            function.arity(),
                            arguments.len()
                        ),
                    ));
                }
                function.call(self, arguments)
            }
            // Short-circuiting is semantics, not an optimization: the right
            // operand runs only when the left one cannot decide the result.
            // The deciding operand itself is the result, so `nil or "x"` is "x".
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(left)?;
                let decided = match operator.token_type() {
                    TokenType::Or => left.is_truthy(),
                    _ => !left.is_truthy(),
                };
                if decided {
                    Ok(left)
                } else {
                    self.evaluate(right)
                }
            }
        }
    }

    /// Statements share the expression evaluator and its runtime errors.
    pub fn execute(&mut self, statement: &Stmt) -> Result<(), RuntimeError> {
        match statement {
            Stmt::Block(statements) => {
                let scope = Environment::enclosed(Rc::clone(&self.environment));
                self.execute_block(statements, scope)?;
            }
            // Annotations describe declarations; they do not change execution.
            Stmt::Var {
                name, initializer, ..
            } => {
                // Define only after success; initializer assignments keep their own effects.
                let value = match initializer {
                    Some(expression) => self.evaluate(expression)?,
                    None => Value::Nil,
                };
                self.environment.borrow_mut().define(name.lexeme(), value);
            }
            Stmt::Print(expression) => println!("{}", self.evaluate(expression)?),
            Stmt::Expression(expression) => {
                self.evaluate(expression)?;
            }
            // Only the selected branch runs; the condition uses Lab 3 truthiness.
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
            }
            // The function captures the scope it is declared in. Its name is
            // bound in that same scope, so the body can call itself.
            Stmt::Function(declaration) => {
                let function = Function::new(Rc::clone(declaration), Rc::clone(&self.environment));
                self.environment.borrow_mut().define(
                    declaration.name.lexeme(),
                    Value::Callable(Rc::new(function)),
                );
            }
            // The condition is re-evaluated before every iteration. A braced body
            // is a block, so each iteration gets a fresh local scope.
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
            }
        }
        Ok(())
    }

    /// Runs `statements` in `scope`, then reinstates the previous scope.
    /// The result is saved before propagating errors so the caller's scope
    /// comes back on success and on a runtime error, at every nesting level.
    pub fn execute_block(
        &mut self,
        statements: &[Stmt],
        scope: SharedEnvironment,
    ) -> Result<(), RuntimeError> {
        let previous = std::mem::replace(&mut self.environment, scope);
        let result = statements
            .iter()
            .try_for_each(|statement| self.execute(statement));
        self.environment = previous;
        result
    }
}

fn binary(operator: &Token, left: Value, right: Value) -> Result<Value, RuntimeError> {
    match operator.token_type() {
        // Equality accepts any pair; mismatched variants are simply unequal.
        TokenType::EqualEqual => Ok(Value::Boolean(left == right)),
        TokenType::BangEqual => Ok(Value::Boolean(left != right)),
        TokenType::Plus => match (left, right) {
            (Value::Number(left), Value::Number(right)) => number(left + right, operator),
            (Value::String(left), Value::String(right)) => Ok(Value::String(left + &right)),
            _ => Err(RuntimeError::new(
                operator,
                "Operands must be two numbers or two strings.",
            )),
        },
        TokenType::Minus => {
            let (left, right) = check_number_operands(operator, &left, &right)?;
            number(left - right, operator)
        }
        TokenType::Star => {
            let (left, right) = check_number_operands(operator, &left, &right)?;
            number(left * right, operator)
        }
        TokenType::Slash => {
            let (left, right) = check_number_operands(operator, &left, &right)?;
            if right == 0.0 {
                return Err(RuntimeError::new(operator, "Division by zero."));
            }
            number(left / right, operator)
        }
        TokenType::Greater => compare(operator, &left, &right, |l, r| l > r),
        TokenType::GreaterEqual => compare(operator, &left, &right, |l, r| l >= r),
        TokenType::Less => compare(operator, &left, &right, |l, r| l < r),
        TokenType::LessEqual => compare(operator, &left, &right, |l, r| l <= r),
        _ => Err(RuntimeError::new(operator, "Unsupported binary operator.")),
    }
}

fn compare(
    operator: &Token,
    left: &Value,
    right: &Value,
    ordering: fn(f64, f64) -> bool,
) -> Result<Value, RuntimeError> {
    let (left, right) = check_number_operands(operator, left, right)?;
    Ok(Value::Boolean(ordering(left, right)))
}

fn check_number_operand(operator: &Token, operand: &Value) -> Result<f64, RuntimeError> {
    match operand {
        Value::Number(value) => Ok(*value),
        _ => Err(RuntimeError::new(operator, "Operand must be a number.")),
    }
}

fn check_number_operands(
    operator: &Token,
    left: &Value,
    right: &Value,
) -> Result<(f64, f64), RuntimeError> {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => Ok((*left, *right)),
        _ => Err(RuntimeError::new(operator, "Operands must be numbers.")),
    }
}

/// Rejects overflow to infinity so every printed number is finite.
fn number(value: f64, operator: &Token) -> Result<Value, RuntimeError> {
    if value.is_finite() {
        Ok(Value::Number(value))
    } else {
        Err(RuntimeError::new(operator, "Numeric result is not finite."))
    }
}
