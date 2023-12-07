use crate::{token::{Token, TokenType}, error_reporter::ErrorReporter};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn scan_token(&mut self, char: char, error_reporter: &mut ErrorReporter) {
        use TokenType::*;
        match char {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            _ => error_reporter.error(self.line, "Unexpected character.")
        };
    }

    pub fn scan_tokens(&mut self, error_reporter: &mut ErrorReporter) -> &Vec<Token> {
        while let Some(char) = self.advance() {
            self.start = self.current -1; //has already been advanced
            self.scan_token(char, error_reporter)
        }

        self.tokens.push(Token::new(TokenType::Eof, String::new(), self.line));
        &self.tokens
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line))
    }

    fn advance(&mut self) -> Option<char> {
        let char = self.source.chars().nth(self.current);
        if char.is_some() {
            self.current += 1;
        }
        char
    }

    fn matches(&mut self, expected: char) -> bool {
        match self.source.chars().nth(self.current) {
            Some(x) if x == expected => {
                self.current += 1;
                true
            },
            _ => false
        }

    }
}