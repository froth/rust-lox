use core::fmt::Display;

use crate::ast::{expr::Name, stmt::Stmt};

use self::Callable::*;
use super::{
    environment::Environment, runtime_error::RuntimeError, value::Value, Interpreter, Result,
};
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
                let global = interpreter.global.clone();
                let mut env = Environment::from_parent(global);
                parameters
                    .iter()
                    .zip(arguments.iter())
                    .for_each(|(p, a)| env.define(p, a.clone()));
                let result = interpreter.execute_block(body, env);
                match result {
                    Ok(_) => Ok(Value::Nil),
                    Err(RuntimeError::Return { value, .. }) => Ok(value),
                    Err(err) => Err(err),
                }
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
