use std::sync::Arc;
use std::vec;

use miette::{NamedSource, SourceSpan};

use crate::ast::expr::Literal::{self};
use crate::ast::expr::{Expr, NameExpr};
use crate::ast::stmt::Stmt;
use crate::ast::{
    expr::ExprType,
    token::{Token, TokenType},
};
use crate::source_span_extensions::SourceSpanExtensions;

use super::parser_error::ParserError::{self, *};
use super::parser_error::ParserErrors;

pub struct Parser {
    tokens: Vec<Token>,
    errors: Vec<ParserError>,
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
    pub fn parse(
        tokens: Vec<Token>,
        verbose: bool,
    ) -> core::result::Result<Vec<Stmt>, ParserErrors> {
        let stmts = Self::new(tokens).parse_internal()?;
        if verbose {
            eprintln!("Statements:");
            stmts.iter().for_each(|s| eprint!("{}", s));
        }
        Ok(stmts)
    }

    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            errors: vec![],
            current: 0,
        }
    }

    fn parse_internal(&mut self) -> core::result::Result<Vec<Stmt>, ParserErrors> {
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

    fn declaration(&mut self) -> Result<Stmt> {
        use TokenType::*;
        match self.peek().token_type {
            Var => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt> {
        let var_location = self.advance().location;
        let peek = self.peek();
        let src = peek.src.clone();
        if let TokenType::Identifier(name) = &peek.token_type {
            let name = name.clone();
            self.advance();
            let mut expr = None;
            if match_token!(self, TokenType::Equal).is_some() {
                expr = Some(self.expression()?)
            }
            let semicolon_location = self.expect_semicolon(var_location, src.clone())?;
            Ok(Stmt::var(
                name,
                expr,
                var_location.until(semicolon_location),
                src,
            ))
        } else {
            Err(ExpectedIdentifier {
                src: peek.src.clone(),
                location: peek.location,
            })
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
        let semicolon_location = match self.expect_semicolon(expr.location, expr.src.clone()) {
            Ok(location) => location,
            Err(ExpectedSemicolon {
                expr: _,
                src,
                location,
            }) => {
                return Err(ExpectedSemicolon {
                    expr: Some(expr),
                    src,
                    location,
                })
            }
            err => err?,
        };
        let location = expr.location.until(semicolon_location);
        Ok(Stmt::expr(expr, location))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let print_token_location = self.advance().location;
        let expr = self.expression()?;
        let semicolon_location = self.expect_semicolon(expr.location, expr.src.clone())?;
        let location = print_token_location.until(semicolon_location);
        Ok(Stmt::print(expr, location))
    }

    fn expression(&mut self) -> Result<Expr> {
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
            dbg!(value);
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

    fn expect_semicolon(
        &mut self,
        last_location: SourceSpan,
        src: Arc<NamedSource<String>>,
    ) -> Result<SourceSpan> {
        if let Some(semicolon) = match_token!(self, TokenType::Semicolon) {
            let location = semicolon.location;
            Ok(location)
        } else {
            Err(ExpectedSemicolon {
                expr: None,
                src,
                location: last_location,
            })
        }
    }
}

#[cfg(test)]
mod parser_tests {

    use miette::{NamedSource, SourceSpan};

    use crate::{
        ast::token::{Token, TokenType},
        parsing::parser_error::ParserError,
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
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(stmts[0].to_string().trim_end(), r#"Expr("foo")"#);
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
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(stmts[0].to_string().trim_end(), r#"Print("foo")"#);
    }

    #[test]
    fn parse_eof() {
        let token = token(TokenType::Eof);
        let tokens = vec![token.clone()];
        let stmts = Parser::parse(tokens, false).unwrap();
        assert!(stmts.is_empty());
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
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(
            stmts[0].to_string().trim_end(),
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
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(stmts[0].to_string().trim_end(), r#"Expr(Minus (1))"#);
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
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(
            stmts[0].to_string().trim_end(),
            r#"Expr(group (group ("foo")))"#
        );
        assert_eq!(stmts[0].location, (1, 10).into())
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
        let errs = Parser::parse(tokens, false).unwrap_err().parser_errors;
        assert_matches!(
            errs[0],
            ParserError::ExpectedRightParan {
                src: _,
                location: _,
            }
        )
    }

    #[test]
    fn parse_variable_declaration() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::Var),
            token(TokenType::Identifier(name.clone())),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(stmts[0].to_string().trim_end(), "Var name")
    }

    #[test]
    fn parse_variable_assignment() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::Identifier(name.clone())),
            token(TokenType::Equal),
            token(TokenType::True),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(stmts[0].to_string().trim_end(), "Expr(name=(true))")
    }

    #[test]
    fn parse_invalid_variable_assignment() {
        let tokens = vec![
            token(TokenType::True),
            token(TokenType::Equal),
            token(TokenType::True),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let errs = Parser::parse(tokens, false).unwrap_err().parser_errors;
        assert_matches!(
            errs[0],
            ParserError::InvalidAssignmentTarget {
                src: _,
                location: _,
            }
        )
    }

    #[test]
    fn parse_variable_initialisation() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::Var),
            token(TokenType::Identifier(name.clone())),
            token(TokenType::Equal),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmts = Parser::parse(tokens, false).unwrap();
        assert_eq!(stmts[0].to_string().trim_end(), "Var name = (nil)")
    }

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
            ParserError::ExpectedRightParan {
                src: _,
                location: _,
            }
        );
        assert_matches!(
            &errors.parser_errors[1],
            ParserError::ExpectedRightParan {
                src: _,
                location: _,
            }
        )
    }

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
