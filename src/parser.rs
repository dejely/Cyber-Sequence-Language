//! Recursive descent expression parser, with recovery between file expressions.

use crate::expr::Expr;
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

    pub fn expression(&mut self) -> Result<Expr, String> {
        self.equality()
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

#[cfg(test)]
mod tests {
    use super::Parser;
    use crate::scanner::Scanner;
    use crate::token::{Token, TokenType};

    fn parser(source: &str) -> Parser {
        Parser::new(Scanner::new(source.to_string()).scan_tokens().unwrap())
    }

    #[test]
    fn prints_lab_expression_examples() {
        let cases = [
            ("42", "42.0"),
            ("3.14", "3.14"),
            ("\"Hello\"", "Hello"),
            ("true", "true"),
            ("false", "false"),
            ("nil", "nil"),
            ("1 + 2", "(+ 1.0 2.0)"),
            ("3 - 4 * 5", "(- 3.0 (* 4.0 5.0))"),
            ("--10", "(- (- 10.0))"),
            (
                "(1 + 2) == (3 - 0)",
                "(== (group (+ 1.0 2.0)) (group (- 3.0 0.0)))",
            ),
            ("(1 + 2) * 3", "(* (group (+ 1.0 2.0)) 3.0)"),
            ("-(4 + 5)", "(- (group (+ 4.0 5.0)))"),
            ("1 + 2 * 3 - 4", "(- (+ 1.0 (* 2.0 3.0)) 4.0)"),
            ("10 + -5 * 15 - 6", "(- (+ 10.0 (* (- 5.0) 15.0)) 6.0)"),
            ("1 <= 2 != 3 >= 4", "(!= (<= 1.0 2.0) (>= 3.0 4.0))"),
            ("5 > 3", "(> 5.0 3.0)"),
            ("!true", "(! true)"),
            ("!!false", "(! (! false))"),
            ("\"hello\" != \"world\"", "(!= hello world)"),
            ("\"CMSC 124 is the best\"", "CMSC 124 is the best"),
        ];
        for (source, expected) in cases {
            assert_eq!(
                parser(source).parse().unwrap().to_string(),
                expected,
                "{source}"
            );
        }
    }

    #[test]
    fn folds_binary_operators_to_the_left() {
        for (source, expected) in [
            ("10 - 3 - 2", "(- (- 10.0 3.0) 2.0)"),
            ("8 / 4 / 2", "(/ (/ 8.0 4.0) 2.0)"),
            ("1 < 2 < 3", "(< (< 1.0 2.0) 3.0)"),
            ("1 == 2 == 3", "(== (== 1.0 2.0) 3.0)"),
        ] {
            assert_eq!(parser(source).parse().unwrap().to_string(), expected);
        }
    }

    #[test]
    fn handles_every_precedence_level_together() {
        assert_eq!(
            parser("1 + 2 * -3 < 4 == !false")
                .parse()
                .unwrap()
                .to_string(),
            "(== (< (+ 1.0 (* 2.0 (- 3.0))) 4.0) (! false))"
        );
    }

    #[test]
    fn rejects_malformed_and_trailing_input() {
        for (source, expected) in [
            (
                "(1 + 2",
                "[line 1] Error at end: Expect ')' after expression.",
            ),
            (
                "this is nonsense",
                "[line 1] Error at 'this': Expect expression.",
            ),
            (
                "// just a comment",
                "[line 1] Error at end: Expect expression.",
            ),
            ("1 +", "[line 1] Error at end: Expect expression."),
            ("()", "[line 1] Error at ')': Expect expression."),
            ("1 2", "[line 1] Error at '2': Expect end of expression."),
            ("1 = 2", "[line 1] Error at '=': Expect end of expression."),
            ("1)", "[line 1] Error at ')': Expect end of expression."),
        ] {
            assert_eq!(parser(source).parse().unwrap_err(), expected, "{source}");
        }
    }

    #[test]
    fn parses_multiline_file_expressions_with_optional_final_semicolon() {
        let expressions = parser("(1 +\n2);\ntrue; nil;").parse_all().unwrap();
        let printed: Vec<_> = expressions.iter().map(ToString::to_string).collect();
        assert_eq!(printed, ["(group (+ 1.0 2.0))", "true", "nil"]);
        assert_eq!(parser("42;").parse().unwrap().to_string(), "42.0");
    }

    #[test]
    fn recovers_at_semicolons_and_reports_errors_in_source_order() {
        assert_eq!(
            parser("1 + ;\n2 * ;\n3 + 4;\n(5 + 6;")
                .parse_all()
                .unwrap_err(),
            "[line 1] Error at ';': Expect expression.\n\
             [line 2] Error at ';': Expect expression.\n\
             [line 4] Error at ';': Expect ')' after expression."
        );
    }

    #[test]
    fn rejects_missing_separators_and_empty_expressions_without_looping() {
        assert_eq!(
            parser("1 2;\n;\n3 +").parse_all().unwrap_err(),
            "[line 1] Error at '2': Expect ';' between expressions.\n\
             [line 2] Error at ';': Expect expression.\n\
             [line 3] Error at end: Expect expression."
        );
    }

    #[test]
    fn handles_empty_token_streams_and_missing_eof() {
        assert_eq!(
            Parser::new(Vec::new()).parse().unwrap_err(),
            "[line 1] Error at end: Expect expression."
        );
        let tokens = vec![Token::new(TokenType::True, "true".to_string(), None, 2)];
        assert_eq!(Parser::new(tokens).parse().unwrap().to_string(), "true");
    }
}
