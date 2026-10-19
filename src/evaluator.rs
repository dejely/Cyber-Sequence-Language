use crate::expr::Expr;
use crate::token::{Literal, Token, TokenType};
use crate::value::Value;

pub fn evaluate(expression: &Expr) -> Result<Value, String> {
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
            match (operator.token_type(), right) {
                (TokenType::Minus, Value::Number(value)) => number(-value, operator),
                (TokenType::Bang, value) => Ok(Value::Boolean(!value.is_truthy())),
                (TokenType::Minus, _) => Err(error(operator, "Operand must be a number.")),
                _ => Err(error(operator, "Unsupported unary operator.")),
            }
        }
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            let left = evaluate(left)?;
            let right = evaluate(right)?;
            match operator.token_type() {
                TokenType::EqualEqual => return Ok(Value::Boolean(left == right)),
                TokenType::BangEqual => return Ok(Value::Boolean(left != right)),
                _ => {}
            }
            match (left, operator.token_type(), right) {
                (Value::String(left), TokenType::Plus, Value::String(right)) => {
                    Ok(Value::String(left + &right))
                }
                (Value::Number(left), op, Value::Number(right)) => match op {
                    TokenType::Plus => number(left + right, operator),
                    TokenType::Minus => number(left - right, operator),
                    TokenType::Star => number(left * right, operator),
                    TokenType::Slash if right == 0.0 => Err(error(operator, "Division by zero.")),
                    TokenType::Slash => number(left / right, operator),
                    TokenType::Greater => Ok(Value::Boolean(left > right)),
                    TokenType::GreaterEqual => Ok(Value::Boolean(left >= right)),
                    TokenType::Less => Ok(Value::Boolean(left < right)),
                    TokenType::LessEqual => Ok(Value::Boolean(left <= right)),
                    _ => Err(error(operator, "Unsupported binary operator.")),
                },
                (_, TokenType::Plus, _) => Err(error(
                    operator,
                    "Operands must be two numbers or two strings.",
                )),
                _ => Err(error(operator, "Operands must be numbers.")),
            }
        }
    }
}

fn error(operator: &Token, message: &str) -> String {
    format!(
        "[line {}] Runtime error at '{}': {message}",
        operator.line(),
        operator.lexeme()
    )
}

fn number(value: f64, operator: &Token) -> Result<Value, String> {
    if value.is_finite() {
        Ok(Value::Number(value))
    } else {
        Err(error(operator, "Numeric result is not finite."))
    }
}
