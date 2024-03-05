use std::sync::Arc;
use std::vec;

use miette::{NamedSource, SourceSpan};

use crate::ast::stmt::{Stmt, StmtType};
use crate::ast::token::TokenType;
use crate::source_span_extensions::SourceSpanExtensions;

use super::parser_error::ParserError::*;

use super::macros::match_token;
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
            LeftBrace => self.block_statement(),
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

    fn block(&mut self) -> Result<InternalBlock> {
        let left_brace_location = self.advance().location;
        let mut stmts = vec![];
        while !matches!(self.peek().token_type, TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        let peek = self.peek();

        if let TokenType::RightBrace = peek.token_type {
            let right_brace = self.advance();
            Ok(InternalBlock {
                stmts,
                location: left_brace_location.until(right_brace.location),
                src: right_brace.src.clone(),
            })
        } else {
            Err(ExpectedRightBrace {
                src: peek.src.clone(),
                location: peek.location,
            })
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
        let stmts = parse_stmt(tokens).unwrap();
        assert_eq!(stmts.to_string().trim_end(), r#"Print("foo")"#);
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
        let errs = parse_stmt(tokens).unwrap_err();
        assert_matches!(
            errs,
            ParserError::ExpectedRightBrace {
                src: _,
                location: _,
            }
        )
    }
}
