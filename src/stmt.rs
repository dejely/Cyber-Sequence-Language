//! Statements perform effects; their expression values are not echoed in files.

use crate::expr::Expr;
use crate::token::{Literal, Token};

/// Validated declaration metadata, kept in source order and never evaluated.
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    pub name: Token,
    pub arguments: Vec<Literal>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Var {
        annotations: Vec<Annotation>,
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
    /// Also the target of `for` desugaring, so the evaluator has one loop form.
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}
