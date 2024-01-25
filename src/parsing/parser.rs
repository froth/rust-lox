use std::sync::Arc;

use miette::NamedSource;

use crate::expr::Literal::{self, Boolean};
use crate::{expr::Expr, token::Token};

use crate::token::TokenType::*;

use super::parser_error::ParserError::{self, *};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    named_source: Arc<NamedSource>,
}
pub type Result<T> = core::result::Result<T, ParserError>;
impl Parser {
    pub fn new(tokens: Vec<Token>, named_source: Arc<NamedSource>) -> Self {
        Self {
            tokens,
            current: 0,
            named_source,
        }
    }

    pub fn parse(&mut self) -> Result<Expr> {
        assert!(self
            .tokens
            .last()
            .is_some_and(|t| matches!(t.token_type, Eof)));
        match self.expression() {
            Ok(res) => Ok(res),
            Err(err) => {
                self.synchronize(); // TODO: does not make sense yet as we can only parse single expressions
                Err(err)
            }
        }
    }

    fn synchronize(&mut self) {
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

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparision()?;
        while matches!(self.peek().token_type, BangEqual | EqualEqual) {
            let operator = self.advance().clone();
            let right = self.comparision()?;
            expr = Expr::binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        while matches!(
            self.peek().token_type,
            Greater | GreaterEqual | Less | LessEqual
        ) {
            let operator = self.advance().clone();
            let right = self.term()?;
            expr = Expr::binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;
        while matches!(self.peek().token_type, Minus | Plus) {
            let operator = self.advance().clone();
            let right = self.factor()?;
            expr = Expr::binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;
        while matches!(self.peek().token_type, Slash | Star) {
            let operator = self.advance().clone();
            let right = self.unary()?;
            expr = Expr::binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if matches!(self.peek().token_type, Bang | Minus) {
            let operator = self.advance().clone();
            Ok(Expr::unary(operator, self.unary()?))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        let token = self.advance().clone();
        let expr = match token.token_type.clone() {
            False => Expr::literal(Boolean(false)),
            True => Expr::literal(Boolean(true)),
            Nil => Expr::literal(Literal::Nil),
            Number(n) => Expr::literal(Literal::Number(n)),
            String(s) => Expr::literal(Literal::String(s)),
            LeftParen => {
                let expr = self.expression()?;
                let peek = self.peek();
                if let RightParen = peek.token_type {
                    self.advance();
                } else {
                    Err(ExpectedRightParan {
                        src: self.named_source.clone(),
                        location: peek.location,
                    })?
                }
                Expr::grouping(expr)
            }
            _ => Err(ExpectedExpression {
                src: self.named_source.clone(),
                location: token.location,
            })?,
        };
        Ok(expr)
    }
    fn advance(&mut self) -> &Token {
        let current = self.current;
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[current] // guaranteed by bounds check in advance
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, Eof)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current] // guaranteed by bounds check in advance
    }

    fn previous(&mut self) -> &Token {
        &self.tokens[self.current - 1]
    }
}

#[cfg(test)]
mod parser_tests {

    use miette::NamedSource;

    use crate::{
        parsing::parser_error::ParserError,
        token::{Token, TokenType},
    };

    use super::Parser;

    #[test]
    fn parse_string_literal() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::String(string.clone())),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens, NamedSource::new("", "").into());
        let expr = parser.parse().unwrap();
        assert_eq!(expr.to_string(), r#"("foo")"#);
    }

    #[test]
    fn parse_eof() {
        let token = token(TokenType::Eof);
        let tokens = vec![token.clone()];
        let mut parser = Parser::new(tokens, NamedSource::new("", "").into());
        let err = parser.parse().unwrap_err();
        assert_matches!(err, ParserError::ExpectedExpression {
             src: _,
             location,
         } if location == token.location)
    }

    #[test]
    fn parse_recursive_equal() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::String(string.clone())),
            token(TokenType::BangEqual),
            token(TokenType::String(string.clone())),
            token(TokenType::EqualEqual),
            token(TokenType::String(string.clone())),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens, NamedSource::new("", "").into());
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr.to_string(),
            r#"(EqualEqual (BangEqual ("foo") ("foo")) ("foo"))"#
        );
    }

    #[test]
    fn parse_grouping() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::LeftParen),
            token(TokenType::LeftParen),
            token(TokenType::String(string.clone())),
            token(TokenType::RightParen),
            token(TokenType::RightParen),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens, NamedSource::new("", "").into());
        let expr = parser.parse().unwrap();
        assert_eq!(expr.to_string(), r#"(group (group ("foo")))"#);
    }

    #[test]
    fn parse_grouping_report_unclosed_paren() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::LeftParen),
            token(TokenType::LeftParen),
            token(TokenType::String(string.clone())),
            token(TokenType::RightParen),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens, NamedSource::new("", "").into());
        let err = parser.parse().unwrap_err();
        assert_matches!(
            err,
            ParserError::ExpectedRightParan {
                src: _,
                location: _,
            }
        )
    }

    // TODO: test for synchronize

    fn token(token_type: TokenType) -> Token {
        Token {
            token_type,
            lexeme: "FAKE_LEXEME".into(),
            location: (0, 1).into(),
        }
    }
}
