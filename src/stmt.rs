//! Statements perform effects; their expression values are not echoed in files.

use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Print(Expr),
    Expression(Expr),
}
