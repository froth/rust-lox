use crate::{ast::stmt::{Stmt, StmtType}, printer::Printer};

use super::{runtime_error::RuntimeError, value_interpreter::ValueInterpreter};

pub struct Interpreter<'a>{
    printer: &'a dyn Printer
}

type Result<T> = std::result::Result<T, RuntimeError>;
impl <'a> Interpreter<'a> {
    pub fn new(printer: &'a dyn Printer) -> Self {
        Self { printer }
    }

    pub fn interpret(&self, statements: Vec<Stmt>) -> Result<()> {
        statements.into_iter().try_for_each(|s| self.interpret_single(s))
    }

    fn interpret_single(&self, statement: Stmt) -> Result<()> {
        match statement.stmt_type {
            StmtType::Expression(expr) => expr.interpret().map(|_| ()),
            StmtType::Print(expr) => expr.interpret().map(|value| self.printer.print(value)),
        }
    }
}
#[cfg(test)]
mod interpreter_tests {

    use miette::NamedSource;

    use crate::{ast::{expr::{Expr, Literal}, stmt::Stmt}, interpreting::Interpreter, printer::vec_printer::VecPrinter, token::{Token, TokenType}};

    #[test]
    fn print_string_literal() {
        let printer = VecPrinter::new();
        let stmt = Stmt::print(literal(Literal::String("string".to_string())), (0,1).into());
        let interpreter = Interpreter::new(&printer);
        interpreter.interpret_single(stmt).unwrap();
        assert_eq!(printer.get_lines(), vec!["string".to_string().into()])
    }

    fn token(token_type: TokenType) -> Token {
        Token::new(
            token_type,
            "",
            (0, 1).into(),
            NamedSource::new("name", String::new()).into(),
        )
    }

    fn literal(literal: Literal) -> Expr {
        Expr::literal(literal, &token(TokenType::Eof))
    }
}