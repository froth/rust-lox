use std::collections::HashMap;
mod expr;
pub mod resolution_error;
mod statement;

use crate::ast::{
    expr::Expr,
    name::{Name, NameExpr},
    stmt::Stmt,
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
            eprintln!("Locals:");
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
