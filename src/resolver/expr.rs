use std::sync::Arc;

use miette::{NamedSource, SourceSpan};

use super::{resolution_error::ResolutionError, Resolver, Result};
use crate::ast::{
    expr::{Expr, ExprType::*},
    name::NameExpr,
};

impl Resolver {
    pub(super) fn resolve_expr(&mut self, expression: &Expr) -> Result<()> {
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
            Super(_) => self.resolve_super(expression.location, &expression.src),
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

    fn resolve_super(
        &mut self,
        location: SourceSpan,
        src: &Arc<NamedSource<String>>,
    ) -> Result<()> {
        // if self.current_class.is_none() {
        //     Err(ResolutionError::InvalidThis {
        //         src: src.clone(),
        //         location,
        //     })
        // } else {
        let name_expr = NameExpr::super_name(location, src.clone());
        self.resolve_local(&name_expr);
        Ok(())
        // }
    }
}
