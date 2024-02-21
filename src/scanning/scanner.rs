use std::sync::Arc;

use miette::NamedSource;
use phf::phf_map;

use crate::ast::token::{Token, TokenType};

use super::{
    error_combiner::ErrorCombiner,
    scanner_error::{
        ScannerError::{self, *},
        ScannerErrors,
    },
};

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

pub struct Scanner {
    source: String,
    named_source: Arc<NamedSource<String>>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    error_combiner: ErrorCombiner,
}

pub type Result<T> = core::result::Result<T, ScannerError>;

impl Scanner {
    pub fn new(source: String, named_source: Arc<NamedSource<String>>) -> Self {
        let error_combiner = ErrorCombiner::new(named_source.clone());
        Self {
            source,
            named_source,
            tokens: vec![],
            start: 0,
            current: 0,
            error_combiner,
        }
    }

    pub fn scan_tokens(&mut self) -> core::result::Result<Vec<Token>, ScannerErrors> {
        let mut scanner_errors = vec![];
        while let Some(char) = self.advance() {
            self.start = self.current - 1; //has already been advanced
            match self.scan_token(char) {
                Ok(Some(token)) => self.add_token(token),
                Ok(None) => (),
                Err(err) => scanner_errors.push(err),
            }
        }
        self.tokens.push(Token::new(
            TokenType::Eof,
            String::new(),
            (self.current, 0).into(),
            self.named_source.clone(),
        ));
        if scanner_errors.is_empty() {
            Ok(self.tokens.to_vec())
        } else {
            let scanner_errors = self.error_combiner.combine(scanner_errors);
            Err(ScannerErrors { scanner_errors })
        }
    }

    fn scan_token(&mut self, char: char) -> Result<Option<TokenType>> {
        use TokenType::*;
        match char {
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
            ' ' | '\r' | '\t' | '\n' => Ok(None),
            '"' => self.read_string(),
            c if c.is_ascii_digit() => self.read_number(),
            c if c.is_ascii_alphabetic() || c == '_' => Ok(Some(self.read_identifier())),

            _ => Err(UnexpectedCharacter {
                char,
                src: self.named_source.clone(),
                location: (self.current - 1, 1).into(),
            }),
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(
            token_type,
            text,
            (self.start, self.current - self.start).into(),
            self.named_source.clone(),
        ))
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
                Some(_) => self.current += 1,
                None => Err(NonTerminatedString {
                    src: self.named_source.clone(),
                    location: (start - 1, self.current - start + 1).into(),
                })?,
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
        token.unwrap_or(TokenType::Identifier(text.to_string()))
    }
}

#[cfg(test)]
mod scanner_tests {
    use miette::NamedSource;

    use crate::scanning::scanner_error::ScannerError;

    use super::Scanner;
    use crate::ast::token::TokenType::*;

    #[test]
    fn parse_string() {
        let input = "\"test\"".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input).into());
        let result = scanner.scan_tokens().unwrap();
        let head = &result[0].token_type;
        assert_matches!(head, String(x) if x == "test");
    }
    #[test]
    fn parse_float() {
        let input = "1.1".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input).into());
        let result = scanner.scan_tokens().unwrap();
        assert_eq!(result.len(), 2);
        let head = &result[0].token_type;
        assert_matches!(head, Number(_));
    }
    #[test]
    fn parse_identifier() {
        let input = "variable_name".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input.clone()).into());
        let result = scanner.scan_tokens().unwrap();
        let head = &result[0];
        let token_type = &head.token_type;
        assert_matches!(token_type, Identifier(string) if string == &input);
    }

    #[test]
    fn parse_for() {
        let input = "for".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input).into());
        let result = scanner.scan_tokens().unwrap();
        let head = &result[0];
        let token_type = &head.token_type;
        assert_matches!(token_type, For);
    }

    #[test]
    fn raise_error_on_unterminated_string() {
        let input = "1+1; \"12345".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input).into());
        let acc = scanner.scan_tokens().unwrap_err();
        let result = acc.scanner_errors.first().unwrap();
        assert_matches!(result, ScannerError::NonTerminatedString {
             src,
             location,
         } if src.name() == "" && *location == (5,6).into())
    }

    #[test]
    fn raise_error_on_unexpected_char() {
        let input = "^".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input).into());
        let acc = scanner.scan_tokens().unwrap_err();
        let result = acc.scanner_errors.first().unwrap();
        assert_matches!(result, ScannerError::UnexpectedCharacter {
             char: '^',
             src,
             location,
         } if src.name() == "" && *location == (0,1).into())
    }
    #[test]
    fn combine_unexpected_chars() {
        let input = "^^^^".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input).into());
        let acc = scanner.scan_tokens().unwrap_err();
        let result = acc.scanner_errors.first().unwrap();
        assert_matches!(result, ScannerError::UnexpectedCharacters {
             chars,
             src,
             location,
         } if chars == "^^^^" && src.name() == "" && *location == (0,4).into())
    }

    #[test]
    fn combine_unexpected_chars_only_if_offsets_overlap() {
        let input = "^^ @@".to_string();
        let mut scanner = Scanner::new(input.clone(), NamedSource::new("", input).into());
        let acc = scanner.scan_tokens().unwrap_err();
        let result1 = acc.scanner_errors.first().unwrap();
        let result2 = acc.scanner_errors.get(1).unwrap();
        assert_matches!(result1, ScannerError::UnexpectedCharacters {
             chars,
             src,
             location,
         } if chars == "^^" && src.name() == "" && *location == (0,2).into());
        assert_matches!(result2, ScannerError::UnexpectedCharacters {
             chars,
             src,
             location,
         } if chars == "@@" && src.name() == "" && *location == (3,2).into());
    }
}
