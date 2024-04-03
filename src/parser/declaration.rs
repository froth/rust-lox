use std::vec;

use miette::SourceSpan;

use crate::ast::stmt::{Function, Stmt, StmtType};
use crate::ast::token::{Token, TokenType};
use crate::source_span_extensions::SourceSpanExtensions;

use super::parser_error::ParserError::{self, *};

use super::macros::{check, consume, match_token};
use super::{Parser, Result};

impl Parser {
    pub(super) fn declaration(&mut self) -> Result<Stmt> {
        use TokenType::*;
        match self.peek().token_type {
            Var => self.var_declaration(),
            Fun => self.fun_declaration(),
            Class => self.class_declaration(),
            _ => self.statement(),
        }
    }

    pub(super) fn var_declaration(&mut self) -> Result<Stmt> {
        use TokenType::*;
        let var_location = self.advance().location;
        let peek = self.peek();
        if let Identifier(name) = &peek.token_type {
            let name = name.clone();
            self.advance();
            let mut expr = None;
            if match_token!(self, TokenType::Equal).is_some() {
                expr = Some(self.expression()?)
            }
            let semicolon = consume!(self, Semicolon, |t| self.expected_semicolon(t));
            Ok(Stmt::var(
                name,
                expr,
                var_location.until(semicolon.location),
                self.src.clone(),
            ))
        } else {
            Err(ExpectedIdentifier {
                src: self.src.clone(),
                location: peek.location,
            })
        }
    }

    fn fun_declaration(&mut self) -> Result<Stmt> {
        let (function, location) = self.function()?;
        Ok(Stmt {
            stmt_type: StmtType::Function(function),
            location,
            src: self.src.clone(),
        })
    }

    fn function(&mut self) -> Result<(Function, SourceSpan)> {
        use TokenType::*;
        let fun_location = self.advance().location;
        let identifier = self.peek();
        if let Identifier(name) = &identifier.token_type {
            let name = name.clone();
            self.advance();

            let parameters = self.parameter_list()?;

            let left_brace = self.peek();
            if !matches!(left_brace.token_type, LeftBrace) {
                return Err(ExpectedLeftBrace {
                    src: self.src.clone(),
                    location: left_brace.location,
                });
            }

            let body = self.block()?;

            Ok((
                Function {
                    name: name.into(),
                    parameters: parameters.into_iter().map(|arg| arg.into()).collect(),
                    body: body.stmts,
                },
                fun_location.until(body.location),
            ))
        } else {
            Err(ExpectedIdentifier {
                src: self.src.clone(),
                location: identifier.location,
            })
        }
    }

    fn class_declaration(&mut self) -> Result<Stmt> {
        use TokenType::*;
        let class_location = self.advance().location;
        let identifier = self.peek();
        if let Identifier(name) = &identifier.token_type {
            let name = name.clone();
            self.advance();
            consume!(self, LeftBrace, |t: &Token| {
                ExpectedLeftBrace {
                    src: t.src.clone(),
                    location: self.previous_if_eof(t.location),
                }
            });

            let mut methods = vec![];

            while !check!(self, RightBrace) && !self.is_at_end() {
                methods.push(self.function()?.0)
            }

            let right_brace = consume!(self, RightBrace, |t: &Token| {
                ExpectedRightBrace {
                    src: t.src.clone(),
                    location: self.previous_if_eof(t.location),
                }
            });
            let location = class_location.until(right_brace.location);
            Ok(Stmt::class(name, methods, location, self.src.clone()))
        } else {
            Err(ExpectedIdentifier {
                src: self.src.clone(),
                location: identifier.location,
            })
        }
    }

    fn parameter_list(&mut self) -> Result<Vec<String>> {
        use TokenType::*;
        consume!(self, LeftParen, |t: &Token| {
            ExpectedLeftParen {
                src: self.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });
        let mut parameters = vec![];
        if !check!(self, RightParen) {
            loop {
                if parameters.len() >= 255 {
                    self.errors.push(ParserError::TooManyParameters {
                        src: self.peek().src.clone(),
                        location: self.peek().location,
                    })
                }

                let identifier = self.peek();
                let identifier_location = identifier.location;
                if let Identifier(arg_name) = &identifier.token_type {
                    parameters.push(arg_name.clone());
                    self.advance();
                } else {
                    return Err(ParserError::ExpectedIdentifier {
                        src: identifier.src.clone(),
                        location: identifier_location,
                    });
                }

                if match_token!(self, Comma).is_none() {
                    break;
                }
            }
        }
        consume!(self, RightParen, |t: &Token| {
            ExpectedRightParen {
                src: t.src.clone(),
                location: self.previous_if_eof(t.location),
            }
        });
        Ok(parameters)
    }
}

#[cfg(test)]
mod test {

    use crate::{
        ast::{
            stmt::Stmt,
            token::{Token, TokenType},
        },
        parser::test_helpers::*,
    };

    use super::{Parser, Result};

    fn parse_declaration(tokens: Vec<Token>) -> Result<Stmt> {
        let mut parser = Parser::new(tokens);
        parser.declaration()
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
        let stmts = parse_declaration(tokens).unwrap();
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
        let stmts = parse_declaration(tokens).unwrap();
        assert_eq!(stmts.to_string().trim_end(), "Var name = (nil)")
    }

    #[test]
    fn parse_function_declaration_with_2_arguments() {
        let name: String = "name".into();
        let arg1: String = "a".into();
        let arg2: String = "b".into();

        let tokens = vec![
            token(TokenType::Fun),
            token(TokenType::Identifier(name)),
            token(TokenType::LeftParen),
            token(TokenType::Identifier(arg1)),
            token(TokenType::Comma),
            token(TokenType::Identifier(arg2)),
            token(TokenType::RightParen),
            token(TokenType::LeftBrace),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::RightBrace),
            token(TokenType::Eof),
        ];
        let stmt = parse_declaration(tokens).unwrap();
        assert_eq!(
            stmt.to_string().trim_end(),
            "fun name(a, b, ) {\nExpr(nil)\n}"
        )
    }

    #[test]
    fn parse_class_declaration() {
        let class_name: String = "class_name".into();
        let method_name: String = "method_name".into();

        let tokens = vec![
            token(TokenType::Class),
            token(TokenType::Identifier(class_name)),
            token(TokenType::LeftBrace),
            token(TokenType::Fun),
            token(TokenType::Identifier(method_name)),
            token(TokenType::LeftParen),
            token(TokenType::RightParen),
            token(TokenType::LeftBrace),
            token(TokenType::Nil),
            token(TokenType::Semicolon),
            token(TokenType::RightBrace),
            token(TokenType::RightBrace),
            token(TokenType::Eof),
        ];
        let stmt = parse_declaration(tokens).unwrap();
        assert_eq!(
            stmt.to_string().trim_end(),
            "class class_name{\nfun method_name() {\nExpr(nil)\n}\n}"
        )
    }
}
