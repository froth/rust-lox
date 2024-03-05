use crate::ast::expr::Literal::{self};
use crate::ast::expr::{Expr, NameExpr};
use crate::ast::{expr::ExprType, token::TokenType};
use crate::source_span_extensions::SourceSpanExtensions;

use super::parser_error::ParserError::*;

use super::macros::match_token;
use super::{Parser, Result};

impl Parser {
    pub(super) fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.equality()?;
        if match_token!(self, TokenType::Equal).is_some() {
            let value = self.assignment()?;
            if let ExprType::Variable(name) = &expr.expr_type {
                let name_expr = NameExpr {
                    name: name.clone(),
                    location: expr.location,
                    src: expr.src,
                };
                return Ok(Expr::assign(name_expr, value));
            }
            self.errors.push(InvalidAssignmentTarget {
                src: expr.src.clone(),
                location: expr.location,
            });
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.comparision()?;
        while let Some(token) = match_token!(self, BangEqual | EqualEqual).cloned() {
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
            Identifier(name) => Expr::variable(name, token),
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
}

#[cfg(test)]
mod tests {

    use crate::{
        ast::{
            expr::Expr,
            token::{Token, TokenType},
        },
        parser::{parser_error::ParserError, test_helpers::*},
    };

    use super::{Parser, Result};

    fn parse_expr(tokens: Vec<Token>) -> Result<Expr> {
        let mut parser = Parser::new(tokens);
        parser.expression()
    }

    #[test]
    fn parse_string_literal() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::String(string.clone())),
            token(TokenType::Eof),
        ];
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(expr.to_string().trim_end(), r#"("foo")"#);
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
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(
            expr.to_string().trim_end(),
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
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(expr.to_string().trim_end(), r#"(Minus (1))"#);
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
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(expr.to_string().trim_end(), r#"(group (group ("foo")))"#);
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
        let err = parse_expr(tokens).unwrap_err();
        assert_matches!(
            err,
            ParserError::ExpectedRightParan {
                src: _,
                location: _,
            }
        )
    }

    #[test]
    fn parse_variable_assignment() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::Identifier(name.clone())),
            token(TokenType::Equal),
            token(TokenType::True),
            token(TokenType::Eof),
        ];
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(expr.to_string().trim_end(), "(name=(true))")
    }

    #[test]
    fn parse_invalid_variable_assignment() {
        let tokens = vec![
            token(TokenType::True),
            token(TokenType::Equal),
            token(TokenType::True),
            token(TokenType::Eof),
        ];

        // parse using Parser::parse as error is reported via side-channel
        let err = Parser::parse(tokens, true).unwrap_err().parser_errors;
        assert_matches!(
            err[0],
            ParserError::InvalidAssignmentTarget {
                src: _,
                location: _,
            }
        )
    }
}
