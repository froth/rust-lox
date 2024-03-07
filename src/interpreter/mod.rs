mod environment;
mod expression;
mod literal;
pub mod printer;
pub mod runtime_error;
mod statement;

use std::mem;

use crate::ast::stmt::Stmt;

use self::{environment::Environment, printer::Printer, runtime_error::RuntimeError};

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Default)]
pub struct Interpreter {
    printer: Box<dyn Printer>,
    environment: Environment,
}

impl Interpreter {
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        statements.iter().try_for_each(|s| self.interpret_stmt(s))
    }

    fn push_environment(&mut self) {
        let new = Environment::default();
        let old = mem::replace(&mut self.environment, new);
        self.environment.parent = Some(Box::new(old))
    }

    fn pop_environment(&mut self) {
        let parent = self.environment.parent.take();
        let parent = parent.expect("Tried to pop global environment, bug in interpreter");
        self.environment = *parent;
    }

    #[cfg(test)]
    pub fn new(printer: Box<dyn Printer>) -> Self {
        Self {
            printer,
            environment: Environment::default(),
        }
    }
    #[cfg(test)]
    pub fn with_env(printer: Box<dyn Printer>, environment: Environment) -> Self {
        Self {
            printer,
            environment,
        }
    }
}
