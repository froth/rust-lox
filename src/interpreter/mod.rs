mod environment;
mod expression;
mod literal;
pub mod printer;
pub mod runtime_error;
mod statement;

use crate::ast::stmt::Stmt;

use self::{environment::GlobalEnvironment, printer::Printer, runtime_error::RuntimeError};

type Result<T> = std::result::Result<T, RuntimeError>;
#[derive(Default)]
pub struct Interpreter {
    printer: Box<dyn Printer>,
    environment: GlobalEnvironment,
}

impl Interpreter {
    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<()> {
        statements
            .into_iter()
            .try_for_each(|s| self.interpret_stmt(s))
    }
    #[cfg(test)]
    pub fn new(printer: Box<dyn Printer>) -> Self {
        Self {
            printer,
            environment: GlobalEnvironment::default(),
        }
    }
    #[cfg(test)]
    pub fn with_env(printer: Box<dyn Printer>, environment: GlobalEnvironment) -> Self {
        Self {
            printer,
            environment,
        }
    }
}
