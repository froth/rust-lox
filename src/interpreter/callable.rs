use core::fmt::Display;

use super::{value::Value, Interpreter, Result};
#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Native {
        function: fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>,
        arity: usize,
        name: String,
    },
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        match self {
            Callable::Native { function, .. } => function(interpreter, arguments),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Callable::Native { arity, .. } => *arity,
        }
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Native { name, arity, .. } => {
                write!(f, "<native fun {name} ({arity} arguments)>",)
            }
        }
    }
}
