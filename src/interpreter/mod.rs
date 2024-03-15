mod callable;
mod environment;
mod expression;
mod literal;
mod native_functions;
pub mod printer;
pub mod runtime_error;
mod statement;
mod types;
pub mod value;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::{name::NameExpr, stmt::Stmt};

use self::{
    environment::Environment,
    printer::{ConsolePrinter, Printer},
    runtime_error::{RuntimeError, RuntimeErrorOrReturn},
};

type Result<T> = std::result::Result<T, RuntimeError>;
type OrReturnResult<T> = std::result::Result<T, RuntimeErrorOrReturn>;
pub struct Interpreter {
    printer: Box<dyn Printer>,
    environment: Rc<RefCell<Environment>>,
    global: Rc<RefCell<Environment>>,
    locals: HashMap<NameExpr, usize>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Environment::with_native_functions()));
        Self {
            printer: Box::new(ConsolePrinter),
            environment: global.clone(),
            global,
            locals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) -> Result<()> {
        let ret = statements.iter().try_for_each(|s| self.interpret_stmt(s));
        ret.map_err(|err| err.unwrap_runtime_error())
    }

    pub fn add_locals(&mut self, locals: HashMap<NameExpr, usize>) {
        self.locals.extend(locals)
    }
    #[cfg(test)]
    pub fn from_printer(printer: Box<dyn Printer>) -> Self {
        let global = Rc::new(RefCell::new(Environment::new()));
        Self {
            printer,
            environment: global.clone(),
            global,
            locals: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn with_env(printer: Box<dyn Printer>, environment: Environment) -> Self {
        let global = Rc::new(RefCell::new(environment));
        Self {
            printer,
            environment: global.clone(),
            global,
            locals: HashMap::new(),
        }
    }
}
