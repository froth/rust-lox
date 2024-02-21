mod expr;
mod literal;
pub mod runtime_error;
mod stmt;

use crate::{ast::stmt::Stmt, printer::Printer};

use self::{runtime_error::RuntimeError, stmt::StmtInterpreter};

type Result<T> = std::result::Result<T, RuntimeError>;
pub struct Interpreter<'a> {
    stmt_interpreter: StmtInterpreter<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn new(printer: &'a dyn Printer) -> Self {
        Self {
            stmt_interpreter: StmtInterpreter::new(printer),
        }
    }

    pub fn interpret(&self, statements: Vec<Stmt>) -> Result<()> {
        statements
            .into_iter()
            .try_for_each(|s| self.stmt_interpreter.interpret(s))
    }
}
