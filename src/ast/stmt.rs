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
            stmt_type: StmtType::Var(Name::new(name), expr),
            src,
            location,
        }
    }

    pub fn if_stmt(
        condition: Expr,
        then_stmt: Stmt,
        else_stmt: Option<Stmt>,
        location: SourceSpan,
    ) -> Self {
        let src = condition.src.clone();
        Stmt {
            stmt_type: StmtType::If(condition, then_stmt.into(), else_stmt.map(Box::new)),
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
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StmtType::*;
        match &self.stmt_type {
            Expression(expr) => writeln!(f, "Expr{}", expr),
            Print(expr) => writeln!(f, "Print{}", expr),
            Var(name, Some(expr)) => writeln!(f, "Var {} = {}", name, expr),
            Var(name, None) => writeln!(f, "Var {}", name),
            Block(stmts) => {
                writeln!(f, "{{")?;
                stmts.iter().try_for_each(|s| write!(f, "{}", s))?;
                writeln!(f, "}}")
            }
            If(condition, then_branch, Some(else_branch)) => {
                writeln!(f, "if {}", condition)?;
                write!(f, "{}", then_branch)?;
                writeln!(f, "else")?;
                write!(f, "{}", else_branch)?;
                writeln!(f, "endif")
            }
            If(condition, then_branch, None) => {
                writeln!(f, "if {}", condition)?;
                write!(f, "{}", then_branch)?;
                writeln!(f, "endif")
            }
        }
    }
}
