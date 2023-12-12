use phf::phf_map;

use crate::{
    error_reporter::ErrorReporter,
    token::{Token, TokenType},
};

pub struct Scanner<'a> {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    error_reporter: &'a mut ErrorReporter,
}

enum ScanResult {
    TokenResult(TokenType),
    Ignore,
    Error(String),
}

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "fun" => TokenType::Fun,
    "for" => TokenType::For,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
};

impl<'a> Scanner<'a> {
    pub fn new(source: String, e: &'a mut ErrorReporter) -> Self {
        Self {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            error_reporter: e,
        }
    }

    fn scan_token(&mut self, char: char) {
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
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_ascii_alphabetic() || c == '_' => self.read_identifier(),

            _ => Error(format!("Unexpected character '{}'.", char)),
        };

        match token {
            TokenResult(token) => self.add_token(token),
            Error(e) => self.error_reporter.error(self.line, e.as_str()),
            Ignore => (),
        }
    }

    pub fn scan_tokens(&mut self) -> Vec<Token> {
        while let Some(char) = self.advance() {
            self.start = self.current - 1; //has already been advanced
            self.scan_token(char)
        }

        self.tokens
            .push(Token::new(TokenType::Eof, String::new(), self.line));
        self.tokens.to_vec()
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
                Some('"') => break,
                Some('\n') => {
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

    fn read_identifier(&mut self) -> ScanResult {
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.current += 1;
        }

        let text = &self.source[self.start..self.current];
        let token = KEYWORDS.get(text).cloned();
        ScanResult::TokenResult(token.unwrap_or(TokenType::Identifier))
    }
}

#[cfg(test)]
mod scanner_tests {

    use crate::error_reporter::ErrorReporter;

    use super::Scanner;
    use super::TokenType::*;

    #[test]
    fn parse_string() {
        let input = "\"test\"";
        let mut error_reporter = ErrorReporter::default();
        let mut scanner = Scanner::new(input.into(), &mut error_reporter);
        let result = scanner.scan_tokens();
        let head = &result[0].token_type;
        assert!(matches!(head, String(x) if x == "test"));
    }
    #[test]
    fn parse_float() {
        let input = "1.1";
        let mut error_reporter = ErrorReporter::default();
        let mut scanner = Scanner::new(input.into(), &mut error_reporter);
        let result = scanner.scan_tokens();
        assert_eq!(result.len(), 2);
        let head = &result[0].token_type;
        assert!(matches!(head, Number(_)));
    }
    #[test]
    fn parse_identifier() {
        let input = "variable_name";
        let mut error_reporter = ErrorReporter::default();
        let mut scanner = Scanner::new(input.into(), &mut error_reporter);
        let result = scanner.scan_tokens();
        let head = &result[0];
        let token_type = &head.token_type;
        assert!(matches!(token_type, Identifier));
        assert_eq!(head.lexeme, input)
    }

    #[test]
    fn parse_for() {
        let input = "for";
        let mut error_reporter = ErrorReporter::default();
        let mut scanner = Scanner::new(input.into(), &mut error_reporter);
        let result = scanner.scan_tokens();
        let head = &result[0];
        let token_type = &head.token_type;
        assert!(matches!(token_type, For));
    }
}
