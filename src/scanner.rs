use crate::{
    error_reporter::ErrorReporter,
    token::{Token, TokenType},
};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

enum ScanResult {
    TokenResult(TokenType),
    Ignore,
    Error(String),
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
        use ScanResult::*;
        use TokenType::*;
        let token = match char {
            '(' => TokenResult(LeftParen),
            ')' => TokenResult(RightParen),
            '{' => TokenResult(LeftBrace),
            '}' => TokenResult(RightBrace),
            ',' => TokenResult(Comma),
            '.' => TokenResult(Dot),
            '-' => TokenResult(Minus),
            '+' => TokenResult(Plus),
            ';' => TokenResult(Semicolon),
            '*' => TokenResult(Star),
            '!' => TokenResult(if self.matches('=') { BangEqual } else { Bang }),
            '=' => TokenResult(if self.matches('=') { EqualEqual } else { Equal }),
            '<' => TokenResult(if self.matches('=') { LessEqual } else { Less }),
            '>' => TokenResult(if self.matches('=') {
                GreaterEqual
            } else {
                Greater
            }),
            '/' => {
                if self.matches('/') {
                    self.consume_comment();
                    Ignore
                } else {
                    TokenResult(Slash)
                }
            }
            '\n' => {
                self.line += 1;
                Ignore
            }
            ' ' | '\r' | '\t' => Ignore,
            '"' => self.read_string(),
            x if x.is_ascii_digit() => self.read_number(),
            _ => Error(format!("Unexpected character '{}'.", char)),
        };

        match token {
            TokenResult(token) => self.add_token(token),
            Error(e) => error_reporter.error(self.line, e.as_str()),
            Ignore => (),
        }
    }

    pub fn scan_tokens(&mut self, error_reporter: &mut ErrorReporter) -> &Vec<Token> {
        while let Some(char) = self.advance() {
            self.start = self.current - 1; //has already been advanced
            self.scan_token(char, error_reporter)
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.line));
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
            }
            _ => false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current)
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current + 1)
    }

    fn consume_comment(&mut self) {
        while let Some(x) = self.peek() {
            if x == '\n' {
                break;
            } else {
                self.advance();
            }
        }
    }

    fn read_string(&mut self) -> ScanResult {
        use ScanResult::*;
        loop {
            match self.peek() {
                Some(c) if c == '"' => break,
                Some(c) if c == '\n' => {
                    self.line += 1;
                    self.current += 1;
                }
                Some(_) => self.current += 1,
                None => {
                    return Error("Unterminated string".into());
                }
            }
        }
        self.current += 1; // the closing ""
        let string = self.source[self.start + 1..self.current - 1].to_string();
        TokenResult(TokenType::String(string))
    }

    fn read_number(&mut self) -> ScanResult {
        while self.peek().is_some_and(|x| x.is_ascii_digit()) {
            self.current += 1;
        }
        if self.peek().is_some_and(|x| x == '.')
            && self.peek_next().is_some_and(|x| x.is_ascii_digit())
        {
            self.current += 1; // the .
            while self.peek().is_some_and(|x| x.is_ascii_digit()) {
                self.current += 1;
            }
        }
        match self.source[self.start..self.current].parse::<f32>() {
            Ok(f) => ScanResult::TokenResult(TokenType::Number(f)),
            Err(e) => ScanResult::Error(e.to_string()),
        }
    }
}
