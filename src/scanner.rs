//! Converts source text into tokens.

use crate::token::{Literal, Token, TokenType};

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize, // Start of the current token.
    current: usize, // Next character to read.
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Scanner {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // Caller must ensure input remains.
    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    /// Returns all tokens plus EOF, or all scanning errors.
    pub fn scan_tokens(mut self) -> Result<Vec<Token>, String> {
        let mut errors = Vec::new();
        while !self.is_at_end() {
            self.start = self.current;
            if let Err(message) = self.scan_token() {
                errors.push(message);
            }
        }

        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }

        self.tokens.push(
            Token::new(
                TokenType::Eof,
                String::new(), // EOF has no source spelling.
                None,
                self.line
            )
        );
        Ok(self.tokens)
    }

    fn scan_token(&mut self) -> Result<(), String> {
        let c = self.advance();

        match c {
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '@' => self.add_token(TokenType::At),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string()?,
            '0'..='9' => self.number(),
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            _ => return Err(format!("line {}: Unexpected character {:?}.", self.line, c)),
        }
        Ok(())
    }

    // Read the whole name before checking whether it is a keyword.
    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }
        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = match text.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "fun" => TokenType::Fun,
            "for" => TokenType::For,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "var" => TokenType::Var,
            "print" => TokenType::Print,
            "while" => TokenType::While,
            "watch" => TokenType::Watch,
            "ability" => TokenType::Ability,
            "target" => TokenType::Target,
            "source" => TokenType::Source,
            "sequence" => TokenType::Sequence,
            "require" => TokenType::Require,
            "capability" => TokenType::Capability,
            "count" => TokenType::Count,
            "by" => TokenType::By,
            "within" => TokenType::Within,
            "alert" => TokenType::Alert,
            "inspect" => TokenType::Inspect,
            "isolate" => TokenType::Isolate,
            "execute" => TokenType::Execute,
            _ => TokenType::Identifier,
        };
        self.add_token(token_type);
    }

    fn number(&mut self) {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        // A dot belongs to a number only when it starts a fractional part.
        if self.peek() == '.' {
            let next = self.current + 1;
            if next < self.source.len() && self.source[next].is_ascii_digit() {
                self.advance();
                while self.peek().is_ascii_digit() {
                    self.advance();
                }
            }
        }

        let lexeme: String = self.source[self.start..self.current].iter().collect();
        let value = lexeme.parse::<f64>().expect("scanner produced a valid number");
        self.tokens.push(Token::new(
            TokenType::Number,
            lexeme,
            Some(Literal::Number(value)),
            self.line,
        ));
    }

    fn string(&mut self) -> Result<(), String> {
        let opening_line = self.line;
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }
        if self.is_at_end() {
            return Err(format!("line {}: Unterminated string.", opening_line));
        }
        self.advance(); // Consume the closing quote.
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        let value: String = self.source[self.start + 1..self.current - 1].iter().collect();
        self.tokens.push(Token::new(
            TokenType::String,
            lexeme,
            Some(Literal::String(value)),
            opening_line,
        ));
        Ok(())
    }

    // Consume only if the next character matches.
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme: String = self.source[self.start..self.current]
            .iter()
            .collect();

        self.tokens.push(
            Token::new(
                token_type,
                lexeme,
                None,
                self.line
            )
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;

    #[test]
    fn reports_multiple_invalid_characters_in_source_order() {
        let result = Scanner::new("valid\n#\n?\n".to_string()).scan_tokens();
        let error = result.expect_err("invalid characters should reject the scan");

        assert_eq!(
            error,
            "line 2: Unexpected character '#'.\nline 3: Unexpected character '?'."
        );
        assert_eq!(error.matches('\n').count(), 1);
        assert!(!error.ends_with('\n'));
    }
}
