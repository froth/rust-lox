use std::sync::Arc;
use std::vec;

use miette::{NamedSource, SourceSpan};

use crate::ast::stmt::{Stmt, StmtType};
use crate::ast::token::{Token, TokenType};
use crate::source_span_extensions::SourceSpanExtensions;

use super::parser_error::ParserError::*;

use super::macros::{consume, match_token};
use super::{Parser, Result};

struct InternalBlock {
    stmts: Vec<Stmt>,
    location: SourceSpan,
    src: Arc<NamedSource<String>>,
}

impl Parser {
    pub(super) fn stmt(&mut self) -> Result<Stmt> {
        self.declaration()
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
            let semicolon = consume!(self, TokenType::Semicolon, |t: &Token| ExpectedSemicolon {
                expr: None,
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            });
            Ok(Stmt::var(
                name,
                expr,
                var_location.until(semicolon.location),
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
            LeftBrace => self.block_statement(),
            If => self.if_statement(),
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
        let if_location = self.advance().location;

        let left_paren = consume!(self, TokenType::RightBrace, |t: &Token| {
            ExpectedLeftParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });

        todo!()
    }

    fn block(&mut self) -> Result<InternalBlock> {
        let left_brace_location = self.advance().location;
        let mut stmts = vec![];
        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.is_at_end() {
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

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        let semicolon = consume!(self, TokenType::Semicolon, |t: &Token| ExpectedSemicolon {
            expr: Some(expr),
            src: t.src.clone(),
            location: self.previous_if_eof(t.location),
        });
        let location = expr.location.until(semicolon.location);
        Ok(Stmt::expr(expr, location))
    }

    fn print_statement(&mut self) -> Result<Stmt> {
        let print_token_location = self.advance().location;
        let expr = self.expression()?;
        let semicolon = consume!(self, TokenType::Semicolon, |t: &Token| ExpectedSemicolon {
            expr: None,
            src: t.src.clone(),
            location: self.previous_if_eof(t.location),
        });
        let location = print_token_location.until(semicolon.location);
        Ok(Stmt::print(expr, location))
    }
}

#[cfg(test)]
mod tests {

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
        parser.stmt()
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
    fn parse_variable_declaration() {
        let name: String = "name".into();
        let tokens = vec![
            token(TokenType::Var),
            token(TokenType::Identifier(name.clone())),
            token(TokenType::Semicolon),
            token(TokenType::Eof),
        ];
        let stmts = parse_stmt(tokens).unwrap();
        assert_eq!(stmts.to_string().trim_end(), "Var name")
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
        let stmts = parse_stmt(tokens).unwrap();
        assert_eq!(stmts.to_string().trim_end(), "Var name = (nil)")
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
        let stmts = parse_stmt(tokens).unwrap();
        assert_eq!(stmts.to_string().trim_end(), "{\nExpr(nil)\nExpr(nil)\n}")
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
}
