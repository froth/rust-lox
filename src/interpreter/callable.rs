use super::{value::Value, Interpreter, Result};
#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Builtin {
        function: fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>,
        arity: usize,
    },
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        match self {
            Callable::Builtin { function, .. } => function(interpreter, arguments),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Callable::Builtin { arity, .. } => *arity,
        }
    }
}
