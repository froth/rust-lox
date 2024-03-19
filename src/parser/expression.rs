use crate::ast::expr::Expr;
use crate::ast::literal::Literal;
use crate::ast::token::Token;
use crate::ast::{expr::ExprType, token::TokenType};
use crate::parser::macros::{check, consume};
use crate::parser::parser_error::ParserError;
use crate::source_span_extensions::SourceSpanExtensions;

use super::parser_error::ParserError::*;

use super::macros::match_token;
use super::{Parser, Result};

impl Parser {
    pub(super) fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;
        if match_token!(self, TokenType::Equal).is_some() {
            let value = self.assignment()?;
            if let ExprType::Variable(name_expr) = expr.expr_type {
                return Ok(Expr::assign(name_expr, value));
            }
            self.errors.push(InvalidAssignmentTarget {
                src: expr.src.clone(),
                location: expr.location,
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.and()?;
        while let Some(token) = match_token!(self, Or).cloned() {
            let right = self.and()?;
            expr = Expr::logical(expr, token, right);
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.equality()?;
        while let Some(token) = match_token!(self, And).cloned() {
            let right = self.equality()?;
            expr = Expr::logical(expr, token, right);
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
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr> {
        use TokenType::*;
        let mut expr = self.primary()?;
        while match_token!(self, LeftParen).is_some() {
            expr = self.finish_call(expr)?;
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        use TokenType::*;
        let callee_location = callee.location;
        let mut arguments = vec![];
        if !check!(self, RightParen) {
            loop {
                if arguments.len() >= 255 {
                    self.errors.push(ParserError::TooManyArguments {
                        src: self.peek().src.clone(),
                        location: self.peek().location,
                    })
                }
                arguments.push(self.expression()?);
                if match_token!(self, Comma).is_none() {
                    break;
                }
            }
        }

        let right_paran = consume!(self, RightParen, |t: &Token| {
            ExpectedRightParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        Ok(Expr::call(
            callee,
            arguments,
            callee_location.until(right_paran.location),
        ))
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

                let right_paren = consume!(self, TokenType::RightParen, |t: &Token| {
                    ExpectedRightParen {
                        src: t.src.clone(),
                        location: self.previous_if_eof(t.location),
                    }
                });
                let location = token.location.until(right_paren.location);
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
mod test {

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
            r#"(== (!= ("foo") ("foo")) ("foo"))"#
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
        assert_eq!(expr.to_string().trim_end(), r#"(- (1))"#);
    }

    #[test]
    fn and_or_precedence() {
        let tokens = vec![
            token(TokenType::False),
            token(TokenType::Or),
            token(TokenType::False),
            token(TokenType::And),
            token(TokenType::True),
            token(TokenType::Eof),
        ];
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(
            expr.to_string().trim_end(),
            "(Logical or (false) (Logical and (false) (true)))"
        );
    }

    #[test]
    fn and_and() {
        let tokens = vec![
            token(TokenType::True),
            token(TokenType::And),
            token(TokenType::True),
            token(TokenType::And),
            token(TokenType::False),
            token(TokenType::Eof),
        ];
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(
            expr.to_string().trim_end(),
            "(Logical and (Logical and (true) (true)) (false))"
        );
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
            ParserError::ExpectedRightParen {
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
    #[test]
    fn parse_multi_argument_lists_function_call() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::Identifier(name.clone())),
            token(TokenType::LeftParen),
            token(TokenType::True),
            token(TokenType::RightParen),
            token(TokenType::LeftParen),
            token(TokenType::True),
            token(TokenType::RightParen),
            token(TokenType::Eof),
        ];
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(
            expr.to_string().trim_end(),
            "(Call (Call (variable name)=>((true), ))=>((true), ))"
        )
    }

    #[test]
    fn parse_empty_argument_list_function_call() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::Identifier(name.clone())),
            token(TokenType::LeftParen),
            token(TokenType::RightParen),
            token(TokenType::Eof),
        ];
        let expr = parse_expr(tokens).unwrap();
        assert_eq!(expr.to_string().trim_end(), "(Call (variable name)=>())")
    }
}
