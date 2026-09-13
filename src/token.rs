//! Token categories, source text, parsed literal values, and CLI formatting.
//!
//! A token's `lexeme` preserves the original source spelling. Its `literal`
//! stores a parsed value when available: the source `"hello"` keeps its quotes
//! in the lexeme, but stores `hello` as a string literal without the quotes.

/// Classifies one scanned item independently of its spelling or literal value.
///
/// Declaring a variant does not make the scanner recognize it automatically.
/// In particular, numeric literals are represented here but are not scanned yet.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens.
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// `[`.
    LeftBracket,
    /// `]`.
    RightBracket,
    /// `,`.
    Comma,
    /// `.`.
    Dot,
    /// `-`.
    Minus,
    /// `+`.
    Plus,
    /// `;`.
    Semicolon,
    /// `/`; comment syntax is not implemented yet.
    Slash,
    /// `*`.
    Star,

    // One or two character tokens.
    /// `!`.
    Bang,
    /// `!=`.
    BangEqual,
    /// `=`.
    Equal,
    /// `==`.
    EqualEqual,
    /// `>`.
    Greater,
    /// `>=`.
    GreaterEqual,
    /// `<`.
    Less,
    /// `<=`.
    LessEqual,

    // Literals.
    /// An ASCII name that does not exactly match a reserved keyword.
    Identifier,
    /// Text enclosed in double quotes, with a corresponding string literal.
    String,
    /// Reserved for numeric literals; the scanner does not emit this yet.
    Number,

    // Keywords.
    /// Keyword `and`.
    And,
    /// Keyword `class`.
    Class,
    /// Keyword `else`.
    Else,
    /// Keyword `true`; currently emitted without a boolean literal value.
    True,
    /// Keyword `false`; currently emitted without a boolean literal value.
    False,
    /// Keyword `fun`.
    Fun,
    /// Keyword `for`.
    For,
    /// Keyword `if`.
    If,

    /// Keyword `var`.
    Var,
    /// Keyword `print`.
    Print,

    // End of Input.
    /// Marks successful end of input; emitted once with an empty lexeme.
    Eof,
}

/// One source item together with its category, optional value, and line number.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The category used to distinguish punctuation, names, keywords, and values.
    token_type: TokenType,
    /// Exact source text for this item, including quotes for string tokens.
    lexeme: String,
    /// Parsed value, or `None` for tokens such as identifiers and punctuation.
    literal: Option<Literal>,
    /// One-based source line; multiline strings use their opening quote's line.
    line: usize,
}

/// A parsed value, distinct from the original source spelling in a token.
///
/// The current scanner only constructs the string variant. Numbers, boolean
/// values, and null are available for later implementation stages.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// A floating-point number.
    Number(f64),
    /// String contents without the surrounding source quotes.
    String(String),
    /// A boolean value, distinct from the `True` and `False` keyword categories.
    Boolean(bool),
    /// An explicit null value; `None` on a token means no literal was supplied.
    Null,
}

impl Token {
    /// Constructs a token from values already determined by the scanner.
    ///
    /// This constructor stores its arguments without scanning or validating them.
    /// It also lets other modules create tokens while their fields stay private.
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize
    ) -> Self {
        Self {
            // Field shorthand uses the parameter with the same name.
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

/// Formats tokens in the text layout used by the CLI and `.expected` fixtures.
///
/// Token names use uppercase words separated by underscores. Missing literals
/// and explicit null values both print as `null`. Text is written as stored,
/// without escaping embedded newlines or adding quotes around literal values.
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token_type = match self.token_type {
            TokenType::LeftParen => "LEFT_PAREN",
            TokenType::RightParen => "RIGHT_PAREN",
            TokenType::LeftBrace => "LEFT_BRACE",
            TokenType::RightBrace => "RIGHT_BRACE",
            TokenType::LeftBracket => "LEFT_BRACKET",
            TokenType::RightBracket => "RIGHT_BRACKET",
            TokenType::Comma => "COMMA",
            TokenType::Dot => "DOT",
            TokenType::Minus => "MINUS",
            TokenType::Plus => "PLUS",
            TokenType::Semicolon => "SEMICOLON",
            TokenType::Slash => "SLASH",
            TokenType::Star => "STAR",
            TokenType::Bang => "BANG",
            TokenType::BangEqual => "BANG_EQUAL",
            TokenType::Equal => "EQUAL",
            TokenType::EqualEqual => "EQUAL_EQUAL",
            TokenType::Greater => "GREATER",
            TokenType::GreaterEqual => "GREATER_EQUAL",
            TokenType::Less => "LESS",
            TokenType::LessEqual => "LESS_EQUAL",
            TokenType::Identifier => "IDENTIFIER",
            TokenType::String => "STRING",
            TokenType::Number => "NUMBER",
            TokenType::And => "AND",
            TokenType::Class => "CLASS",
            TokenType::Else => "ELSE",
            TokenType::True => "TRUE",
            TokenType::False => "FALSE",
            TokenType::Fun => "FUN",
            TokenType::For => "FOR",
            TokenType::If => "IF",
            TokenType::Var => "VAR",
            TokenType::Print => "PRINT",
            TokenType::Eof => "EOF",
        };
        let literal = match &self.literal {
            Some(Literal::Number(value)) => value.to_string(),
            Some(Literal::String(value)) => value.clone(),
            Some(Literal::Boolean(value)) => value.to_string(),
            Some(Literal::Null) | None => "null".to_string(),
        };
        write!(
            f,
            "Token(type={}, lexeme={}, literal={}, line={})",
            token_type,
            self.lexeme,
            literal,
            self.line
        )
    }
}
