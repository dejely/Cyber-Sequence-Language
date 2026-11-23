//! Statements perform effects; their expression values are not echoed in files.

use std::rc::Rc;

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
    Return(Option<Expr>),
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
    /// Shared, so each function value points at its declaration instead of
    /// copying the body every time the declaration runs.
    Function(Rc<FunctionDecl>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
}

/// A top-level event rule, separate from executable script statements.
#[derive(Debug, Clone, PartialEq)]
pub struct WatchDecl {
    pub name: Token,
    pub source: String,
    pub kind: String,
    pub group_field: String,
    pub window: u32,
    pub threshold: u32,
    pub alerts: Vec<String>,
}
