use crate::expr::ExprWithContext;
use crate::token::TokenType;
use crate::types::Type;
use crate::value::Value;
use crate::{
    expr::{Expr, Literal},
    token::Token,
};

use super::runtime_error::RuntimeError;
use super::runtime_error::RuntimeError::*;

type Result<T> = std::result::Result<T, RuntimeError>;
pub trait Interpreter {
    fn interpret(&self) -> Result<Value>;
}

fn is_truthy(value: Value) -> bool {
    match value {
        Value::Boolean(bool) => bool,
        Value::Nil => false,
        _ => true,
    }
}

fn interpret_unary(token: &Token, expr: &ExprWithContext) -> Result<Value> {
    let right = expr.interpret()?;
    match &token.token_type {
        TokenType::Minus => {
            if let Value::Number(num) = right {
                Ok(Value::Number(-num))
            } else {
                Err(UnaryTypeMissmatch {
                    operator: token.lexeme.clone(),
                    expected: Type::Number,
                    actual: right.into(),
                    src: token.src.clone(),
                    operator_location: token.location,
                    operand_location: expr.location,
                })
            }
        }
        TokenType::Bang => Ok(Value::Boolean(!is_truthy(right))),
        t => panic!("Wrong token type:{}, should have been handled by parser", t),
    }
}

impl Interpreter for ExprWithContext {
    fn interpret(&self) -> Result<Value> {
        match &self.expr {
            Expr::Binary(_, _, _) => todo!(),
            Expr::Grouping(expr) => expr.interpret(),
            Expr::Literal(l) => l.interpret(),
            Expr::Unary(token, expr) => interpret_unary(token, expr),
        }
    }
}

impl Interpreter for Literal {
    fn interpret(&self) -> Result<Value> {
        Ok(match self {
            Literal::String(s) => Value::String(s.clone()),
            Literal::Number(n) => Value::Number(*n),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Nil => Value::Nil,
        })
    }
}

#[cfg(test)]
mod interpreter_tests {

    use miette::NamedSource;

    use crate::{
        expr::{ExprWithContext, Literal},
        token::{Token, TokenType},
        value::Value,
    };

    use super::Interpreter;

    #[test]
    fn interpret_string_literal() {
        let expr = literal(Literal::String("Test".to_string()));
        assert_matches!(expr.interpret().unwrap(), Value::String(string) if string == "Test");
    }
    #[test]
    fn interpret_minus_one() {
        let expr = literal(Literal::Number(1.0));
        let expr = ExprWithContext::unary(token(TokenType::Minus), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Number(number) if number < -0.9);
    }
    #[test]
    fn interpret_bang_one() {
        let expr = literal(Literal::Number(1.0));
        let expr = ExprWithContext::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(false));
    }
    #[test]
    fn interpret_bang_false() {
        let expr = literal(Literal::Boolean(false));
        let expr = ExprWithContext::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(true));
    }
    #[test]
    fn interpret_bang_nil() {
        let expr = literal(Literal::Nil);
        let expr = ExprWithContext::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(true));
    }
    fn token(token_type: TokenType) -> Token {
        Token::new(
            token_type,
            "",
            (0, 1).into(),
            NamedSource::new("name", String::new()).into(),
        )
    }

    fn literal(literal: Literal) -> ExprWithContext {
        ExprWithContext::literal(literal, &token(TokenType::Eof))
    }
}
