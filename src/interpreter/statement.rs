use std::{cell::RefCell, rc::Rc};

use crate::ast::{
    expr::Expr,
    name::Name,
    stmt::{Stmt, StmtType::*},
};

use super::{
    callable::Callable, environment::Environment, runtime_error::RuntimeErrorOrReturn,
    value::Value, Interpreter, OrReturnResult, Result,
};

impl Interpreter {
    pub(super) fn interpret_stmt(&mut self, statement: &Stmt) -> OrReturnResult<()> {
        match &statement.stmt_type {
            Expression(expr) => self.interpret_expr(expr).map(|_| ())?,
            Print(expr) => self
                .interpret_expr(expr)
                .map(|value| self.printer.print(value))?,
            Var {
                name: key,
                initializer,
            } => self.define_var(key, initializer)?,
            Block(stmts) => {
                let local_env = Environment::from_parent(self.environment.clone());
                self.execute_block(stmts, local_env)?
            }
            If {
                condition,
                then_stmt,
                else_stmt,
            } => self.execute_if(condition, then_stmt, else_stmt)?,
            While { condition, body } => self.execute_while(condition, body.as_ref())?,
            Function {
                name,
                parameters,
                body,
            } => self.define_function(name, parameters, body)?,
            Return(expr) => self.execute_return(expr)?,
        };
        Ok(())
    }

    fn define_var(&mut self, key: &Name, initializer: &Option<Expr>) -> Result<()> {
        let initializer = initializer
            .as_ref()
            .map_or(Ok(Value::Nil), |expr| self.interpret_expr(expr))?;
        self.environment.borrow_mut().define(key, initializer);
        Ok(())
    }

    fn define_function(&mut self, name: &Name, parameters: &[Name], body: &[Stmt]) -> Result<()> {
        let function = Callable::Function {
            name: name.clone(),
            parameters: parameters.to_vec(),
            body: body.to_vec(),
            closure: self.environment.clone(),
        };
        self.environment
            .borrow_mut()
            .define(name, Value::Callable(function));
        Ok(())
    }

    pub(super) fn execute_block(
        &mut self,
        stmts: &[Stmt],
        environment: Environment,
    ) -> OrReturnResult<()> {
        let prev = self.environment.clone();
        self.environment = Rc::new(RefCell::new(environment));
        let result = stmts.iter().try_for_each(|s| self.interpret_stmt(s));
        self.environment = prev;
        result
    }

    fn execute_if(
        &mut self,
        condition: &Expr,
        then_stmt: &Stmt,
        else_stmt: &Option<Box<Stmt>>,
    ) -> OrReturnResult<()> {
        if self.interpret_expr(condition)?.is_truthy() {
            self.interpret_stmt(then_stmt)?;
        } else if let Some(else_stmt) = else_stmt {
            self.interpret_stmt(else_stmt.as_ref())?;
        }
        Ok(())
    }

    fn execute_while(&mut self, condition: &Expr, body: &Stmt) -> OrReturnResult<()> {
        while self.interpret_expr(condition)?.is_truthy() {
            self.interpret_stmt(body)?;
        }
        Ok(())
    }

    fn execute_return(&mut self, expr: &Option<Expr>) -> OrReturnResult<()> {
        let value = expr.as_ref().map(|e| self.interpret_expr(e)).transpose()?;
        let value = value.unwrap_or(Value::Nil);
        Err(RuntimeErrorOrReturn::Return(value))
    }
}
#[cfg(test)]
mod stmt_interpreter_tests {

    use std::sync::Arc;

    use miette::NamedSource;

    use crate::{
        ast::{
            expr::Expr,
            literal::Literal,
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
        let mut interpreter = Interpreter::from_printer(Box::new(printer.clone()));
        interpreter.interpret_stmt(&stmt).unwrap();
        assert_eq!(printer.get_lines(), vec!["string".into()])
    }

    #[test]
    fn restore_env() {
        let printer = VecPrinter::new();
        let stmt = block(vec![var("a")]);
        let mut interpreter = Interpreter::from_printer(Box::new(printer.clone()));
        interpreter.interpret_stmt(&stmt).unwrap();
        let stmt = Stmt::expr(
            Expr::variable("a".to_string(), token(TokenType::Eof)),
            (0, 1).into(),
        );
        let err = interpreter
            .interpret_stmt(&stmt)
            .unwrap_err()
            .unwrap_runtime_error();
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
        let mut interpreter = Interpreter::from_printer(Box::new(printer.clone()));
        let _ = interpreter.interpret_stmt(&stmt).unwrap_err();
        let stmt = Stmt::expr(
            Expr::variable("a".to_string(), token(TokenType::Eof)),
            (0, 1).into(),
        );
        let err = interpreter
            .interpret_stmt(&stmt)
            .unwrap_err()
            .unwrap_runtime_error();
        assert_matches!(err, RuntimeError::UndefinedVariable { .. })
    }

    fn token(token_type: TokenType) -> Token {
        Token::new(
            token_type,
            (0, 1).into(),
            Arc::new(NamedSource::new("name", String::new())),
        )
    }

    fn block(stmts: Vec<Stmt>) -> Stmt {
        Stmt {
            stmt_type: StmtType::Block(stmts),
            location: (0, 1).into(),
            src: Arc::new(NamedSource::new("name", String::new())),
        }
    }

    fn literal(literal: Literal) -> Expr {
        Expr::literal(literal, &token(TokenType::Eof))
    }

    fn var(name: &str) -> Stmt {
        Stmt {
            stmt_type: StmtType::Var {
                name: name.into(),
                initializer: None,
            },
            location: (0, 1).into(),
            src: Arc::new(NamedSource::new("name", String::new())),
        }
    }
}
