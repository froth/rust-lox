use std::sync::Arc;

use miette::{NamedSource, SourceSpan};

use super::{resolution_error::ResolutionError, ClassType, FunctionType, Resolver, Result};
use crate::ast::{
    expr::Expr,
    name::{Name, NameExpr},
    stmt::{Function, Stmt, StmtType::*},
};

impl Resolver {
    pub(super) fn resolve_statement(&mut self, statement: &Stmt) -> Result<()> {
        match &statement.stmt_type {
            Expression(expr) => self.resolve_expr(expr),
            Print(expr) => self.resolve_expr(expr),
            Var { name, initializer } => self.resolve_var(name, initializer),
            Function(function) => {
                self.declare(&function.name);
                self.define(&function.name);
                self.resolve_function(&function.parameters, &function.body, FunctionType::Function)
            }
            Return(expr) => self.resolve_return(expr, statement.location, &statement.src),
            Block(statements) => self.resolve_block(statements),
            If {
                condition,
                then_stmt,
                else_stmt,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_statement(then_stmt)?;
                else_stmt.iter().try_for_each(|s| self.resolve_statement(s))
            }
            While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_statement(body)
            }
            Class {
                name,
                methods,
                superclass,
            } => self.resolve_class(name, methods, superclass),
        }
    }

    fn resolve_function(
        &mut self,
        parameters: &[Name],
        body: &[Stmt],
        function_type: FunctionType,
    ) -> Result<()> {
        let enclosing_function = std::mem::replace(&mut self.current_function, Some(function_type));
        self.begin_scope();
        parameters.iter().for_each(|p| {
            self.declare(p);
            self.define(p);
        });
        self.resolve_statements(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn resolve_var(&mut self, name: &Name, initializer: &Option<Expr>) -> Result<()> {
        self.declare(name);
        initializer.iter().try_for_each(|e| self.resolve_expr(e))?;
        self.define(name);
        Ok(())
    }

    fn resolve_class(
        &mut self,
        name: &Name,
        methods: &[Function],
        superclass: &Option<NameExpr>,
    ) -> Result<()> {
        let class_type = if superclass.is_some() {
            ClassType::Subclass
        } else {
            ClassType::Class
        };
        let enclosing_class = std::mem::replace(&mut self.current_class, Some(class_type));
        self.declare(name);
        self.define(name);

        if let Some(superclass) = superclass {
            if superclass.name == *name {
                return Err(ResolutionError::SelfInheritance {
                    src: superclass.src.clone(),
                    location: superclass.location,
                });
            } else {
                self.resolve_local(superclass)
            }
        }

        if superclass.is_some() {
            self.begin_scope();
            let scope = self.scopes.last_mut().expect("scope declared above");
            scope.insert(Name::super_name(), true);
        }

        self.begin_scope();

        self.define(&Name::this());
        methods.iter().try_for_each(|m| {
            let function_type = if m.name == Name::init() {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(&m.parameters, &m.body, function_type)
        })?;
        self.end_scope();

        if superclass.is_some() {
            self.end_scope();
        }

        self.current_class = enclosing_class;
        Ok(())
    }

    fn resolve_block(&mut self, statements: &[Stmt]) -> Result<()> {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();
        Ok(())
    }

    fn resolve_return(
        &mut self,
        expr: &Option<Expr>,
        location: SourceSpan,
        src: &Arc<NamedSource<String>>,
    ) -> Result<()> {
        if self.current_function.is_none() {
            Err(ResolutionError::InvalidReturn {
                src: src.clone(),
                location,
            })
        } else {
            expr.iter().try_for_each(|e| {
                if self
                    .current_function
                    .as_ref()
                    .is_some_and(|c| *c == FunctionType::Initializer)
                {
                    Err(ResolutionError::ReturnInInitializer {
                        src: src.clone(),
                        location,
                    })
                } else {
                    self.resolve_expr(e)
                }
            })
        }
    }
}
