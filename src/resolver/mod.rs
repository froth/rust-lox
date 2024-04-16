use std::{collections::HashMap, sync::Arc};
pub mod resolution_error;

use miette::{NamedSource, SourceSpan};

use crate::ast::{
    expr::{Expr, ExprType::*},
    name::{Name, NameExpr},
    stmt::{Function, Stmt, StmtType::*},
};

use self::resolution_error::ResolutionError;

#[derive(Debug, Default)]
pub struct Resolver {
    locals: HashMap<NameExpr, usize>,
    scopes: Vec<HashMap<Name, bool>>,
    current_function: Option<FunctionType>,
    current_class: Option<ClassType>,
}

#[derive(Debug, PartialEq)]
enum FunctionType {
    Function,
    Initializer,
    Method,
}

#[derive(Debug)]
enum ClassType {
    Class,
}

type Result<T> = std::result::Result<T, ResolutionError>;

impl Resolver {
    pub fn resolve(statements: &[Stmt], verbose: bool) -> Result<HashMap<NameExpr, usize>> {
        let mut resolver = Resolver::default();
        resolver.resolve_statements(statements)?;
        if verbose {
            eprintln!("{:?}", resolver.locals);
        }
        Ok(resolver.locals)
    }

    pub fn resolve_expression(
        expression: &Expr,
        verbose: bool,
    ) -> Result<HashMap<NameExpr, usize>> {
        let mut resolver = Resolver::default();
        resolver.resolve_expr(expression)?;
        if verbose {
            eprintln!("{:?}", resolver.locals);
        }
        Ok(resolver.locals)
    }

    fn resolve_statements(&mut self, statements: &[Stmt]) -> Result<()> {
        statements
            .iter()
            .try_for_each(|s| self.resolve_statement(s))
    }

    fn resolve_statement(&mut self, statement: &Stmt) -> Result<()> {
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
            Class { name, methods } => self.resolve_class(name, methods),
        }
    }

    fn resolve_block(&mut self, statements: &[Stmt]) -> Result<()> {
        self.begin_scope();
        self.resolve_statements(statements)?;
        self.end_scope();
        Ok(())
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

    fn resolve_class(&mut self, name: &Name, methods: &[Function]) -> Result<()> {
        let enclosing_class = std::mem::replace(&mut self.current_class, Some(ClassType::Class));
        self.declare(name);
        self.define(name);
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
        self.current_class = enclosing_class;
        Ok(())
    }

    fn resolve_expr(&mut self, expression: &Expr) -> Result<()> {
        match &expression.expr_type {
            Assign(name_expr, expr) => {
                self.resolve_expr(expr)?;
                self.resolve_local(name_expr);
                Ok(())
            }
            Binary(lhs, _, rhs) => {
                self.resolve_expr(lhs)?;
                self.resolve_expr(rhs)
            }
            Logical(lhs, _, rhs) => {
                self.resolve_expr(lhs)?;
                self.resolve_expr(rhs)
            }
            Grouping(expr) => self.resolve_expr(expr),
            Literal(_) => Ok(()),
            Unary(_, expr) => self.resolve_expr(expr),
            Variable(name_expr) => self.resolve_var_expr(name_expr),
            Call(name, arguments) => {
                self.resolve_expr(name)?;
                arguments.iter().try_for_each(|e| self.resolve_expr(e))
            }
            Get(expr, _) => self.resolve_expr(expr),
            Set(expr, _, object) => {
                self.resolve_expr(expr)?;
                self.resolve_expr(object)
            }
            This => self.resolve_this(expression.location, &expression.src),
        }
    }

    fn resolve_var_expr(&mut self, name_expr: &NameExpr) -> Result<()> {
        if let Some(false) = self.scopes.last().and_then(|s| s.get(&name_expr.name)) {
            Err(ResolutionError::InitializedWithSelf {
                name: name_expr.name.clone(),
                src: name_expr.src.clone(),
                location: name_expr.location,
            })
        } else {
            self.resolve_local(name_expr);
            Ok(())
        }
    }

    fn resolve_this(&mut self, location: SourceSpan, src: &Arc<NamedSource<String>>) -> Result<()> {
        if self.current_class.is_none() {
            Err(ResolutionError::InvalidThis {
                src: src.clone(),
                location,
            })
        } else {
            let name_expr = NameExpr::this(location, src.clone());
            self.resolve_local(&name_expr);
            Ok(())
        }
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
    fn declare(&mut self, name: &Name) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), false);
        }
    }

    fn define(&mut self, name: &Name) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.clone(), true);
        }
    }

    fn resolve_local(&mut self, name_expr: &NameExpr) {
        let resolved = self
            .scopes
            .iter()
            .rev()
            .enumerate()
            .find(|(_index, scope)| scope.contains_key(&name_expr.name));
        if let Some((index, _)) = resolved {
            self.locals.insert(name_expr.clone(), index);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
