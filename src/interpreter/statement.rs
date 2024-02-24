use std::rc::Rc;

use crate::{
    ast::stmt::{Stmt, StmtType},
    printer::Printer,
};

use super::Result;
use super::{environment::Environment, expression::ExprInterpreter};

pub struct StmtInterpreter {
    printer: Rc<dyn Printer>,
    environment: Rc<Environment>,
    expr_interpreter: ExprInterpreter,
}
impl StmtInterpreter {
    pub fn new(printer: Rc<dyn Printer>, environment: Environment) -> Self {
        let environment = Rc::new(environment);
        Self {
            printer,
            environment: environment.clone(),
            expr_interpreter: ExprInterpreter::new(environment),
        }
    }

    pub fn interpret(&self, statement: Stmt) -> Result<()> {
        match statement.stmt_type {
            StmtType::Expression(expr) => self.expr_interpreter.interpret(&expr).map(|_| ()),
            StmtType::Print(expr) => self
                .expr_interpreter
                .interpret(&expr)
                .map(|value| self.printer.print(value)),
            StmtType::Var(_, _) => todo!(),
        }
    }
}
#[cfg(test)]
mod stmt_interpreter_tests {

    use std::rc::Rc;

    use miette::NamedSource;

    use crate::{
        ast::{
            expr::{Expr, Literal},
            stmt::Stmt,
            token::{Token, TokenType},
        },
        interpreter::{environment::Environment, statement::StmtInterpreter},
        printer::vec_printer::VecPrinter,
    };

    #[test]
    fn print_string_literal() {
        let printer = Rc::new(VecPrinter::new());
        let stmt = Stmt::print(
            literal(Literal::String("string".to_string())),
            (0, 1).into(),
        );
        let interpreter = StmtInterpreter::new(printer.clone(), Environment::new());
        interpreter.interpret(stmt).unwrap();
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
