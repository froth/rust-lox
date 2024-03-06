use crate::{
    ast::{
        expr::{Expr, Name},
        stmt::{Stmt, StmtType::*},
    },
    value::Value,
};

use super::{Interpreter, Result};

impl Interpreter {
    pub(super) fn interpret_stmt(&mut self, statement: Stmt) -> Result<()> {
        match statement.stmt_type {
            Expression(expr) => self.interpret_expr(&expr).map(|_| ()),
            Print(expr) => self
                .interpret_expr(&expr)
                .map(|value| self.printer.print(value)),
            Var(key, initializer) => self.define_var(key, initializer),
            Block(stmts) => self.execute_block(stmts),
            If(condition, then_stmt, else_stmt) => {
                self.execute_if(condition, *then_stmt, else_stmt.map(|x| *x))
            }
        }
    }

    fn define_var(&mut self, key: Name, initializer: Option<Expr>) -> Result<()> {
        let initializer = initializer.map_or(Ok(Value::Nil), |expr| self.interpret_expr(&expr))?;
        self.environment.define(key, initializer);
        Ok(())
    }

    fn execute_block(&mut self, stmts: Vec<Stmt>) -> Result<()> {
        self.push_environment();
        let result = stmts.into_iter().try_for_each(|s| self.interpret_stmt(s));
        self.pop_environment();
        result
    }

    fn execute_if(
        &mut self,
        condition: Expr,
        then_stmt: Stmt,
        else_stmt: Option<Stmt>,
    ) -> Result<()> {
        if self.interpret_expr(&condition)?.is_truthy() {
            self.interpret_stmt(then_stmt)?;
        } else if let Some(else_stmt) = else_stmt {
            self.interpret_stmt(else_stmt)?;
        }
        Ok(())
    }
}
#[cfg(test)]
mod stmt_interpreter_tests {

    use miette::NamedSource;

    use crate::{
        ast::{
            expr::{Expr, Literal},
            stmt::{Stmt, StmtType},
            token::{Token, TokenType},
        },
        interpreter::{printer::vec_printer::VecPrinter, runtime_error::RuntimeError, Interpreter},
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

    #[test]
    fn restore_env() {
        let printer = VecPrinter::new();
        let stmt = block(vec![var("a")]);
        let mut interpreter = Interpreter::new(Box::new(printer.clone()));
        interpreter.interpret_stmt(stmt).unwrap();
        let stmt = Stmt::expr(
            Expr::variable("a".to_string(), token(TokenType::Eof)),
            (0, 1).into(),
        );
        let err = interpreter.interpret_stmt(stmt).unwrap_err();
        assert_matches!(err, RuntimeError::UndefinedVariable { .. })
    }

    #[test]
    fn restore_env_on_error() {
        let printer = VecPrinter::new();
        let read_undefined_var = Stmt::expr(
            Expr::variable("b".to_string(), token(TokenType::Eof)),
            (0, 1).into(),
        );
        let stmt = block(vec![var("a"), read_undefined_var]);
        let mut interpreter = Interpreter::new(Box::new(printer.clone()));
        let _ = interpreter.interpret_stmt(stmt).unwrap_err();
        let stmt = Stmt::expr(
            Expr::variable("a".to_string(), token(TokenType::Eof)),
            (0, 1).into(),
        );
        let err = interpreter.interpret_stmt(stmt).unwrap_err();
        assert_matches!(err, RuntimeError::UndefinedVariable { .. })
    }

    fn token(token_type: TokenType) -> Token {
        Token::new(
            token_type,
            "",
            (0, 1).into(),
            NamedSource::new("name", String::new()).into(),
        )
    }

    fn block(stmts: Vec<Stmt>) -> Stmt {
        Stmt {
            stmt_type: StmtType::Block(stmts),
            location: (0, 1).into(),
            src: NamedSource::new("name", String::new()).into(),
        }
    }

    fn literal(literal: Literal) -> Expr {
        Expr::literal(literal, &token(TokenType::Eof))
    }

    fn var(name: &str) -> Stmt {
        Stmt {
            stmt_type: StmtType::Var(name.into(), None),
            location: (0, 1).into(),
            src: NamedSource::new("name", String::new()).into(),
        }
    }
}
