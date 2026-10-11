//! Expression nodes and their parenthesized AST representation.

use std::fmt;

use crate::token::{Literal, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Literal(Literal::Number(value)) => write!(f, "{value:?}"),
            Self::Literal(Literal::String(value)) => write!(f, "{value}"),
            Self::Literal(Literal::Boolean(value)) => write!(f, "{value}"),
            Self::Literal(Literal::Null) => write!(f, "nil"),
            Self::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme()),
            Self::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {left} {right})", operator.lexeme())
            }
            Self::Grouping(expression) => write!(f, "(group {expression})"),
        }
    }
}
