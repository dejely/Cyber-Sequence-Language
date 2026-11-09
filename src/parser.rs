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

    /// Script mode requires a semicolon after declarations, prints, and expressions.
    /// Parse everything before execution so syntax errors cannot cause side effects.
    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, String> {
        self.statement_list(false)
    }

    fn statement_list(&mut self, in_block: bool) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() && !(in_block && self.check(&TokenType::RightBrace)) {
            let start = self.current;
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(message) => {
                    errors.push(message);
                    self.synchronize_statement(start, in_block);
                }
            }
        }
        if in_block
            && let Err(message) = self.consume(&TokenType::RightBrace, "Expect '}' after block.")
        {
            errors.push(message);
        }
        if errors.is_empty() {
            Ok(statements)
        } else {
            Err(errors.join("\n"))
        }
    }

    /// The prompt accepts a declaration, print statement, block, or one expression.
    pub fn parse_repl_statement(&mut self) -> Result<Stmt, String> {
        if !self.at_statement_keyword() {
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
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return self.statement_list(true).map(Stmt::Block);
        }
        if self.match_token(&[TokenType::Print]) {
            let expression = self.expression()?;
            self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
            return Ok(Stmt::Print(expression));
        }
        let expression = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after expression.")?;
        Ok(Stmt::Expression(expression))
    }

    /// The dangling `else` binds to the nearest `if`: the innermost call is the
    /// one still parsing when `else` arrives, so it claims the branch.
    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after condition.")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
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

    /// Tokens that can only begin a statement. The REPL treats them as
    /// statements rather than echoed expressions, and recovery restarts at them.
    fn at_statement_keyword(&self) -> bool {
        matches!(
            self.peek().token_type(),
            TokenType::Var
                | TokenType::Print
                | TokenType::LeftBrace
                | TokenType::If
                | TokenType::While
        )
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

    fn synchronize_statement(&mut self, start: usize, in_block: bool) {
        // Always advance after a failure at the same token, except when the
        // closing brace belongs to the caller. Expression modes keep their
        // original semicolon-only recovery above.
        if self.current == start && !self.is_at_end() {
            if in_block && self.check(&TokenType::RightBrace) {
                return;
            }
            self.advance();
        }
        while !self.is_at_end() {
            if self.current > 0
                && matches!(
                    self.previous().token_type(),
                    TokenType::Semicolon | TokenType::RightBrace
                )
            {
                return;
            }
            if self.at_statement_keyword() {
                return;
            }
            match self.peek().token_type() {
                TokenType::RightBrace => {
                    if !in_block {
                        self.advance();
                    }
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }
}
