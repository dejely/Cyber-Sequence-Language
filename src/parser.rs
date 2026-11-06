//! Recursive descent parser for expressions and programs, with semicolon recovery.

use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::token::{Literal, Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize, // Index of the next token to consume.
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        // Keep lookahead safe even when callers supply an empty or unfinished stream.
        if tokens.last().map(Token::token_type) != Some(&TokenType::Eof) {
            let line = tokens.last().map_or(1, Token::line);
            tokens.push(Token::new(TokenType::Eof, String::new(), None, line));
        }
        Self { tokens, current: 0 }
    }

    /// Parses one complete REPL expression, allowing a final semicolon.
    pub fn parse(&mut self) -> Result<Expr, String> {
        let expression = self.expression()?;
        self.match_token(&[TokenType::Semicolon]);
        if !self.is_at_end() {
            return Err(self.error("Expect end of expression."));
        }
        Ok(expression)
    }

    /// Parses a file of semicolon-separated expressions and collects errors.
    pub fn parse_all(&mut self) -> Result<Vec<Expr>, String> {
        let mut expressions = Vec::new();
        let mut errors = Vec::new();

        loop {
            match self.expression() {
                Ok(expression) => {
                    if self.match_token(&[TokenType::Semicolon]) || self.is_at_end() {
                        expressions.push(expression);
                    } else {
                        errors.push(self.error("Expect ';' between expressions."));
                        self.synchronize();
                    }
                }
                Err(message) => {
                    errors.push(message);
                    self.synchronize();
                }
            }
            if self.is_at_end() {
                break;
            }
        }

        if errors.is_empty() {
            Ok(expressions)
        } else {
            Err(errors.join("\n"))
        }
    }

    /// Script mode requires a semicolon after every statement, including the last.
    /// Parse everything before execution so syntax errors cannot cause side effects.
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(message) => {
                    errors.push(message);
                    self.synchronize();
                }
            }
        }
        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors.join("\n"))
        }
    }

    /// The prompt accepts a declaration, print statement, or one expression.
    pub fn parse_repl_statement(&mut self) -> Result<Stmt, String> {
        if !self.check(&TokenType::Print) && !self.check(&TokenType::Var) {
            return self.parse().map(Stmt::Expression);
        }
        let statement = self.declaration()?;
        if !self.is_at_end() {
            return Err(self.error("Expect end of statement."));
        }
        Ok(statement)
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[TokenType::Var]) {
            let name = self
                .consume(&TokenType::Identifier, "Expect variable name.")?
                .clone();
            let initializer = if self.match_token(&[TokenType::Equal]) {
                Some(self.expression()?)
            } else {
                None
            };
            self.consume(
                &TokenType::Semicolon,
                "Expect ';' after variable declaration.",
            )?;
            return Ok(Stmt::Var { name, initializer });
        }
        self.statement()
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[TokenType::Print]) {
            let expression = self.expression()?;
            self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
            return Ok(Stmt::Print(expression));
        }
        let expression = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expression))
    }

    pub fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expression = self.equality()?;
        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            // Recursing on the right makes a = b = value right associative.
            let value = self.assignment()?;
            if let Expr::Variable(name) = expression {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(format!(
                "[line {}] Error at '{}': Invalid assignment target.",
                equals.line(),
                equals.lexeme()
            ));
        }
        Ok(expression)
    }

    pub fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;

            // Fold each operator into the left side to preserve associativity.
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub fn comparison(&mut self) -> Result<Expr, String> {
        // Start at the next tighter precedence level; self-recursion never advances.
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;

            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    // iterator here for ast:
    pub fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    pub fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }
        self.primary()
    }

    pub fn primary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous().clone()));
        }
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Literal::Boolean(false)));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Literal::Boolean(true)));
        }
        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Literal::Null));
        }
        if self.check(&TokenType::Number) || self.check(&TokenType::String) {
            let literal = self
                .peek()
                .literal()
                .cloned()
                .ok_or_else(|| self.error("Expect a literal value."))?;
            self.advance();
            return Ok(Expr::Literal(literal));
        }
        if self.match_token(&[TokenType::LeftParen]) {
            let expression = self.expression()?;
            self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expression)));
        }
        Err(self.error("Expect expression."))
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, String> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        self.peek().token_type() == token_type
    }

    fn advance(&mut self) -> &Token {
        let index = self.current;
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[index]
    }

    fn is_at_end(&self) -> bool {
        self.check(&TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error(&self, message: &str) -> String {
        let token = self.peek();
        let location = if self.is_at_end() {
            " at end".to_string()
        } else {
            format!(" at '{}'", token.lexeme())
        };
        format!("[line {}] Error{location}: {message}", token.line())
    }

    fn synchronize(&mut self) {
        // Semicolons are file-expression boundaries; newlines remain whitespace.
        while !self.is_at_end() {
            if self.advance().token_type() == &TokenType::Semicolon {
                return;
            }
        }
    }
}
