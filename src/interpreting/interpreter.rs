use crate::token::TokenType;
use crate::types::Type;
use crate::value::Value;
use crate::{
    ast::expr::{ExprType, Literal, Expr},
    token::Token,
};

use super::runtime_error::RuntimeError;
use super::runtime_error::RuntimeError::*;

type Result<T> = std::result::Result<T, RuntimeError>;
pub trait Interpreter {
    fn interpret(&self) -> Result<Value>;
}

impl Interpreter for Expr {
    fn interpret(&self) -> Result<Value> {
        match &self.expr_type {
            ExprType::Binary(left, token, right) => interpret_binary(left, token, right),
            ExprType::Grouping(expr) => expr.interpret(),
            ExprType::Literal(l) => l.interpret(),
            ExprType::Unary(token, expr) => interpret_unary(token, expr),
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

fn handle_numbers(
    left: &Expr,
    token: &Token,
    right: &Expr,
    f: fn(f32, f32) -> Value,
) -> Result<Value> {
    let left_value = left.interpret();
    let right_value = right.interpret();
    match (left_value?, right_value?) {
        (Value::Number(l), Value::Number(r)) => Ok(f(l, r)),
        (Value::Number(_), value) => Err(WrongType {
            operator: token.lexeme.clone(),
            expected: Type::Number,
            actual: value.into(),
            src: token.src.clone(),
            operator_location: token.location,
            operand_location: right.location,
        }),
        (value, Value::Number(_)) => Err(WrongType {
            operator: token.lexeme.clone(),
            expected: Type::Number,
            actual: value.into(),
            src: token.src.clone(),
            operator_location: token.location,
            operand_location: left.location,
        }),
        (lhs, rhs) => Err(WrongTypes {
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

fn handle_values(
    left: &Expr,
    right: &Expr,
    f: fn(Value, Value) -> bool,
) -> Result<Value> {
    let l = left.interpret();
    let r = right.interpret();
    Ok(Value::Boolean(f(l?, r?)))
}

fn handle_plus_binary(
    left: &Expr,
    token: &Token,
    right: &Expr,
) -> Result<Value> {
    let l = left.interpret();
    let r = right.interpret();
    match (l?, r?) {
        (Value::String(l), Value::String(r)) => Ok(Value::String(l + r.as_str())),
        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
        (l, r) => Err(PlusOperatorWrongTypes {
            actual_lhs: l.into(),
            actual_rhs: r.into(),
            src: token.src.clone(),
            operator_location: token.location,
            lhs: left.location,
            rhs: right.location,
        }),
    }
}

fn interpret_binary(
    left: &Expr,
    token: &Token,
    right: &Expr,
) -> Result<Value> {
    match token.token_type {
        TokenType::Minus => handle_numbers(left, token, right, |l, r| (l - r).into()),
        TokenType::Slash => handle_numbers(left, token, right, |l, r| (l / r).into()),
        TokenType::Star => handle_numbers(left, token, right, |l, r| (l * r).into()),
        TokenType::Plus => handle_plus_binary(left, token, right),
        TokenType::Greater => handle_numbers(left, token, right, |l, r| (l > r).into()),
        TokenType::GreaterEqual => handle_numbers(left, token, right, |l, r| (l >= r).into()),
        TokenType::Less => handle_numbers(left, token, right, |l, r| (l < r).into()),
        TokenType::LessEqual => handle_numbers(left, token, right, |l, r| (l <= r).into()),
        TokenType::BangEqual => handle_values(left, right, |l, r| l != r),
        TokenType::EqualEqual => handle_values(left, right, |l, r| l == r),
        _ => panic!("wrong token type in Expr::Binary, bug in parser"),
    }
}

fn interpret_unary(token: &Token, expr: &Expr) -> Result<Value> {
    let right = expr.interpret()?;
    match &token.token_type {
        TokenType::Minus => {
            if let Value::Number(num) = right {
                Ok(Value::Number(-num))
            } else {
                Err(WrongType {
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
        ast::expr::{Expr, Literal},
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
        let expr = literal(1.0.into());
        let expr = Expr::unary(token(TokenType::Minus), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Number(number) if number == -1.0);
    }
    #[test]
    fn bang_one() {
        let expr = literal(1.0.into());
        let expr = Expr::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(false));
    }
    #[test]
    fn bang_false() {
        let expr = literal(Literal::Boolean(false));
        let expr = Expr::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(true));
    }
    #[test]
    fn bang_nil() {
        let expr = literal(Literal::Nil);
        let expr = Expr::unary(token(TokenType::Bang), expr);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(true));
    }
    #[test]
    fn five_minus_one() {
        let one = literal(1.0.into());
        let five = literal(5.0.into());
        let expr = Expr::binary(five, token(TokenType::Minus), one);
        assert_matches!(expr.interpret().unwrap(), Value::Number(number) if float_eq!(number, 4.0, ulps_all <= 4));
    }
    #[test]
    fn one_minus_string() {
        let left = literal(1.0.into());
        let right = literal(Literal::String("sdfsdf".to_string()));
        let operator = token(TokenType::Minus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(
            expr.interpret().unwrap_err(),
            WrongType {
                operator: _,
                expected: Type::Number,
                actual: Type::String,
                ..
            }
        );
    }
    #[test]
    fn nil_equals_string() {
        let left = literal(Literal::Nil);
        let right = literal("sdfsdf".to_string().into());
        let operator = token(TokenType::EqualEqual);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(expr.interpret().unwrap(), Value::Boolean(false));
    }
    #[test]
    fn string_minus_one() {
        let left = literal(Literal::String("sdfsdf".to_string()));
        let right = literal(1.0.into());
        let operator = token(TokenType::Minus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(
            expr.interpret().unwrap_err(),
            WrongType {
                operator: _,
                expected: Type::Number,
                actual: Type::String,
                ..
            }
        );
    }
    #[test]
    fn string_minus_nil() {
        let left = literal("sdfsdf".to_string().into());
        let right = literal(Literal::Nil);
        let operator = token(TokenType::Minus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(
            expr.interpret().unwrap_err(),
            WrongTypes {
                operator: _,
                expected: Type::Number,
                actual_lhs: Type::String,
                actual_rhs: Type::Nil,
                ..
            }
        );
    }
    #[test]
    fn string_plus_string() {
        let left = literal("dogs ".to_string().into());
        let right = literal("are good".to_string().into());
        let operator = token(TokenType::Plus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(expr.interpret().unwrap(), Value::String(string) if  string == "dogs are good");
    }
    #[test]
    fn five_plus() {
        let one = literal(1.0.into());
        let five = literal(5.0.into());
        let expr = Expr::binary(five, token(TokenType::Plus), one);
        assert_matches!(expr.interpret().unwrap(), Value::Number(number) if float_eq!(number, 6.0, ulps_all <= 4));
    }
    #[test]
    fn string_plus_one() {
        let left = literal(Literal::String("sdfsdf".to_string()));
        let right = literal(1.0.into());
        let operator = token(TokenType::Plus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(
            expr.interpret().unwrap_err(),
            PlusOperatorWrongTypes {
                actual_lhs: Type::String,
                actual_rhs: Type::Number,
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

    fn literal(literal: Literal) -> Expr {
        Expr::literal(literal, &token(TokenType::Eof))
    }
}
