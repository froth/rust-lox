use std::fmt::Display;

use miette::SourceSpan;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameExpr {
    pub name: Name,
    pub location: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name(pub(super) String);

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
