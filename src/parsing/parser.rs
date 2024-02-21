use std::vec;

use crate::ast::expr::Expr;
use crate::ast::expr::Literal::{self};
use crate::ast::stmt::Stmt;
use crate::source_span_extensions::SourceSpanExtensions;
use crate::token::TokenType;
use crate::{ast::expr::ExprType, token::Token};

use super::parser_error::ParserError::{self, *};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

macro_rules! match_token {
    ($self:ident, $pattern:pat $(if $guard:expr)?) => {
        match $self.peek().token_type {
            $pattern $(if $guard)? => Some($self.advance()),
            _ => None
        }
    };
}
pub type Result<T> = core::result::Result<T, ParserError>;
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = vec![];
        while !self.is_at_end() {
            statements.push(self.statement()?)
        }
        Ok(statements)

        // match self.expression() {
        //     Ok(res) => Ok(res),
        //     Err(err) => {
        //         self.synchronize(); // TODO: does not make sense yet as we can only parse single expressions
        //         Err(err)
        //     }
        // }
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

    fn statement(&mut self) -> Result<Stmt> {
        use TokenType::*;
        match self.peek().token_type {
            Print => self.print_statement(),
            _ => self.expression_statement(),
        }
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        if let Some(semicolon) = match_token!(self, TokenType::Semicolon) {
            let location = expr.location.until(semicolon.location);
            Ok(Stmt::expr(expr, location))
        } else {
            Err(ExpectedSemicolon {
                src: expr.src,
                location: expr.location,
            })
        }
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let print_token_location = self.advance().location;
        let expr = self.expression()?;
        if let Some(semicolon) = match_token!(self, TokenType::Semicolon) {
            let location = print_token_location.until(semicolon.location);
            Ok(Stmt::print(expr, location))
        } else {
            Err(ExpectedSemicolon {
                src: expr.src,
                location: expr.location,
            })
        }
    }

    fn expression(&mut self) -> Result<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.comparision()?;
        while let Some(token) = match_token!(self,  BangEqual | EqualEqual).cloned() {
            let right = self.comparision()?;
            expr = Expr::binary(expr, token, right)
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.term()?;
        while let Some(token) =
            match_token!(self, Greater | GreaterEqual | Less | LessEqual).cloned()
        {
            let right = self.term()?;
            expr = Expr::binary(expr, token, right)
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.factor()?;
        while let Some(token) = match_token!(self, Minus | Plus).cloned() {
            let right = self.factor()?;
            expr = Expr::binary(expr, token, right)
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.unary()?;
        while let Some(token) = match_token!(self, Slash | Star).cloned() {
            let right = self.unary()?;
            expr = Expr::binary(expr, token, right)
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        use TokenType::*;
        if let Some(token) = match_token!(self, Bang | Minus).cloned() {
            Ok(Expr::unary(token, self.unary()?))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        use TokenType::*;
        let token = self.advance().clone();
        let expr = match token.token_type.clone() {
            False => Expr::literal(Literal::Boolean(false), &token),
            True => Expr::literal(Literal::Boolean(true), &token),
            Nil => Expr::literal(Literal::Nil, &token),
            Number(n) => Expr::literal(Literal::Number(n), &token),
            String(s) => Expr::literal(Literal::String(s), &token),
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
                Expr::new(ExprType::grouping(expr), location, token.src)
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
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let stmt = &parser.parse().unwrap()[0];
        assert_eq!(stmt.to_string().trim_end(), r#"Expr("foo")"#);
    }

    #[test]
    fn parse_print_string() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::Print),
            token(TokenType::String(string.clone())),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let stmt = &parser.parse().unwrap()[0];
        assert_eq!(stmt.to_string().trim_end(), r#"Print("foo")"#);
    }

    #[test]
    fn parse_eof() {
        let token = token(TokenType::Eof);
        let tokens = vec![token.clone()];
        let mut parser = Parser::new(tokens);
        let err = parser.parse().unwrap();
        assert!(err.is_empty());
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
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let stmt = &parser.parse().unwrap()[0];
        assert_eq!(
            stmt.to_string().trim_end(),
            r#"Expr(EqualEqual (BangEqual ("foo") ("foo")) ("foo"))"#
        );
    }

    #[test]
    fn parse_minus_1() {
        let tokens = vec![
            token(TokenType::Minus),
            token(TokenType::Number(1.0)),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let stmt = &parser.parse().unwrap()[0];
        assert_eq!(stmt.to_string().trim_end(), r#"Expr(Minus (1))"#);
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
            token_with_location(TokenType::Semicolon, (10, 1).into()),
            token(TokenType::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let stmt = &parser.parse().unwrap()[0];
        assert_eq!(
            stmt.to_string().trim_end(),
            r#"Expr(group (group ("foo")))"#
        );
        assert_eq!(stmt.location, (1, 10).into())
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
