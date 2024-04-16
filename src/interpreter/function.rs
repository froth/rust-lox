use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::ast::{name::Name, stmt::Stmt};

use super::{
    class::Instance, environment::Environment, runtime_error::RuntimeErrorOrReturn, value::Value,
    Interpreter, Result,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    name: Name,
    parameters: Vec<Name>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl Function {
    pub fn new(
        name: Name,
        parameters: Vec<Name>,
        body: Vec<Stmt>,
        closure: Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            name,
            parameters,
            body,
            closure,
            is_initializer,
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
            Ok(_) if self.is_initializer => Ok(self
                .closure
                .borrow()
                .get_at(0, &Name::this())
                .unwrap_or(Value::Nil)),
            Ok(_) => Ok(Value::Nil),
            Err(RuntimeErrorOrReturn::Return(value)) => Ok(value),
            Err(RuntimeErrorOrReturn::RuntimeError(err)) => Err(err),
        }
    }

    pub fn arity(&self) -> usize {
        self.parameters.len()
    }

    pub fn bind(self, instance: &Instance) -> Self {
        let mut env = Environment::from_parent(self.closure.clone());
        env.define(&Name::this(), Value::Instance(instance.clone()));
        //TODO: ahhhh, instances are not clonable
        Self {
            name: self.name,
            parameters: self.parameters,
            body: self.body,
            closure: Rc::new(RefCell::new(env)),
            is_initializer: self.is_initializer,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let arity = self.parameters.len();
        let name = &self.name;
        write!(f, "<fun {name} ({arity} parameters)>")
    }
}
