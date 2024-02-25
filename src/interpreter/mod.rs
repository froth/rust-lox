mod environment;
mod expression;
mod literal;
pub mod runtime_error;
mod statement;

use crate::{ast::stmt::Stmt, printer::Printer};

use self::{environment::Environment, runtime_error::RuntimeError};

type Result<T> = std::result::Result<T, RuntimeError>;
pub struct Interpreter {
    printer: Box<dyn Printer>,
    environment: Environment,
}

impl Interpreter {
    pub fn new(printer: Box<dyn Printer>) -> Self {
        Self {
            printer,
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        statements
            .into_iter()
            .try_for_each(|s| self.interpret_stmt(s))
    }
}
