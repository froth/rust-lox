use super::{value::Value, Result};
use crate::ast::expr::Literal;
pub(super) trait LiteralInterpreter {
    fn interpret(&self) -> Result<Value>;
}
impl LiteralInterpreter for Literal {
    fn interpret(&self) -> Result<Value> {
        Ok(match self {
            Literal::String(s) => Value::String(s.clone()),
            Literal::Number(n) => Value::Number(*n),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Nil => Value::Nil,
        })
    }
}
