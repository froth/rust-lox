use std::{fmt::Display, sync::Arc};

use miette::{NamedSource, SourceSpan};

use crate::{ast::token::Token, source_span_extensions::SourceSpanExtensions};

#[derive(Debug, Clone)]
pub struct NameExpr {
    pub name: Name,
    pub location: SourceSpan,
    pub src: Arc<NamedSource<String>>,
}
#[derive(Debug, Clone)]
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
            expr_type: ExprType::variable(name),
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
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.expr_type {
            ExprType::Binary(left, token, right) => {
                write!(f, "({} {} {})", token.token_type, left, right)
            }
            ExprType::Logical(left, token, right) => {
                write!(f, "(Logical {} {} {})", token.token_type, left, right)
            }
            ExprType::Grouping(expr) => write!(f, "(group {})", expr),
            ExprType::Literal(literal) => write!(f, "({})", literal),
            ExprType::Unary(token, expr) => write!(f, "({} {})", token.token_type, expr),
            ExprType::Variable(name) => write!(f, "(variable {})", name.0),
            ExprType::Assign(name, right) => write!(f, "({}={})", name.name, right),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprType {
    Assign(NameExpr, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
    Variable(Name),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.to_string())
    }
}

impl Name {
    pub fn new(s: String) -> Self {
        Name(s)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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

    pub fn variable(name: String) -> ExprType {
        Self::Variable(Name(name))
    }

    pub fn assign(name: NameExpr, expr: Expr) -> ExprType {
        Self::Assign(name, Box::new(expr))
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f32),
    Boolean(bool),
    Nil,
}
impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Boolean(b) => write!(f, "{}", b),
            Literal::Nil => write!(f, "nil"),
        }
    }
}
impl From<bool> for Literal {
    fn from(value: bool) -> Self {
        Literal::Boolean(value)
    }
}

impl From<f32> for Literal {
    fn from(value: f32) -> Self {
        Literal::Number(value)
    }
}

impl From<String> for Literal {
    fn from(value: String) -> Self {
        Literal::String(value)
    }
}

impl From<&str> for Literal {
    fn from(value: &str) -> Self {
        Literal::String(value.to_string())
    }
}
