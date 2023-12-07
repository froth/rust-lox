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
    NewLine,
    Error,
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
                    while let Some(x) = self.peek() {
                        if x == '\n' {
                            break;
                        } else {
                            self.advance();
                        }
                    }
                    Ignore
                } else {
                    TokenResult(Slash)
                }
            },
            '\n' => NewLine,
            ' ' | '\r' | '\t' => Ignore,
            _ => Error,
        };

        match token {
            TokenResult(token) => self.add_token(token),
            Error => {
                error_reporter.error(self.line, format!("Unexpected character.{}", char).as_str())
            }
            Ignore => (),
            NewLine => self.line +=1
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
}
