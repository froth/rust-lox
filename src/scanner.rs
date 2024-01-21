use miette::{NamedSource, IntoDiagnostic, Result};
use phf::phf_map;

use crate::{
    error::ScannerError::{*},
    error_reporter::ErrorReporter,
    token::{Token, TokenType},
};
pub struct Scanner<'a> {
    source: String,
    filename: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    error_reporter: &'a mut dyn ErrorReporter,
}

static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => TokenType::And,
    "false" => TokenType::False,
    "fun" => TokenType::Fun,
    "for" => TokenType::For,
    "if" => TokenType::If,
    "else" => TokenType::Else,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
    "class" => TokenType::Class,
};

impl<'a> Scanner<'a> {
    pub fn new(source: String, filename: String, e: &'a mut dyn ErrorReporter) -> Self {
        Self {
            source,
            filename,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            error_reporter: e,
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

    fn scan_token(&mut self, char: char) {
        use TokenType::*;
        let token = match char {
            '(' => Ok(Some(LeftParen)),
            ')' => Ok(Some(RightParen)),
            '{' => Ok(Some(LeftBrace)),
            '}' => Ok(Some(RightBrace)),
            ',' => Ok(Some(Comma)),
            '.' => Ok(Some(Dot)),
            '-' => Ok(Some(Minus)),
            '+' => Ok(Some(Plus)),
            ';' => Ok(Some(Semicolon)),
            '*' => Ok(Some(Star)),
            '!' => Ok(Some(if self.matches('=') { BangEqual } else { Bang })),
            '=' => Ok(Some(if self.matches('=') { EqualEqual } else { Equal })),
            '<' => Ok(Some(if self.matches('=') { LessEqual } else { Less })),
            '>' => Ok(Some(if self.matches('=') {
                GreaterEqual
            } else {
                Greater
            })),
            '/' => {
                if self.matches('/') {
                    self.consume_comment();
                    Ok(None)
                } else {
                    Ok(Some(Slash))
                }
            }
            '\n' => {
                self.line += 1;
                Ok(None)
            }
            ' ' | '\r' | '\t' => Ok(None),
            '"' => self.read_string(),
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_ascii_alphabetic() || c == '_' => Ok(Some(self.read_identifier())),

            _ => Err(Generic(format!("Unexpected character '{}'.", char))).into_diagnostic(), // TODO: miette
        };

        match token {
            Ok(Some(token)) => self.add_token(token),
            Err(e) => self
                .error_reporter
                .error(self.line, format!("{e:?}").as_str()), //TODO: remove format hack
            _ => (),
        }
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

    fn read_string(&mut self) -> Result<Option<TokenType>> {
        let start = self.current;
        loop {
            match self.peek() {
                Some('"') => break,
                Some('\n') => {
                    self.line += 1;
                    self.current += 1;
                }
                Some(_) => self.current += 1,
                None => 
                    Err(NonTerminatedString {
                        src: self.named_source(),
                        location: (start -1, self.current - start).into(),
                    })?
                
            }
        }
        self.current += 1; // the closing ""
        let string = self.source[self.start + 1..self.current - 1].to_string();
        Ok(Some(TokenType::String(string)))
    }

    fn read_number(&mut self) -> Result<Option<TokenType>> {
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
        let result = self.source[self.start..self.current]
            .parse::<f32>()
            .into_diagnostic()// TODO: miette
            .map(|f| Some(TokenType::Number(f)))?;
        Ok(result)
    }

    fn read_identifier(&mut self) -> TokenType {
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.current += 1;
        }

        let text = &self.source[self.start..self.current];
        let token = KEYWORDS.get(text).cloned();
        token.unwrap_or(TokenType::Identifier)
    }
    
    fn named_source(&self) -> NamedSource {
        NamedSource::new(self.filename.clone(), self.source.to_string())
    }
}

#[cfg(test)]
mod scanner_tests {

    use std::string::String;
    use crate::error_reporter::testing::VectorErrorReporter;

    use crate::error_reporter::ErrorReporter;

    use super::Scanner;
    use super::TokenType::*;

    #[test]
    fn parse_string() {
        let input = "\"test\"";
        let mut error_reporter = VectorErrorReporter::new();
        let mut scanner = Scanner::new(input.into(), String::new(), &mut error_reporter);
        let result = scanner.scan_tokens();
        let head = &result[0].token_type;
        assert!(!error_reporter.had_error());
        assert_matches!(head, String(x) if x == "test");
    }
    #[test]
    fn parse_float() {
        let input = "1.1";
        let mut error_reporter = VectorErrorReporter::new();
        let mut scanner = Scanner::new(input.into(), String::new(), &mut error_reporter);
        let result = scanner.scan_tokens();
        assert_eq!(result.len(), 2);
        let head = &result[0].token_type;
        assert!(!error_reporter.had_error());
        assert_matches!(head, Number(_));
    }
    #[test]
    fn parse_identifier() {
        let input = "variable_name";
        let mut error_reporter = VectorErrorReporter::new();
        let mut scanner = Scanner::new(input.into(), String::new(), &mut error_reporter);
        let result = scanner.scan_tokens();
        let head = &result[0];
        let token_type = &head.token_type;
        assert!(!error_reporter.had_error());
        assert_matches!(token_type, Identifier);
        assert_eq!(head.lexeme, input)
    }

    #[test]
    fn parse_for() {
        let input = "for";
        let mut error_reporter = VectorErrorReporter::new();
        let mut scanner = Scanner::new(input.into(), String::new(), &mut error_reporter);
        let result = scanner.scan_tokens();
        let head = &result[0];
        let token_type = &head.token_type;
        assert!(!error_reporter.had_error());
        assert_matches!(token_type, For);
    }

    #[test]
    fn raise_error_on_unterminated_string() {
        let input = "\"";
        let mut error_reporter = VectorErrorReporter::new();
        let mut scanner = Scanner::new(input.into(), String::new(), &mut error_reporter);
        let result = scanner.scan_tokens();
        let head = &result[0];
        let token_type = &head.token_type;
        assert!(error_reporter.had_error());
        // error_reporter.assert_first(Logline::new(1, "", "Static(\"Unterminated string\")")); // TODO: reenable once ErrorReporter has vanished
        assert_matches!(token_type, Eof);
    }
}
