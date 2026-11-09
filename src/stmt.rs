//! Statements perform effects; their expression values are not echoed in files.

use crate::expr::Expr;
use crate::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Print(Expr),
    Expression(Expr),
    /// Branches are statements, so `else if` needs no extra grammar rule.
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
}
