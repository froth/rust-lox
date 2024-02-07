use crate::expr::ExprWithContext;
use crate::expr::Literal::{self};
use crate::source_span_extensions::SourceSpanExtensions;
use crate::token::TokenType;
use crate::{expr::Expr, token::Token};

use super::parser_error::ParserError::{self, *};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}
pub type Result<T> = core::result::Result<T, ParserError>;
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<ExprWithContext> {
        assert!(self
            .tokens
            .last()
            .is_some_and(|t| matches!(t.token_type, TokenType::Eof)));
        match self.expression() {
            Ok(res) => Ok(res),
            Err(err) => {
                self.synchronize(); // TODO: does not make sense yet as we can only parse single expressions
                Err(err)
            }
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

    fn expression(&mut self) -> Result<ExprWithContext> {
        self.equality()
    }

    fn equality(&mut self) -> Result<ExprWithContext> {
        use TokenType::*;
        let mut expr = self.comparision()?;
        while matches!(self.peek().token_type, BangEqual | EqualEqual) {
            let operator = self.advance().clone();
            let right = self.comparision()?;
            expr = ExprWithContext::binary(expr, operator, right)
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> Result<ExprWithContext> {
        use TokenType::*;
        let mut expr = self.term()?;
        while matches!(
            self.peek().token_type,
            Greater | GreaterEqual | Less | LessEqual
        ) {
            let operator = self.advance().clone();
            let right = self.term()?;
            expr = ExprWithContext::binary(expr, operator, right)
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<ExprWithContext> {
        use TokenType::*;
        let mut expr = self.factor()?;
        while matches!(self.peek().token_type, Minus | Plus) {
            let operator = self.advance().clone();
            let right = self.factor()?;
            expr = ExprWithContext::binary(expr, operator, right)
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<ExprWithContext> {
        use TokenType::*;
        let mut expr = self.unary()?;
        while matches!(self.peek().token_type, Slash | Star) {
            let operator = self.advance().clone();
            let right = self.unary()?;
            expr = ExprWithContext::binary(expr, operator, right)
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<ExprWithContext> {
        use TokenType::*;
        if matches!(self.peek().token_type, Bang | Minus) {
            let operator = self.advance().clone();
            Ok(ExprWithContext::unary(operator, self.unary()?))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<ExprWithContext> {
        use TokenType::*;
        let token = self.advance().clone();
        let expr = match token.token_type.clone() {
            False => ExprWithContext::literal(Literal::Boolean(false), &token),
            True => ExprWithContext::literal(Literal::Boolean(true), &token),
            Nil => ExprWithContext::literal(Literal::Nil, &token),
            Number(n) => ExprWithContext::literal(Literal::Number(n), &token),
            String(s) => ExprWithContext::literal(Literal::String(s), &token),
            LeftParen => {
                let expr = self.expression()?;
                let peek = self.peek().clone();
                if let RightParen = peek.token_type {
                    self.advance();
                } else {
                    Err(ExpectedRightParan {
                        src: peek.src.clone(),
                        location: peek.location,
                    })?
                }
                let location = token.location.until(peek.location);
                ExprWithContext::new(Expr::grouping(expr), location, token.src)
            }
            Eof => Err(UnexpectedEof {
                src: token.src.clone(),
                location: (
                    token.location.offset().saturating_sub(1),
                    token.location.len(),
                )
                    .into(),
            })?,
            _ => Err(ExpectedExpression {
                src: token.src.clone(),
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
        matches!(self.peek().token_type, TokenType::Eof)
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

    use miette::{NamedSource, SourceSpan};

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
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();
        assert_eq!(expr.to_string(), r#"("foo")"#);
    }

    #[test]
    fn parse_eof() {
        let token = token(TokenType::Eof);
        let tokens = vec![token.clone()];
        let mut parser = Parser::new(tokens);
        let err = parser.parse().unwrap_err();
        assert_matches!(err, ParserError::UnexpectedEof {
             src: _,
             location,
         } if location.offset() == token.location.offset() -1)
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
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr.to_string(),
            r#"(EqualEqual (BangEqual ("foo") ("foo")) ("foo"))"#
        );
    }

    #[test]
    fn parse_minus_1() {
        let tokens = vec![
            token(TokenType::Minus),
            token(TokenType::Number(1.0)),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();
        assert_eq!(expr.to_string(), r#"(Minus (1))"#);
    }

    #[test]
    fn parse_grouping() {
        let string: String = "foo".into();
        let tokens = vec![
            token_with_location(TokenType::LeftParen, (1, 1).into()),
            token(TokenType::LeftParen),
            token(TokenType::String(string.clone())),
            token(TokenType::RightParen),
            token_with_location(TokenType::RightParen, (9, 1).into()),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let expr = parser.parse().unwrap();
        assert_eq!(expr.to_string(), r#"(group (group ("foo")))"#);
        assert_eq!(expr.location, (1, 9).into())
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
        let mut parser = Parser::new(tokens);
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
            location: (1, 1).into(),
            src: NamedSource::new("", String::new()).into(),
        }
    }
    fn token_with_location(token_type: TokenType, location: SourceSpan) -> Token {
        Token {
            token_type,
            lexeme: "FAKE_LEXEME".into(),
            location,
            src: NamedSource::new("", String::new()).into(),
        }
    }
}
