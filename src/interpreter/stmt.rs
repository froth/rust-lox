use crate::{
    ast::stmt::{Stmt, StmtType},
    printer::Printer,
};

use super::expr::ExprInterpreter;
use super::Result;

pub struct StmtInterpreter<'a> {
    printer: &'a dyn Printer,
    expr_interpreter: ExprInterpreter,
}
impl<'a> StmtInterpreter<'a> {
    pub fn new(printer: &'a dyn Printer) -> Self {
        Self {
            printer,
            expr_interpreter: ExprInterpreter::new(),
        }
    }

    pub fn interpret(&self, statement: Stmt) -> Result<()> {
        match statement.stmt_type {
            StmtType::Expression(expr) => self.expr_interpreter.interpret(&expr).map(|_| ()),
            StmtType::Print(expr) => self
                .expr_interpreter
                .interpret(&expr)
                .map(|value| self.printer.print(value)),
        }
    }
}
#[cfg(test)]
mod stmt_interpreter_tests {

    use miette::NamedSource;

    use crate::{
        ast::{
            expr::{Expr, Literal},
            stmt::Stmt,
        },
        interpreter::stmt::StmtInterpreter,
        printer::vec_printer::VecPrinter,
        token::{Token, TokenType},
    };

    #[test]
    fn print_string_literal() {
        let printer = VecPrinter::new();
        let stmt = Stmt::print(
            literal(Literal::String("string".to_string())),
            (0, 1).into(),
        );
        let interpreter = StmtInterpreter::new(&printer);
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
