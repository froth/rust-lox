#[derive(Debug)]
pub enum TokenType {
    //single character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace, Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens.
    Bang, BangEqual, Equal, EqualEqual, Greater, GreaterEqual, Less, LessEqual,

    // Literals
    Identifier, String(String), Number(f32),

    // Eof
    Eof
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self { Self { token_type, lexeme, line } }
}