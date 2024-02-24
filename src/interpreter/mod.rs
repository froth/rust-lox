mod environment;
mod expression;
mod literal;
pub mod runtime_error;
mod statement;

use std::rc::Rc;

use crate::{ast::stmt::Stmt, printer::Printer};

use self::{environment::Environment, runtime_error::RuntimeError, statement::StmtInterpreter};

type Result<T> = std::result::Result<T, RuntimeError>;
pub struct Interpreter {
    stmt_interpreter: StmtInterpreter,
}

impl Interpreter {
    pub fn new(printer: Rc<dyn Printer>) -> Self {
        Self {
            stmt_interpreter: StmtInterpreter::new(printer, Environment::new()),
        }
    }

    pub fn interpret(&self, statements: Vec<Stmt>) -> Result<()> {
        statements
            .into_iter()
            .try_for_each(|s| self.stmt_interpreter.interpret(s))
    }
}
