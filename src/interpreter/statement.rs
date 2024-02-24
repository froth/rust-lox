use crate::ast::stmt::{Stmt, StmtType};

use super::expression::ExprInterpreter;
use super::{Interpreter, Result};

pub trait StmtInterpreter {
    fn interpret_stmt(&mut self, statement: Stmt) -> Result<()>;
}

impl StmtInterpreter for Interpreter {
    fn interpret_stmt(&mut self, statement: Stmt) -> Result<()> {
        match statement.stmt_type {
            StmtType::Expression(expr) => self.interpret_expr(&expr).map(|_| ()),
            StmtType::Print(expr) => self
                .interpret_expr(&expr)
                .map(|value| self.printer.print(value)),
            StmtType::Var(_, _) => todo!(),
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
            token::{Token, TokenType},
        },
        interpreter::{statement::StmtInterpreter, Interpreter},
        printer::vec_printer::VecPrinter,
    };

    #[test]
    fn print_string_literal() {
        let printer = VecPrinter::new();
        let stmt = Stmt::print(
            literal(Literal::String("string".to_string())),
            (0, 1).into(),
        );
        let mut interpreter = Interpreter::new(Box::new(printer.clone()));
        interpreter.interpret_stmt(stmt).unwrap();
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
