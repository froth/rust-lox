use core::fmt::Display;

use self::Callable::*;
use super::{
    class::Class, function::Function, native_functions::Native, value::Value, Interpreter, Result,
};
#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Native(Native),
    Function(Function),
    Class(Class),
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        match self {
            Native(native) => native.call(interpreter, arguments),
            Function(function) => function.call(interpreter, arguments),
            Class(class) => Ok(class.call(arguments)),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Native(native) => native.arity(),
            Function(function) => function.arity(),
            Class(class) => class.arity(),
        }
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Native(native) => native.fmt(f),
            Function(function) => function.fmt(f),
            Class(class) => class.fmt(f),
        }
    }
}
