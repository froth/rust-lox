use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::ast::{name::Name, stmt::Stmt};

use super::{
    environment::Environment, runtime_error::RuntimeErrorOrReturn, value::Value, Interpreter,
    Result,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    name: Name,
    parameters: Vec<Name>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl Function {
    pub fn new(
        name: Name,
        parameters: Vec<Name>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            closure,
        }
    }

    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        let mut env = Environment::from_parent(self.closure.clone());
        self.parameters
            .iter()
            .zip(arguments.iter())
            .for_each(|(p, a)| env.define(p, a.clone()));
        let result = interpreter.execute_block(&self.body, env);
        match result {
            Ok(_) => Ok(Value::Nil),
            Err(RuntimeErrorOrReturn::Return(value)) => Ok(value),
            Err(RuntimeErrorOrReturn::RuntimeError(err)) => Err(err),
        }
    }

    pub fn arity(&self) -> usize {
        self.parameters.len()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arity = self.parameters.len();
        let name = &self.name;
        write!(f, "<fun {name} ({arity} parameters)>")
    }
}
