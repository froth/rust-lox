mod declaration;
mod expression;
mod macros;
pub mod parser_error;
mod statement;

use std::sync::Arc;

use miette::{NamedSource, SourceSpan};

use self::{
    macros::{check, consume},
    parser_error::{
        ParserError::{self, *},
        ParserErrors,
    },
};
use crate::{
    ast::{
        stmt::Stmt,
        token::{Token, TokenType},
    },
    source_span_extensions::SourceSpanExtensions,
};

pub type Result<T> = core::result::Result<T, ParserError>;

pub struct Parser {
    tokens: Vec<Token>,
    errors: Vec<ParserError>,
    current: usize,
    src: Arc<NamedSource<String>>,
}
struct InternalBlock {
    stmts: Vec<Stmt>,
    location: SourceSpan,
    src: Arc<NamedSource<String>>,
}

impl Parser {
    pub fn parse(
        tokens: Vec<Token>,
        verbose: bool,
    ) -> core::result::Result<Vec<Stmt>, ParserErrors> {
        let stmts = Self::new(tokens).do_parse()?;
        if verbose {
            eprintln!("Statements:");
            stmts.iter().for_each(|s| eprint!("{}", s));
        }
        Ok(stmts)
    }

    fn new(tokens: Vec<Token>) -> Self {
        assert!(!tokens.is_empty());
        let src = tokens[0].src.clone();
        Self {
            tokens,
            errors: vec![],
            current: 0,
            src,
        }
    }

    fn do_parse(&mut self) -> core::result::Result<Vec<Stmt>, ParserErrors> {
        let mut statements = vec![];
        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.synchronize();
                    self.errors.push(err)
                }
            }
        }
        if self.errors.is_empty() {
            Ok(statements)
        } else {
            let errors = std::mem::take(&mut self.errors);
            Err(ParserErrors {
                parser_errors: errors,
            })
        }
    }

    fn block(&mut self) -> Result<InternalBlock> {
        let left_brace_location = self.advance().location;
        let mut stmts = vec![];
        while !check!(self, TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        let right_brace = consume!(self, TokenType::RightBrace, |t: &Token| {
            ExpectedRightBrace {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        Ok(InternalBlock {
            stmts,
            location: left_brace_location.until(right_brace.location),
            src: right_brace.src.clone(),
        })
    }

    fn expected_semicolon(&self, t: &Token) -> ParserError {
        ExpectedSemicolon {
            expr: None,
            src: t.src.clone(),
            location: self.previous_if_eof(t.location),
        }
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        self.advance();
        while !self.is_at_end() {
            if matches!(self.previous().token_type, Semicolon) {
                return;
            }

            match self.peek().token_type {
                Class | For | Fun | If | Print | Return | Var | While => return,
                _ => (),
            }
            self.advance();
        }
    }

    fn advance(&mut self) -> &Token {
        let current = self.current;
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[current] // guaranteed by bounds check in advance
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current] // guaranteed by bounds check in advance
    }

    fn previous(&mut self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn previous_if_eof(&self, location: SourceSpan) -> SourceSpan {
        let len = self.tokens.len();
        assert!(len > 1);
        if location == self.tokens[len - 1].location {
            self.tokens[len - 2].location
        } else {
            location
        }
    }
}

#[cfg(test)]
mod test {

    use crate::{
        ast::token::TokenType,
        parser::{parser_error::ParserError, test_helpers::token},
    };

    use super::Parser;

    #[test]
    fn synchronization_works() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::LeftParen),
            token(TokenType::String(string.clone())),
            token(TokenType::Semicolon),
            token(TokenType::LeftParen),
            token(TokenType::String(string.clone())),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let errors = Parser::parse(tokens, false).unwrap_err();
        assert_eq!(errors.parser_errors.len(), 2);
        assert_matches!(
            &errors.parser_errors[0],
            ParserError::ExpectedRightParen {
                src: _,
                location: _,
            }
        );
        assert_matches!(
            &errors.parser_errors[1],
            ParserError::ExpectedRightParen {
                src: _,
                location: _,
            }
        )
    }

    #[test]
    fn parse_eof() {
        let token = token(TokenType::Eof);
        let tokens = vec![token.clone()];
        let stmts = Parser::parse(tokens, false).unwrap();
        assert!(stmts.is_empty());
    }
}

#[cfg(test)]
mod test_helpers {
    use miette::{NamedSource, SourceSpan};

    use crate::ast::token::{Token, TokenType};

    pub(super) fn token(token_type: TokenType) -> Token {
        Token {
            token_type,
            location: (1, 1).into(),
            src: NamedSource::new("", String::new()).into(),
        }
    }
    pub(super) fn token_with_location(token_type: TokenType, location: SourceSpan) -> Token {
        Token {
            token_type,
            location,
            src: NamedSource::new("", String::new()).into(),
        }
    }
}
