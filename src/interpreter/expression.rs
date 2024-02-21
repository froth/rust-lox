use crate::{
    ast::expr::{Expr, ExprType},
    ast::token::{Token, TokenType},
    types::Type,
    value::Value,
};

use super::Result;
use super::{literal::LiteralInterpreter, runtime_error::RuntimeError::*};
pub struct ExprInterpreter;

impl ExprInterpreter {
    pub fn new() -> Self {
        ExprInterpreter
    }
    pub fn interpret(&self, expr: &Expr) -> Result<Value> {
        match &expr.expr_type {
            ExprType::Binary(left, token, right) => self.interpret_binary(left, token, right),
            ExprType::Grouping(expr) => self.interpret(expr),
            ExprType::Literal(l) => l.interpret(),
            ExprType::Unary(token, expr) => self.interpret_unary(token, expr),
            ExprType::Variable(name) => todo!(),
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
        &self,
        left: &Expr,
        token: &Token,
        right: &Expr,
        f: fn(f32, f32) -> Value,
    ) -> Result<Value> {
        let left_value = self.interpret(left);
        let right_value = self.interpret(right);
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
        &self,
        left: &Expr,
        right: &Expr,
        f: fn(Value, Value) -> bool,
    ) -> Result<Value> {
        let l = self.interpret(left);
        let r = self.interpret(right);
        Ok(Value::Boolean(f(l?, r?)))
    }

    fn handle_plus_binary(&self, left: &Expr, token: &Token, right: &Expr) -> Result<Value> {
        let l = self.interpret(left);
        let r = self.interpret(right);
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

    fn interpret_binary(&self, left: &Expr, token: &Token, right: &Expr) -> Result<Value> {
        match token.token_type {
            TokenType::Minus => self.handle_numbers(left, token, right, |l, r| (l - r).into()),
            TokenType::Slash => self.handle_numbers(left, token, right, |l, r| (l / r).into()),
            TokenType::Star => self.handle_numbers(left, token, right, |l, r| (l * r).into()),
            TokenType::Plus => self.handle_plus_binary(left, token, right),
            TokenType::Greater => self.handle_numbers(left, token, right, |l, r| (l > r).into()),
            TokenType::GreaterEqual => {
                self.handle_numbers(left, token, right, |l, r| (l >= r).into())
            }
            TokenType::Less => self.handle_numbers(left, token, right, |l, r| (l < r).into()),
            TokenType::LessEqual => self.handle_numbers(left, token, right, |l, r| (l <= r).into()),
            TokenType::BangEqual => self.handle_values(left, right, |l, r| l != r),
            TokenType::EqualEqual => self.handle_values(left, right, |l, r| l == r),
            _ => panic!("wrong token type in Expr::Binary, bug in parser"),
        }
    }

    fn interpret_unary(&self, token: &Token, expr: &Expr) -> Result<Value> {
        let right = self.interpret(expr)?;
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
            TokenType::Bang => Ok(Value::Boolean(!Self::is_truthy(right))),
            t => panic!("Wrong token type:{}, should have been handled by parser", t),
        }
    }
}

#[cfg(test)]
mod value_interpreter_tests {

    use float_eq::float_eq;
    use miette::NamedSource;

    use crate::{
        ast::expr::{Expr, Literal},
        interpreter::{expression::ExprInterpreter, runtime_error::RuntimeError::*},
        ast::token::{Token, TokenType},
        types::Type,
        value::Value,
    };

    #[test]
    fn string_literal() {
        let expr = literal(Literal::String("Test".to_string()));
        assert_matches!(ExprInterpreter::new().interpret(&expr).unwrap(), Value::String(string) if string == "Test");
    }
    #[test]
    fn minus_one() {
        let expr = literal(1.0.into());
        let expr = Expr::unary(token(TokenType::Minus), expr);
        assert_matches!(ExprInterpreter::new().interpret(&expr).unwrap(), Value::Number(number) if number == -1.0);
    }
    #[test]
    fn bang_one() {
        let expr = literal(1.0.into());
        let expr = Expr::unary(token(TokenType::Bang), expr);
        assert_matches!(
            ExprInterpreter::new().interpret(&expr).unwrap(),
            Value::Boolean(false)
        );
    }
    #[test]
    fn bang_false() {
        let expr = literal(Literal::Boolean(false));
        let expr = Expr::unary(token(TokenType::Bang), expr);
        assert_matches!(
            ExprInterpreter::new().interpret(&expr).unwrap(),
            Value::Boolean(true)
        );
    }
    #[test]
    fn bang_nil() {
        let expr = literal(Literal::Nil);
        let expr = Expr::unary(token(TokenType::Bang), expr);
        assert_matches!(
            ExprInterpreter::new().interpret(&expr).unwrap(),
            Value::Boolean(true)
        );
    }
    #[test]
    fn five_minus_one() {
        let one = literal(1.0.into());
        let five = literal(5.0.into());
        let expr = Expr::binary(five, token(TokenType::Minus), one);
        assert_matches!(ExprInterpreter::new().interpret(&expr).unwrap(), Value::Number(number) if float_eq!(number, 4.0, ulps_all <= 4));
    }
    #[test]
    fn one_minus_string() {
        let left = literal(1.0.into());
        let right = literal(Literal::String("sdfsdf".to_string()));
        let operator = token(TokenType::Minus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(
            ExprInterpreter::new().interpret(&expr).unwrap_err(),
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
        assert_matches!(
            ExprInterpreter::new().interpret(&expr).unwrap(),
            Value::Boolean(false)
        );
    }
    #[test]
    fn string_minus_one() {
        let left = literal(Literal::String("sdfsdf".to_string()));
        let right = literal(1.0.into());
        let operator = token(TokenType::Minus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(
            ExprInterpreter::new().interpret(&expr).unwrap_err(),
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
            ExprInterpreter::new().interpret(&expr).unwrap_err(),
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
        assert_matches!(ExprInterpreter::new().interpret(&expr).unwrap(), Value::String(string) if  string == "dogs are good");
    }
    #[test]
    fn five_plus() {
        let one = literal(1.0.into());
        let five = literal(5.0.into());
        let expr = Expr::binary(five, token(TokenType::Plus), one);
        assert_matches!(ExprInterpreter::new().interpret(&expr).unwrap(), Value::Number(number) if float_eq!(number, 6.0, ulps_all <= 4));
    }
    #[test]
    fn string_plus_one() {
        let left = literal(Literal::String("sdfsdf".to_string()));
        let right = literal(1.0.into());
        let operator = token(TokenType::Plus);
        let expr = Expr::binary(left, operator, right);
        assert_matches!(
            ExprInterpreter::new().interpret(&expr).unwrap_err(),
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
