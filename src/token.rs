use std::fmt::Display;

use miette::SourceSpan;

#[derive(Debug, Clone)]
pub enum TokenType {
    //single character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier,
    String(String),
    Number(f32),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // Eof
    Eof,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // Display in terms of debug. Antipattern?
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub location: SourceSpan,
}

impl Token {
    pub fn new<S: Into<String>>(token_type: TokenType, lexeme: S, location: SourceSpan) -> Self {
        Self {
            token_type,
            lexeme: lexeme.into(),
            location,
        }
    }
}
