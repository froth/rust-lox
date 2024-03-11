use core::fmt::Display;

use crate::ast::{expr::Name, stmt::Stmt};

use self::Callable::*;
use super::{value::Value, Interpreter, Result};
#[derive(Debug, Clone, PartialEq)]
pub enum Callable {
    Native {
        function: fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>,
        arity: usize,
        name: String,
    },
    Function {
        name: Name,
        parameters: Vec<Name>,
        body: Vec<Stmt>,
    },
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        match self {
            Native { function, .. } => function(interpreter, arguments),
            Function {
                parameters, body, ..
            } => {
                interpreter.push_environment(); // TODO: this is very bad, it allows access to non global variables...
                parameters
                    .iter()
                    .zip(arguments.iter())
                    .for_each(|(p, a)| interpreter.environment.define(p, a.clone()));
                let result = interpreter.interpret(body);
                interpreter.pop_environment();
                result?;
                Ok(Value::Nil)
            }
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Native { arity, .. } => *arity,
            Function {
                parameters: arguments,
                ..
            } => arguments.len(),
        }
    }
}

impl Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Native { name, arity, .. } => {
                write!(f, "<native fun {name} ({arity} arguments)>",)
            }
            Function {
                name,
                parameters: arguments,
                ..
            } => {
                let arity = arguments.len();
                write!(f, "<fun {name} ({arity} arguments)>",)
            }
        }
    }
}
