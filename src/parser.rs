//! Recursive descent parser for expressions and programs, with semicolon recovery.

mod watch;

#[cfg(test)]
#[path = "../tests/parser/annotations.rs"]
mod annotation_tests;
#[cfg(test)]
#[path = "../tests/parser/watch.rs"]
mod watch_tests;

use std::collections::HashSet;
use std::rc::Rc;

use crate::expr::Expr;
use crate::stmt::{AbilityDecl, Annotation, FunctionDecl, Stmt, WatchDecl};
use crate::token::{Literal, Token, TokenType};

#[derive(Debug, Default)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub watches: Vec<WatchDecl>,
    pub abilities: Vec<AbilityDecl>,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize, // Index of the next token to consume.
    function_depth: usize,
    in_ability: bool,
}

impl Parser {
    pub fn new(mut tokens: Vec<Token>) -> Self {
        // Keep lookahead safe even when callers supply an empty or unfinished stream.
        if tokens.last().map(Token::token_type) != Some(&TokenType::Eof) {
            let line = tokens.last().map_or(1, Token::line);
            tokens.push(Token::new(TokenType::Eof, String::new(), None, line));
        }
        Self {
            tokens,
            current: 0,
            function_depth: 0,
            in_ability: false,
        }
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
    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut program = Program::default();
        let mut watch_names = HashSet::new();
        let mut ability_names = HashSet::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            let start = self.current;
            if self.check(&TokenType::Watch) {
                match self.watch_declaration() {
                    Ok(watch) if watch_names.insert(watch.name.lexeme().to_owned()) => {
                        program.watches.push(watch);
                    }
                    Ok(watch) => errors.push(Self::error_at(&watch.name, "Duplicate watch name.")),
                    Err(message) => {
                        errors.push(message);
                        self.synchronize_domain_declaration(start);
                    }
                }
            } else if self.check(&TokenType::Ability) {
                match self.ability_declaration() {
                    Ok(ability) if ability_names.insert(ability.name.lexeme().to_owned()) => {
                        program.abilities.push(ability);
                    }
                    Ok(ability) => {
                        errors.push(Self::error_at(&ability.name, "Duplicate ability name."))
                    }
                    Err(message) => {
                        errors.push(message);
                        self.synchronize_domain_declaration(start);
                    }
                }
            } else {
                match self.declaration() {
                    Ok(statement) => program.statements.push(statement),
                    Err(message) => {
                        errors.push(message);
                        self.synchronize_statement(start, false);
                    }
                }
            }
        }
        if errors.is_empty() {
            Ok(program)
        } else {
            Err(errors.join("\n"))
        }
    }

    fn block_statements(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() && !self.check(&TokenType::RightBrace) {
            let start = self.current;
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(message) => {
                    errors.push(message);
                    self.synchronize_statement(start, true);
                }
            }
        }
        if let Err(message) = self.consume(&TokenType::RightBrace, "Expect '}' after block.") {
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
        let annotations = self.annotations()?;
        if self.match_token(&[TokenType::Var]) {
            return self.var_declaration(annotations);
        }
        // Annotations are `var`-only, so this check runs before `fun` is tried.
        if !annotations.is_empty() {
            return Err(self.error("Annotations must precede a 'var' declaration."));
        }
        if self.match_token(&[TokenType::Fun]) {
            return self.function_declaration();
        }
        self.statement()
    }

    /// Only built-in metadata is accepted. Arguments are strings, never expressions.
    fn annotations(&mut self) -> Result<Vec<Annotation>, String> {
        let mut annotations: Vec<Annotation> = Vec::new();
        while self.match_token(&[TokenType::At]) {
            if !self.check(&TokenType::Identifier) {
                return Err(self.error("Expect annotation name after '@'."));
            }
            if !matches!(self.peek().lexeme(), "deprecated" | "doc") {
                return Err(self.error(&format!("Unknown annotation '@{}'.", self.peek().lexeme())));
            }
            if annotations
                .iter()
                .any(|annotation| annotation.name.lexeme() == self.peek().lexeme())
            {
                return Err(self.error("Duplicate annotation on declaration."));
            }
            let name = self.advance().clone();
            let mut arguments = Vec::new();
            if self.match_token(&[TokenType::LeftParen]) {
                if !self.check(&TokenType::RightParen) {
                    let argument = self
                        .consume(
                            &TokenType::String,
                            "Expect string literal annotation argument.",
                        )?
                        .literal()
                        .cloned()
                        .ok_or_else(|| self.error("Expect string literal annotation argument."))?;
                    arguments.push(argument);
                }
                self.consume(
                    &TokenType::RightParen,
                    "Expect ')' after annotation argument (at most one string).",
                )?;
            }
            if name.lexeme() == "doc" && arguments.len() != 1 {
                return Err(format!(
                    "[line {}] Error at 'doc': Annotation '@doc' requires one string argument.",
                    name.line()
                ));
            }
            annotations.push(Annotation { name, arguments });
        }
        Ok(annotations)
    }

    /// Parses the rest of `var name (= value)? ;` after the `var` keyword.
    fn var_declaration(&mut self, annotations: Vec<Annotation>) -> Result<Stmt, String> {
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
        Ok(Stmt::Var {
            annotations,
            name,
            initializer,
        })
    }

    /// Parses the rest of `fun name(params) { body }` after the `fun` keyword.
    fn function_declaration(&mut self) -> Result<Stmt, String> {
        let name = self
            .consume(&TokenType::Identifier, "Expect function name.")?
            .clone();
        self.consume(&TokenType::LeftParen, "Expect '(' after function name.")?;
        let mut params: Vec<Token> = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                let param = self
                    .consume(&TokenType::Identifier, "Expect parameter name.")?
                    .clone();
                // Parameters share one scope, so a repeat would silently
                // replace an argument; reject it before anything runs.
                if params.iter().any(|seen| seen.lexeme() == param.lexeme()) {
                    return Err(format!(
                        "[line {}] Error at '{}': Duplicate parameter name.",
                        param.line(),
                        param.lexeme()
                    ));
                }
                params.push(param);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(&TokenType::RightParen, "Expect ')' after parameters.")?;
        self.consume(&TokenType::LeftBrace, "Expect '{' before function body.")?;
        self.function_depth += 1;
        let body = self.block_statements();
        self.function_depth -= 1;
        let body = body?;
        Ok(Stmt::Function(Rc::new(FunctionDecl { name, params, body })))
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.at_domain_declaration() {
            return Err(self.error(&format!(
                "'{}' declarations must be at the top level.",
                self.peek().lexeme()
            )));
        }
        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.match_token(&[TokenType::LeftBrace]) {
            return self.block_statements().map(Stmt::Block);
        }
        if self.match_token(&[TokenType::Return]) {
            if self.function_depth == 0 {
                return Err(format!(
                    "[line {}] Error at 'return': Cannot return outside a function.",
                    self.previous().line()
                ));
            }
            let value = if self.check(&TokenType::Semicolon) {
                None
            } else {
                Some(self.expression()?)
            };
            self.consume(&TokenType::Semicolon, "Expect ';' after return.")?;
            return Ok(Stmt::Return(value));
        }
        if self.match_token(&[TokenType::Print]) {
            let expression = self.expression()?;
            self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
            return Ok(Stmt::Print(expression));
        }
        self.expression_statement()
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
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

    /// `for` is syntactic sugar, so the evaluator never sees it. The loop
    ///
    /// ```text
    /// for (init; cond; incr) body
    /// ```
    ///
    /// becomes `{ init; while (cond) { body; incr; } }`. The outer block keeps
    /// a `var` initializer from leaking into the enclosing scope, and a missing
    /// condition means `true`. The loop variable lives in that outer block, so
    /// all iterations share one binding.
    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'.")?;
        let initializer = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.check(&TokenType::Var) || self.check(&TokenType::At) {
            Some(self.declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let condition = if self.check(&TokenType::Semicolon) {
            Expr::Literal(Literal::Boolean(true))
        } else {
            self.expression()?
        };
        self.consume(&TokenType::Semicolon, "Expect ';' after loop condition.")?;
        let increment = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }
        let mut loop_statement = Stmt::While {
            condition,
            body: Box::new(body),
        };
        if let Some(initializer) = initializer {
            loop_statement = Stmt::Block(vec![initializer, loop_statement]);
        }
        Ok(loop_statement)
    }

    pub fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expression = self.logic_or()?;
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

    /// `or` binds looser than `and`, as the words do in English.
    fn logic_or(&mut self) -> Result<Expr, String> {
        let mut expr = self.logic_and()?;
        while self.match_token(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;
        while self.match_token(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
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
        self.call()
    }

    /// `call → primary ( "(" arguments? ")" )*`. Each `(` wraps whatever has
    /// been parsed so far, so `f(1)(2)` calls the result of `f(1)`.
    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        while self.match_token(&[TokenType::LeftParen]) {
            expr = self.finish_call(expr)?;
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self
            .consume(&TokenType::RightParen, "Expect ')' after arguments.")?
            .clone();
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    pub fn primary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[TokenType::Identifier])
            || (self.in_ability
                && self.match_token(&[TokenType::Inspect, TokenType::Isolate, TokenType::Execute]))
        {
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
            TokenType::At
                | TokenType::Var
                | TokenType::Fun
                | TokenType::Print
                | TokenType::Return
                | TokenType::LeftBrace
                | TokenType::If
                | TokenType::While
                | TokenType::For
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
        Self::error_at(self.peek(), message)
    }

    fn error_at(token: &Token, message: &str) -> String {
        let location = if token.token_type() == &TokenType::Eof {
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
            if self.at_statement_keyword() || (!in_block && self.at_domain_declaration()) {
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
