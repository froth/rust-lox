use crate::{
    ast::{
        expr::{Expr, Name},
        stmt::{Stmt, StmtType},
    },
    value::Value,
};

use super::{Interpreter, Result};

impl Interpreter {
    pub fn interpret_stmt(&mut self, statement: Stmt) -> Result<()> {
        match statement.stmt_type {
            StmtType::Expression(expr) => self.interpret_expr(&expr).map(|_| ()),
            StmtType::Print(expr) => self
                .interpret_expr(&expr)
                .map(|value| self.printer.print(value)),
            StmtType::Var(key, initializer) => self.define_var(key, initializer),
        }
    }

    fn define_var(&mut self, key: Name, initializer: Option<Expr>) -> Result<()> {
        let initializer = initializer.map_or(Ok(Value::Nil), |expr| self.interpret_expr(&expr))?;
        self.environment.define(key, initializer);
        Ok(())
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
        interpreter::Interpreter,
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
        assert_eq!(printer.get_lines(), vec!["string".into()])
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
