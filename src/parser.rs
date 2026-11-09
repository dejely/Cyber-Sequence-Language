//! Recursive descent parser for expressions and programs, with semicolon recovery.

use crate::expr::Expr;
use crate::stmt::{Annotation, Stmt};
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
        let annotations = self.annotations()?;
        if self.match_token(&[TokenType::Var]) {
            return self.var_declaration(annotations);
        }
        if !annotations.is_empty() {
            return Err(self.error("Annotations must precede a 'var' declaration."));
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

    fn statement(&mut self) -> Result<Stmt, String> {
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
            return self.statement_list(true).map(Stmt::Block);
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
            TokenType::At
                | TokenType::Var
                | TokenType::Print
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;

    fn program(source: &str) -> Vec<Stmt> {
        Parser::new(Scanner::new(source.to_string()).scan_tokens().unwrap())
            .parse_program()
            .unwrap()
    }

    #[test]
    fn annotation_tokens_keep_names_arguments_and_source_lines() {
        let tokens = Scanner::new("// @ignored\n@deprecated(\"old\")\nvar fruit;".into())
            .scan_tokens()
            .unwrap();
        assert_eq!(tokens[0].token_type(), &TokenType::At);
        assert_eq!(tokens[0].line(), 2);
        assert_eq!(tokens[1].lexeme(), "deprecated");
        assert_eq!(tokens[1].token_type(), &TokenType::Identifier);
        assert_eq!(tokens[3].literal(), Some(&Literal::String("old".into())));
        assert_eq!(tokens[5].token_type(), &TokenType::Var);
        assert_eq!(tokens[5].line(), 3);
    }

    #[test]
    fn metadata_attaches_only_to_the_immediately_following_declaration() {
        let statements = program(concat!(
            "@deprecated\n// description\n@doc(\"Use freshFruit\") var oldFruit = \"banana\";\n",
            "var freshFruit = \"mango\";\n",
            "{ @doc(\"Local stock\") var stock = 3; var price; }",
        ));
        let Stmt::Var {
            annotations,
            name,
            initializer,
        } = &statements[0]
        else {
            panic!("expected variable declaration");
        };
        assert_eq!(name.lexeme(), "oldFruit");
        assert_eq!(
            *initializer,
            Some(Expr::Literal(Literal::String("banana".into())))
        );
        assert_eq!(annotations.len(), 2);
        assert_eq!(annotations[0].name.lexeme(), "deprecated");
        assert_eq!(annotations[0].name.line(), 1);
        assert!(annotations[0].arguments.is_empty());
        assert_eq!(annotations[1].name.lexeme(), "doc");
        assert_eq!(annotations[1].name.line(), 3);
        assert_eq!(
            annotations[1].arguments,
            vec![Literal::String("Use freshFruit".into())]
        );
        assert!(matches!(&statements[1], Stmt::Var { annotations, .. } if annotations.is_empty()));
        let Stmt::Block(locals) = &statements[2] else {
            panic!("expected block")
        };
        assert!(matches!(&locals[0], Stmt::Var { annotations, .. } if annotations.len() == 1));
        assert!(matches!(&locals[1], Stmt::Var { annotations, .. } if annotations.is_empty()));
    }

    #[test]
    fn annotation_arguments_and_order_are_preserved() {
        let statements = program(
            "@doc(\"Old stock\") @deprecated(\"Use stock\") var oldStock; @deprecated() var oldPrice;",
        );
        let Stmt::Var { annotations, .. } = &statements[0] else {
            panic!("expected declaration")
        };
        assert_eq!(annotations[0].name.lexeme(), "doc");
        assert_eq!(
            annotations[1].arguments,
            vec![Literal::String("Use stock".into())]
        );
        assert!(
            matches!(&statements[1], Stmt::Var { annotations, .. } if annotations[0].arguments.is_empty())
        );

        let statements = program("for (@doc(\"Sale index\") var i = 0; i < 1; i = i + 1) {}");
        let Stmt::Block(desugared) = &statements[0] else {
            panic!("expected desugared for block")
        };
        let Stmt::Var {
            annotations, name, ..
        } = &desugared[0]
        else {
            panic!("expected loop initializer")
        };
        assert_eq!(name.lexeme(), "i");
        assert_eq!(
            annotations[0].arguments,
            vec![Literal::String("Sale index".into())]
        );
    }

    #[test]
    fn recovery_retains_annotation_starters_and_block_boundaries() {
        let tokens = Scanner::new("{ var x = 1\n@doc() var y; }\n@deprecated print 2;".into())
            .scan_tokens()
            .unwrap();
        let error = Parser::new(tokens).parse_program().unwrap_err();
        assert_eq!(
            error,
            concat!(
                "[line 2] Error at '@': Expect ';' after variable declaration.\n",
                "[line 2] Error at 'doc': Annotation '@doc' requires one string argument.\n",
                "[line 3] Error at 'print': Annotations must precede a 'var' declaration."
            )
        );
    }
}
