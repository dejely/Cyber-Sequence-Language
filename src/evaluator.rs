//! Tree-walking evaluator: a post-order match over expression nodes.

use std::fmt;

use crate::expr::Expr;
use crate::token::{Literal, Token, TokenType};
use crate::value::Value;

/// A failure detected while evaluating; the token supplies the line number.
#[derive(Debug)]
pub struct RuntimeError {
    operator: Token,
    message: &'static str,
}

impl RuntimeError {
    fn new(operator: &Token, message: &'static str) -> Self {
        Self {
            operator: operator.clone(),
            message,
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

/// Evaluates children before the node itself, so `?` unwinds the first error
/// to the single caller that decides between exit 70 and the next REPL prompt.
pub fn evaluate(expression: &Expr) -> Result<Value, RuntimeError> {
    match expression {
        Expr::Literal(literal) => Ok(match literal {
            Literal::Number(value) => Value::Number(*value),
            Literal::String(value) => Value::String(value.clone()),
            Literal::Boolean(value) => Value::Boolean(*value),
            Literal::Null => Value::Nil,
        }),
        Expr::Grouping(inner) => evaluate(inner),
        Expr::Unary { operator, right } => {
            let right = evaluate(right)?;
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
            let left = evaluate(left)?;
            let right = evaluate(right)?;
            binary(operator, left, right)
        }
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
