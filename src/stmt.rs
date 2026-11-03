//! Statements perform effects; their expression values are not echoed in files.

use crate::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Print(Expr),
    Expression(Expr),
}
