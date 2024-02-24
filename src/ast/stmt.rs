use std::{fmt::Display, sync::Arc};

use miette::{NamedSource, SourceSpan};

use super::expr::{Expr, Name};

#[derive(Debug)]
pub struct Stmt {
    pub stmt_type: StmtType,
    pub location: SourceSpan,
    pub src: Arc<NamedSource<String>>,
}

impl Stmt {
    pub fn expr(expr: Expr, location: SourceSpan) -> Self {
        let src = expr.src.clone();
        Stmt {
            stmt_type: StmtType::Expression(expr),
            location,
            src,
        }
    }
    pub fn print(expr: Expr, location: SourceSpan) -> Self {
        let src = expr.src.clone();
        Stmt {
            stmt_type: StmtType::Print(expr),
            location,
            src,
        }
    }

    pub fn var(
        name: String,
        expr: Option<Expr>,
        location: SourceSpan,
        src: Arc<NamedSource<String>>,
    ) -> Self {
        Stmt {
            stmt_type: StmtType::Var(Name(name), expr),
            src,
            location,
        }
    }
}

#[derive(Debug)]
pub enum StmtType {
    Expression(Expr),
    Print(Expr),
    Var(Name, Option<Expr>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.stmt_type {
            StmtType::Expression(expr) => {
                writeln!(f, "Expr{}", expr)
            }
            StmtType::Print(expr) => writeln!(f, "Print{}", expr),
            StmtType::Var(name, Some(expr)) => writeln!(f, "Var {} = {}", name, expr),
            StmtType::Var(name, None) => writeln!(f, "Var {}", name),
        }
    }
}
