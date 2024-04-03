use core::fmt::Display;
use std::{cell::RefCell, rc::Rc};

use self::Callable::*;
use super::{
    class::{Class, Instance},
    function::Function,
    value::Value,
    Interpreter, Result,
};
#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Native {
        function: fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>,
        arity: usize,
        name: String,
    },
    Function(Function),
    Class(Class),
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        match self {
            Native { function, .. } => function(interpreter, arguments),
            Function(function) => function.call(interpreter, arguments),
            Class(class) => Ok(Value::Instance(Rc::new(RefCell::new(Instance::new(
                class.clone(),
            ))))),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Native { arity, .. } => *arity,
            Function(function) => function.arity(),
            Class(_) => 0,
        }
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Native { name, arity, .. } => {
                write!(f, "<native fun {name} ({arity} arguments)>",)
            }
            Function(function) => function.fmt(f),
            Class(class) => class.fmt(f),
        }
    }
}
