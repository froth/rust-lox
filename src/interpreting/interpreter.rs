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

impl Interpreter for ExprWithContext {
    fn interpret(&self) -> Result<Value> {
        match &self.expr {
            Expr::Binary(left, token, right) => interpret_binary(left, token, right),
            Expr::Grouping(expr) => expr.interpret(),
            Expr::Literal(l) => l.interpret(),
            Expr::Unary(token, expr) => interpret_unary(token, expr),
        }
    }
}

fn is_truthy(value: Value) -> bool {
    match value {
        Value::Boolean(bool) => bool,
        Value::Nil => false,
        _ => true,
    }
}

fn check_number_operands(
    left: &ExprWithContext,
    token: &Token,
    right: &ExprWithContext,
) -> Result<(f32, f32)> {
    let left_value = left.interpret();
    let right_value = right.interpret();
    match (left_value?, right_value?) {
        (Value::Number(l), Value::Number(r)) => Ok((l, r)),
        (Value::Number(_), value) => Err(SingleTypeMissmatch {
            operator: token.lexeme.clone(),
            expected: Type::Number,
            actual: value.into(),
            src: token.src.clone(),
            operator_location: token.location,
            operand_location: right.location,
        }),
        (value, Value::Number(_)) => Err(SingleTypeMissmatch {
            operator: token.lexeme.clone(),
            expected: Type::Number,
            actual: value.into(),
            src: token.src.clone(),
            operator_location: token.location,
            operand_location: left.location,
        }),
        (lhs, rhs) => Err(DoubleTypeMissmatch {
            operator: token.lexeme.clone(),
            expected: Type::Number,
            actual_lhs: lhs.into(),
            actual_rhs: rhs.into(),
            src: token.src.clone(),
            operator_location: token.location,
            lhs: left.location,
            rhs: right.location,
        }),
    }
}

fn interpret_binary(
    left: &ExprWithContext,
    token: &Token,
    right: &ExprWithContext,
) -> Result<Value> {
    match token.token_type {
        TokenType::Minus => {
            let (l, r) = check_number_operands(left, token, right)?;
            Ok(Value::Number(l - r))
        }
        TokenType::Slash => {
            let (l, r) = check_number_operands(left, token, right)?;
            Ok(Value::Number(l / r))
        },
        TokenType::Star => {
            let (l, r) = check_number_operands(left, token, right)?;
            Ok(Value::Number(l * r))
        },
        TokenType::Plus => todo!(),
        TokenType::Greater => {
            let (l, r) = check_number_operands(left, token, right)?;
            Ok(Value::Boolean(l > r))
        },
        TokenType::GreaterEqual => {
            let (l, r) = check_number_operands(left, token, right)?;
            Ok(Value::Boolean(l >= r))
        },
        TokenType::Less => {
            let (l, r) = check_number_operands(left, token, right)?;
            Ok(Value::Boolean(l < r))
        },
        TokenType::LessEqual => {
            let (l, r) = check_number_operands(left, token, right)?;
            Ok(Value::Boolean(l <= r))
        },
        TokenType::BangEqual => {
            let l = left.interpret();
            let r = right.interpret();
            Ok(Value::Boolean(l? != r?))
        },
        TokenType::EqualEqual => {
            let l = left.interpret();
            let r = right.interpret();
            Ok(Value::Boolean(l? == r?))
        },
        _ => panic!("wrong token type in Expr::Binary, bug in parser"),
    }
}

fn interpret_unary(token: &Token, expr: &ExprWithContext) -> Result<Value> {
    let right = expr.interpret()?;
    match &token.token_type {
        TokenType::Minus => {
            if let Value::Number(num) = right {
                Ok(Value::Number(-num))
            } else {
                Err(SingleTypeMissmatch {
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

    use float_eq::float_eq;
    use miette::NamedSource;

    use crate::{
        expr::{ExprWithContext, Literal},
        interpreting::runtime_error::RuntimeError::*,
        token::{Token, TokenType},
        types::Type,
        value::Value,
    };

    use super::Interpreter;

    #[test]
    fn string_literal() {
        let expr = literal(Literal::String("Test".to_string()));
        assert_matches!(expr.interpret().unwrap(), Value::String(string) if string == "Test");
    }
    #[test]
    fn minus_one() {
        let expr = literal(Literal::Number(1.0));
        let expr = ExprWithContext::unary(token(TokenType::Minus), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Number(number) if number == -1.0);
    }
    #[test]
    fn bang_one() {
        let expr = literal(Literal::Number(1.0));
        let expr = ExprWithContext::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(false));
    }
    #[test]
    fn bang_false() {
        let expr = literal(Literal::Boolean(false));
        let expr = ExprWithContext::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(true));
    }
    #[test]
    fn bang_nil() {
        let expr = literal(Literal::Nil);
        let expr = ExprWithContext::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(true));
    }
    #[test]
    fn five_minus_one() {
        let one = literal(Literal::Number(1.0));
        let five = literal(Literal::Number(5.0));
        let expr = ExprWithContext::binary(five, token(TokenType::Minus), one);
        assert_matches!(expr.interpret().unwrap(), Value::Number(number) if float_eq!(number, 4.0, ulps_all <= 4));
    }
    #[test]
    fn one_minus_string() {
        let left = literal(Literal::Number(1.0));
        let right = literal(Literal::String("sdfsdf".to_string()));
        let operator = token(TokenType::Minus);
        let expr = ExprWithContext::binary(left, operator, right);
        assert_matches!(
            expr.interpret().unwrap_err(),
            SingleTypeMissmatch {
                operator: _,
                expected: Type::Number,
                actual: Type::String,
                ..
            }
        );
    }    #[test]
    fn nil_equals_string() {
        let left = literal(Literal::Nil);
        let right = literal(Literal::String("sdfsdf".to_string()));
        let operator = token(TokenType::EqualEqual);
        let expr = ExprWithContext::binary(left, operator, right);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(false));
    }
    #[test]
    fn string_minus_one() {
        let left = literal(Literal::String("sdfsdf".to_string()));
        let right = literal(Literal::Number(1.0));
        let operator = token(TokenType::Minus);
        let expr = ExprWithContext::binary(left, operator, right);
        assert_matches!(
            expr.interpret().unwrap_err(),
            SingleTypeMissmatch {
                operator: _,
                expected: Type::Number,
                actual: Type::String,
                ..
            }
        );
    }
    #[test]
    fn string_minus_nil() {
        let left = literal(Literal::String("sdfsdf".to_string()));
        let right = literal(Literal::Nil);
        let operator = token(TokenType::Minus);
        let expr = ExprWithContext::binary(left, operator, right);
        assert_matches!(
            expr.interpret().unwrap_err(),
            DoubleTypeMissmatch {
                operator: _,
                expected: Type::Number,
                actual_lhs: Type::String,
                actual_rhs: Type::Nil,
                ..
            }
        );
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
