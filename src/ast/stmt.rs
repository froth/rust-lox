use std::{fmt::Display, sync::Arc};

use miette::{NamedSource, SourceSpan};

use super::{
    expr::Expr,
    name::{Name, NameExpr},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub stmt_type: StmtType,
    pub location: SourceSpan,
    pub src: Arc<NamedSource<String>>,
}

impl Stmt {
    pub fn expr(expr: Expr, location: SourceSpan) -> Self {
        let src: Arc<NamedSource<String>> = expr.src.clone();
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
            stmt_type: StmtType::Var {
                name: name.into(),
                initializer: expr,
            },
            src,
            location,
        }
    }

    pub fn class(
        name: String,
        methods: Vec<Function>,
        superclass: Option<NameExpr>,
        location: SourceSpan,
        src: Arc<NamedSource<String>>,
    ) -> Self {
        Stmt {
            stmt_type: StmtType::Class {
                name: name.into(),
                methods,
                superclass,
            },
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
            stmt_type: StmtType::If {
                condition,
                then_stmt: then_stmt.into(),
                else_stmt: else_stmt.map(Box::new),
            },
            src,
            location,
        }
    }

    pub fn while_stmt(condition: Expr, body: Stmt, location: SourceSpan) -> Self {
        let src = condition.src.clone();
        Stmt {
            stmt_type: StmtType::While {
                condition,
                body: body.into(),
            },
            src,
            location,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtType {
    Expression(Expr),
    Print(Expr),
    Var {
        name: Name,
        initializer: Option<Expr>,
    },
    Function(Function),
    Return(Option<Expr>),
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_stmt: Box<Stmt>,
        else_stmt: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Class {
        name: Name,
        methods: Vec<Function>,
        superclass: Option<NameExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Name,
    pub parameters: Vec<Name>,
    pub body: Vec<Stmt>,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fun {}(", self.name)?;
        self.parameters
            .iter()
            .try_for_each(|arg| write!(f, "{arg}, "))?;
        writeln!(f, ") {{")?;
        self.body.iter().try_for_each(|s| write!(f, "{}", s))?;
        writeln!(f, "}}")
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StmtType::*;
        match &self.stmt_type {
            Expression(expr) => writeln!(f, "Expr{expr}"),
            Print(expr) => writeln!(f, "Print{expr}"),
            Var {
                name,
                initializer: Some(expr),
            } => writeln!(f, "Var {name} = {expr}"),
            Var {
                name,
                initializer: None,
            } => writeln!(f, "Var {name}"),
            Block(stmts) => {
                writeln!(f, "{{")?;
                stmts.iter().try_for_each(|s| write!(f, "{}", s))?;
                writeln!(f, "}}")
            }
            If {
                condition,
                then_stmt: then_branch,
                else_stmt: Some(else_branch),
            } => {
                writeln!(f, "if {}", condition)?;
                write!(f, "{}", then_branch)?;
                writeln!(f, "else")?;
                write!(f, "{}", else_branch)?;
                writeln!(f, "endif")
            }
            If {
                condition,
                then_stmt: then_branch,
                else_stmt: None,
            } => {
                writeln!(f, "if {}", condition)?;
                write!(f, "{}", then_branch)?;
                writeln!(f, "endif")
            }
            While { condition, body } => {
                writeln!(f, "while {} {{", condition)?;
                write!(f, "{}", body)?;
                writeln!(f, "}}")
            }
            Function(function) => write!(f, "{function}"),
            Return(None) => writeln!(f, "return"),
            Return(Some(expr)) => writeln!(f, "return {expr}"),
            Class {
                name,
                methods,
                superclass,
            } => {
                write!(f, "class {}", name)?;
                if let Some(superclass) = superclass {
                    write!(f, "< {}", superclass.name)?;
                }
                writeln!(f, "{{")?;
                methods.iter().try_for_each(|s| write!(f, "{}", s))?;
                writeln!(f, "}}")
            }
        }
    }
}
