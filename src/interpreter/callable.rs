use core::fmt::Display;
use std::{cell::RefCell, rc::Rc};

use crate::ast::{name::Name, stmt::Stmt};

use self::Callable::*;
use super::{
    class::{Class, Instance},
    environment::Environment,
    value::Value,
    Interpreter, Result, RuntimeErrorOrReturn,
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
        closure: Rc<RefCell<Environment>>,
    },
    Class(Class),
}

impl Callable {
    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        match self {
            Native { function, .. } => function(interpreter, arguments),
            Function {
                parameters,
                body,
                closure,
                ..
            } => {
                let mut env = Environment::from_parent(closure.clone());
                parameters
                    .iter()
                    .zip(arguments.iter())
                    .for_each(|(p, a)| env.define(p, a.clone()));
                let result = interpreter.execute_block(body, env);
                match result {
                    Ok(_) => Ok(Value::Nil),
                    Err(RuntimeErrorOrReturn::Return(value)) => Ok(value),
                    Err(RuntimeErrorOrReturn::RuntimeError(err)) => Err(err),
                }
            }
            Class(class) => Ok(Value::Instance(Rc::new(RefCell::new(Instance::new(
                class.clone(),
            ))))),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            Native { arity, .. } => *arity,
            Function {
                parameters: arguments,
                ..
            } => arguments.len(),
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
            Function {
                name,
                parameters: arguments,
                ..
            } => {
                let arity = arguments.len();
                write!(f, "<fun {name} ({arity} arguments)>")
            }
            Class(class) => {
                write!(f, "<class {}>", class.name)
            }
        }
    }
}
