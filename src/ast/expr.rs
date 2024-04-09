use std::{fmt::Display, sync::Arc};

use miette::{NamedSource, SourceSpan};

use crate::{ast::token::Token, source_span_extensions::SourceSpanExtensions};

use super::{
    literal::Literal,
    name::{Name, NameExpr},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub expr_type: ExprType,
    pub location: SourceSpan,
    pub src: Arc<NamedSource<String>>,
}

impl Expr {
    pub fn new(expr_type: ExprType, location: SourceSpan, src: Arc<NamedSource<String>>) -> Self {
        Self {
            expr_type,
            location,
            src,
        }
    }
    pub fn literal(literal: Literal, token: &Token) -> Self {
        Self {
            expr_type: ExprType::literal(literal),
            location: token.location,
            src: token.src.clone(),
        }
    }

    pub fn unary(token: Token, expr: Expr) -> Self {
        let src = token.src.clone();
        let location = token.location.until(expr.location);
        Self {
            expr_type: ExprType::unary(token, expr),
            location,
            src,
        }
    }

    pub fn binary(left: Expr, token: Token, right: Expr) -> Self {
        let src = token.src.clone();
        let location = left.location.until(right.location);
        Self {
            expr_type: ExprType::binary(left, token, right),
            location,
            src,
        }
    }

    pub fn logical(left: Expr, token: Token, right: Expr) -> Self {
        let src = token.src.clone();
        let location = left.location.until(right.location);
        Self {
            expr_type: ExprType::logical(left, token, right),
            location,
            src,
        }
    }

    pub fn variable(name: String, token: Token) -> Self {
        let src = token.src.clone();
        let location = token.location;
        Self {
            expr_type: ExprType::variable(name, src.clone(), location),
            location,
            src,
        }
    }

    pub fn assign(name: NameExpr, expr: Expr) -> Self {
        let src = expr.src.clone();
        let location = name.location.until(expr.location);
        Self {
            expr_type: ExprType::assign(name, expr),
            location,
            src,
        }
    }

    pub fn call(callee: Expr, arguments: Vec<Expr>, location: SourceSpan) -> Self {
        let src = callee.src.clone();
        Self {
            expr_type: ExprType::call(callee, arguments),
            location,
            src,
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ExprType::*;
        match &self.expr_type {
            Binary(left, token, right) => write!(f, "({} {} {})", token.token_type, left, right),
            Logical(left, token, right) => {
                write!(f, "(Logical {} {} {})", token.token_type, left, right)
            }
            Grouping(expr) => write!(f, "(group {})", expr),
            Literal(literal) => write!(f, "({})", literal),
            Unary(token, expr) => write!(f, "({} {})", token.token_type, expr),
            Variable(name) => write!(f, "(variable {})", name.name),
            Assign(name, right) => write!(f, "({}={})", name.name, right),
            Call(callee, arguments) => {
                write!(f, "(Call {}=>(", callee)?;
                arguments
                    .iter()
                    .try_for_each(|arg| write!(f, "{}, ", arg))?;
                write!(f, "))")
            }
            Get(object, name) => write!(f, "(Get {}.{})", object, name.name),
            Set(object, name, value) => write!(f, "(Set {}.{} = {})", object, name.name, value),
            This => write!(f, "this"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
    Assign(NameExpr, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
    Variable(NameExpr),
    Call(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, NameExpr),
    Set(Box<Expr>, NameExpr, Box<Expr>),
    This,
}

impl ExprType {
    pub fn binary(left: Expr, token: Token, right: Expr) -> ExprType {
        Self::Binary(Box::new(left), token, Box::new(right))
    }

    pub fn logical(left: Expr, token: Token, right: Expr) -> ExprType {
        Self::Logical(Box::new(left), token, Box::new(right))
    }

    pub fn grouping(expr: Expr) -> ExprType {
        Self::Grouping(Box::new(expr))
    }

    pub fn literal(literal: Literal) -> ExprType {
        Self::Literal(literal)
    }

    pub fn unary(token: Token, expr: Expr) -> ExprType {
        Self::Unary(token, Box::new(expr))
    }

    pub fn variable(name: String, src: Arc<NamedSource<String>>, location: SourceSpan) -> ExprType {
        Self::Variable(NameExpr {
            name: Name::new(name),
            location,
            src: src.clone(),
        })
    }

    pub fn assign(name: NameExpr, expr: Expr) -> ExprType {
        Self::Assign(name, Box::new(expr))
    }

    pub fn call(callee: Expr, arguments: Vec<Expr>) -> ExprType {
        Self::Call(callee.into(), arguments)
    }
}
