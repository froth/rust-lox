use std::{fmt::Display, sync::Arc};

use miette::{NamedSource, SourceSpan};

use crate::{source_span_extensions::SourceSpanExtensions, token::Token};

#[derive(Debug)]
pub struct ExprWithContext {
    pub expr: Expr,
    pub location: SourceSpan,
    pub src: Arc<NamedSource<String>>,
}

impl ExprWithContext {
    pub fn new(expr: Expr, location: SourceSpan, src: Arc<NamedSource<String>>) -> Self {
        Self {
            expr,
            location,
            src,
        }
    }
    pub fn literal(literal: Literal, token: &Token) -> Self {
        Self {
            expr: Expr::literal(literal),
            location: token.location,
            src: token.src.clone(),
        }
    }

    pub fn unary(token: Token, expr: ExprWithContext) -> Self {
        let src = token.src.clone();
        let location = token.location.until(expr.location);
        Self {
            expr: Expr::unary(token, expr),
            location,
            src,
        }
    }

    pub fn binary(left: ExprWithContext, token: Token, right: ExprWithContext) -> Self {
        let src = token.src.clone();
        let location = left.location.until(right.location);
        Self {
            expr: Expr::binary(left, token, right),
            location,
            src,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Binary(Box<ExprWithContext>, Token, Box<ExprWithContext>),
    Grouping(Box<ExprWithContext>),
    Literal(Literal),
    Unary(Token, Box<ExprWithContext>),
}

impl Expr {
    pub fn binary(left: ExprWithContext, token: Token, right: ExprWithContext) -> Expr {
        Self::Binary(Box::new(left), token, Box::new(right))
    }

    pub fn grouping(expr: ExprWithContext) -> Expr {
        Self::Grouping(Box::new(expr))
    }

    pub fn literal(literal: Literal) -> Expr {
        Self::Literal(literal)
    }

    pub fn unary(token: Token, expr: ExprWithContext) -> Expr {
        Self::Unary(token, Box::new(expr))
    }
}

impl Display for ExprWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.expr {
            Expr::Binary(left, token, right) => {
                write!(f, "({} {} {})", token.token_type, left, right)
            }
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Literal(literal) => write!(f, "({})", literal),
            Expr::Unary(token, right) => write!(f, "({} {})", token.token_type, right),
        }
    }
}

#[derive(Debug)]
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
