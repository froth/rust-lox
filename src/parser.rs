use crate::error_reporter::ErrorReporter;
use crate::expr::Literal::{self, Boolean};
use crate::{expr::Expr, token::Token};

use crate::token::TokenType::*;
pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    error_reporter: &'a mut dyn ErrorReporter,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, error_reporter: &'a mut dyn ErrorReporter) -> Self {
        Self {
            tokens,
            current: 0,
            error_reporter,
        }
    }

    pub fn parse(&mut self) -> Expr {
        self.expression()
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparision();
        while matches!(self.peek().token_type, BangEqual | EqualEqual) {
            let operator = self.advance().clone();
            let right = self.comparision();
            expr = Expr::binary(expr, operator, right);
        }
        expr
    }

    fn comparision(&mut self) -> Expr {
        let mut expr = self.term();
        while matches!(
            self.peek().token_type,
            Greater | GreaterEqual | Less | LessEqual
        ) {
            let operator = self.advance().clone();
            let right = self.term();
            expr = Expr::binary(expr, operator, right);
        }
        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();
        while matches!(self.peek().token_type, Minus | Plus) {
            let operator = self.advance().clone();
            let right = self.factor();
            expr = Expr::binary(expr, operator, right);
        }
        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();
        while matches!(self.peek().token_type, Slash | Star) {
            let operator = self.advance().clone();
            let right = self.unary();
            expr = Expr::binary(expr, operator, right);
        }
        expr
    }

    fn unary(&mut self) -> Expr {
        if matches!(self.peek().token_type, Bang | Minus) {
            let operator = self.advance().clone();
            Expr::unary(operator, self.unary())
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        let expr = match self.peek().token_type.clone() {
            False => Expr::literal(Boolean(false)),
            True => Expr::literal(Boolean(true)),
            Nil => Expr::literal(Literal::Nil),
            Number(n) => Expr::literal(Literal::Number(n)),
            String(s) => Expr::literal(Literal::String(s)),
            LeftParen => todo!(),

            e => {
                eprintln!("{:?}", e);
                todo!();
            }
        };
        self.advance(); // TODO: better place for advance after error handling
        expr
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek().token_type, Eof)
    }

    fn peek(&mut self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&mut self) -> &Token {
        self.tokens.get(self.current - 1).unwrap()
    }
}

#[cfg(test)]
mod parser_tests {

    use crate::{
        error_reporter::{testing::VectorErrorReporter, ErrorReporter},
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
        let mut error_reporter = VectorErrorReporter::new();
        let mut parser = Parser::new(tokens, &mut error_reporter);
        let expr = parser.parse();
        assert!(!error_reporter.had_error());
        assert_eq!(expr.to_string(), r#"("foo")"#);
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
        let mut error_reporter = VectorErrorReporter::new();
        let mut parser = Parser::new(tokens, &mut error_reporter);
        let expr = parser.parse();
        assert!(!error_reporter.had_error());
        assert_eq!(
            expr.to_string(),
            r#"(EqualEqual (BangEqual ("foo") ("foo")) ("foo"))"#
        );
    }

    fn token(token_type: TokenType) -> Token {
        Token {
            token_type,
            lexeme: "FAKE_LEXEME".into(),
            line: 1337,
        }
    }
}
