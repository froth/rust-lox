use std::{collections::HashMap, fmt::Display};

use crate::ast::name::Name;

use super::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    class: Class, // TODO: reference or RC?
    fields: HashMap<Name, Value>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Name) -> Option<Value> {
        self.fields.get(name).cloned()
    }

    pub fn set(&mut self, name: &Name, value: Value) {
        self.fields.insert(name.clone(), value);
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
