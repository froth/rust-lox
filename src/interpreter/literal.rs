use crate::ast::literal::Literal;

use super::{value::Value, Result};
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
