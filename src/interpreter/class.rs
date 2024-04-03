use std::fmt::Display;

use crate::ast::name::Name;

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    class: Class, // TODO: reference or RC?
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self { class }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} instance>", self.class.name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Class {
    pub name: Name,
}
