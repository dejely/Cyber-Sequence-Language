use crate::token::{Literal, Token, TokenType};

pub struct Parser {
    literal: Literal 
    unary: Unary
    binary: Binary
    grouping: Grouping
}