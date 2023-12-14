use std::fmt::Display;

use crate::token::Token;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    Unary(Token, Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary(left, token, right) => write!(f, "({} {} {})", token.lexeme, left, right),
            Self::Grouping(expr) => write!(f, "(group {}", expr),
            Self::Literal(literal) =>  write!(f, "({})", literal),
            Self::Unary(token, right) => write!(f, "({} {})", token.lexeme, right),
        }
    }
}


#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f32),
    Nil,
}
impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Nil => write!(f, "nil"),
        }
    }
}
