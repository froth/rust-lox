use super::{value::Value, Interpreter, Result};
#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Builtin(fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>),
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        match self {
            Callable::Builtin(function) => function(interpreter, arguments),
        }
    }
}
