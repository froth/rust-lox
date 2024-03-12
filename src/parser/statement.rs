use std::vec;

use crate::ast::expr::{Expr, ExprType, Literal};
use crate::ast::stmt::{Stmt, StmtType};
use crate::ast::token::{Token, TokenType};
use crate::source_span_extensions::SourceSpanExtensions;

use super::parser_error::ParserError::*;

use super::macros::{check, consume, match_token};
use super::{Parser, Result};

impl Parser {
    pub(super) fn statement(&mut self) -> Result<Stmt> {
        use TokenType::*;
        match self.peek().token_type {
            Print => self.print_statement(),
            LeftBrace => self.block_statement(),
            If => self.if_statement(),
            While => self.while_statement(),
            For => self.for_statement(),
            Return => self.return_statement(),
            _ => self.expression_statement(),
        }
    }

    fn block_statement(&mut self) -> Result<Stmt> {
        let block = self.block()?;
        Ok(Stmt {
            stmt_type: StmtType::Block(block.stmts),
            location: block.location,
            src: block.src,
        })
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        use TokenType::*;
        let if_location = self.advance().location;

        consume!(self, LeftParen, |t: &Token| {
            ExpectedLeftParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        let condition = self.expression()?;

        consume!(self, RightParen, |t: &Token| {
            ExpectedRightParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        let then_statement = self.statement()?;

        let else_statement = match_token!(self, Else)
            .is_some()
            .then(|| self.statement())
            .transpose()?;

        let end_location = else_statement
            .as_ref()
            .map(|s| s.location)
            .unwrap_or(then_statement.location);
        let location = if_location.until(end_location);
        Ok(Stmt::if_stmt(
            condition,
            then_statement,
            else_statement,
            location,
        ))
    }

    fn while_statement(&mut self) -> Result<Stmt> {
        use TokenType::*;
        let while_location = self.advance().location;

        consume!(self, LeftParen, |t: &Token| {
            ExpectedLeftParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        let condition = self.expression()?;

        consume!(self, RightParen, |t: &Token| {
            ExpectedRightParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        let body = self.statement()?;
        let location = while_location.until(body.location);
        Ok(Stmt::while_stmt(condition, body, location))
    }

    // source locations for the parts are weird but should not be needed anyways
    fn for_statement(&mut self) -> Result<Stmt> {
        use TokenType::*;
        let for_location = self.advance().location;

        consume!(self, LeftParen, |t: &Token| {
            ExpectedLeftParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        let initializer = match self.peek().token_type {
            Semicolon => {
                self.advance();
                None
            }
            Var => Some(self.var_declaration()?),
            _ => Some(self.expression_statement()?),
        };

        let condition = if !check!(self, Semicolon) {
            self.expression()?
        } else {
            Expr {
                expr_type: ExprType::literal(Literal::Boolean(true)),
                location: self.peek().location, // technical not correct but best we can do
                src: self.peek().src.clone(),
            }
        };

        consume!(self, Semicolon, |t| self.expected_semicolon(t));

        let increment = if !check!(self, RightParen) {
            Some(self.expression()?)
        } else {
            None
        };

        consume!(self, RightParen, |t: &Token| {
            ExpectedRightParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        let mut body = self.statement()?;

        let location = for_location.until(body.location);
        body = if let Some(increment) = increment {
            let increment_location = increment.location;
            Stmt {
                stmt_type: StmtType::Block(vec![body, Stmt::expr(increment, increment_location)]),
                location,
                src: condition.src.clone(),
            }
        } else {
            body
        };

        let location = for_location.until(body.location);
        let mut while_statement = Stmt::while_stmt(condition, body, location);

        while_statement = if let Some(initializer) = initializer {
            Stmt {
                stmt_type: StmtType::Block(vec![initializer, while_statement]),
                location,
                src: self.peek().src.clone(),
            }
        } else {
            while_statement
        };
        Ok(while_statement)
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        let semicolon = consume!(self, TokenType::Semicolon, |t: &Token| {
            // only recover from expression if it is the top level expression
            let expr = (expr.location.offset() == self.tokens[0].location.offset()).then_some(expr);
            ExpectedSemicolon {
                expr,
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });
        let location = expr.location.until(semicolon.location);
        Ok(Stmt::expr(expr, location))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let print_token_location = self.advance().location;
        let expr = self.expression()?;
        let semicolon = consume!(self, TokenType::Semicolon, |t| self.expected_semicolon(t));
        let location = print_token_location.until(semicolon.location);
        Ok(Stmt::print(expr, location))
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let return_token_location = self.advance().location;
        let expr = (!check!(self, TokenType::Semicolon))
            .then(|| self.expression())
            .transpose()?;

        let semicolon_location =
            consume!(self, TokenType::Semicolon, |t| self.expected_semicolon(t)).location;
        Ok(Stmt {
            stmt_type: StmtType::Return(expr),
            location: return_token_location.until(semicolon_location),
            src: self.src.clone(),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::{
        ast::{
            stmt::Stmt,
            token::{Token, TokenType},
        },
        parser::{parser_error::ParserError, test_helpers::*},
    };

    use super::{Parser, Result};

    fn parse_stmt(tokens: Vec<Token>) -> Result<Stmt> {
        let mut parser = Parser::new(tokens);
        parser.statement()
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
        let stmt = parse_stmt(tokens).unwrap();
        assert_eq!(stmt.to_string().trim_end(), r#"Print("foo")"#);
    }

    #[test]
    fn not_return_eof_location_on_missing_semicolon() {
        let string: String = "foo".into();
        let tokens = vec![
            token(TokenType::Print),
            token_with_location(TokenType::String(string.clone()), (7, 1).into()),
            token_with_location(TokenType::Eof, (8, 1).into()),
        ];
        let err = parse_stmt(tokens).unwrap_err();
        assert_matches!(
            err,
            ParserError::ExpectedSemicolon {
                expr: _,
                src: _,
                location,
            } if location == (7,1).into()
        )
    }

    #[test]
    fn parse_block() {
        let tokens = vec![
            token(TokenType::LeftBrace),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::RightBrace),
            token(TokenType::Eof),
        ];
        let stmt = parse_stmt(tokens).unwrap();
        assert_eq!(stmt.to_string().trim_end(), "{\nExpr(nil)\nExpr(nil)\n}")
    }

    #[test]
    fn parse_non_terminated_block() {
        let tokens = vec![
            token(TokenType::LeftBrace),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let err = parse_stmt(tokens).unwrap_err();
        assert_matches!(
            err,
            ParserError::ExpectedRightBrace {
                src: _,
                location: _,
            }
        )
    }

    #[test]
    fn parse_if_then_else() {
        let tokens = vec![
            token(TokenType::If),
            token(TokenType::LeftParen),
            token(TokenType::Nil),
            token(TokenType::RightParen),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Else),
            token(TokenType::False),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmt = parse_stmt(tokens).unwrap();
        assert_eq!(
            stmt.to_string().trim_end(),
            "if (nil)\nExpr(nil)\nelse\nExpr(false)\nendif"
        )
    }

    #[test]
    fn parse_if_then() {
        let tokens = vec![
            token(TokenType::If),
            token(TokenType::LeftParen),
            token(TokenType::Nil),
            token(TokenType::RightParen),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmt = parse_stmt(tokens).unwrap();
        assert_eq!(stmt.to_string().trim_end(), "if (nil)\nExpr(nil)\nendif")
    }

    #[test]
    fn parse_if_then_missing_left_paren() {
        let tokens = vec![
            token(TokenType::If),
            token(TokenType::Nil),
            token(TokenType::RightParen),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let err = parse_stmt(tokens).unwrap_err();
        assert_matches!(
            err,
            ParserError::ExpectedLeftParen {
                src: _,
                location: _,
            }
        )
    }

    #[test]
    fn parse_if_then_missing_right_paren() {
        let tokens = vec![
            token(TokenType::If),
            token(TokenType::LeftParen),
            token(TokenType::Nil),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let err = parse_stmt(tokens).unwrap_err();
        assert_matches!(
            err,
            ParserError::ExpectedRightParen {
                src: _,
                location: _,
            }
        )
    }

    #[test]
    fn parse_return() {
        let tokens = vec![
            token(TokenType::Return),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmt = parse_stmt(tokens).unwrap();
        assert_eq!(stmt.to_string().trim_end(), "return (nil)")
    }

    #[test]
    fn parse_while() {
        let tokens = vec![
            token(TokenType::While),
            token(TokenType::LeftParen),
            token(TokenType::Nil),
            token(TokenType::RightParen),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmt = parse_stmt(tokens).unwrap();
        assert_eq!(stmt.to_string().trim_end(), "while (nil) {\nExpr(nil)\n}")
    }

    #[test]
    fn parse_and_desugar_for() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::For),
            token(TokenType::LeftParen),
            token(TokenType::Var),
            token(TokenType::Identifier(name.clone())),
            token(TokenType::Equal),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Identifier(name.clone())),
            token(TokenType::EqualEqual),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Identifier(name.clone())),
            token(TokenType::Equal),
            token(TokenType::True),
            token(TokenType::RightParen),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmt = parse_stmt(tokens).unwrap();
        assert_eq!(
            stmt.to_string().trim_end(),
            "{\nVar name = (nil)\nwhile (EqualEqual (variable name) (nil)) {\n{\nExpr(nil)\nExpr(name=(true))\n}\n}\n}"
        )
    }
}
