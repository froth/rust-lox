use std::{fmt::Display, sync::Arc};

use miette::{NamedSource, SourceSpan};

use super::token::TokenType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameExpr {
    pub name: Name,
    pub location: SourceSpan,
    pub src: Arc<NamedSource<String>>,
}

impl NameExpr {
    pub fn this(location: SourceSpan, src: Arc<NamedSource<String>>) -> Self {
        NameExpr {
            name: Name::this(),
            location,
            src: src.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(String);

impl Name {
    pub fn new(name: String) -> Self {
        Name(name)
    }

    pub fn this() -> Self {
        Name(TokenType::This.to_string())
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.to_string())
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name(value)
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
