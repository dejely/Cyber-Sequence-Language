use crate::token::{ Token, TokenType }; // Refers to the Token struct defined in token.rs

/*
NOTES:

Use `&mut self` in function parameters
for methods that need to change the existing scanner

E.G. function advance changes self.current.
--------------------------------------------
Use `&self` in function parameters
for methods that are only for reading without changing the existing scanner.

E.G. function is_at_end only reads self.current and self.source.len() without changing them.
*/

/*
* Scanner struct and implementation
   source code
       ↓
   scanner.rs
       ↓
   Vec<Token>
*/

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
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

    // HELPERS BELOW
    /* tell the scanner when it has reached the end of the source code
        e.g. source = ['v', 'a', 'r']
        current = 3
        len = 3

        Thus: current >= source.len() => true
    */
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    // reads the next character in the source code and returns it.
    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }

    // safely looks ahead without moving current.
    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    pub fn scan_tokens(mut self) -> Vec<Token> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(
            Token::new(
                TokenType::Eof,
                String::new(), //empty lexeme
                None,
                self.line
            )
        );
        self.tokens // Return a copy of the tokens vector
    }

    fn scan_token(&mut self) {
        let c = self.advance(); // sets the current character to c and moves the current pointer forward by 1

        match c {
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '/' => self.add_token(TokenType::Slash),
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
            _ => {}
        }
    }

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
        let lexeme: String = self.source[self.start..self.current] // from start to finish of token
            .iter()
            .collect(); // build to a string

        self.tokens.push(
            Token::new(
                token_type,
                lexeme,
                None, // No literal value for now
                self.line
            )
        );
    }
}
