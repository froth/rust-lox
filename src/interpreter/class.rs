use std::{collections::HashMap, fmt::Display};

use crate::ast::name::Name;

use super::{function::Function, value::Value};

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
        self.fields
            .get(name)
            .cloned()
            .or(self.class.find_method(name))
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
    name: Name,
    methods: HashMap<Name, Function>,
}

impl Class {
    pub fn new(name: Name, methods: HashMap<Name, Function>) -> Self {
        Self { name, methods }
    }

    pub fn find_method(&self, name: &Name) -> Option<Value> {
        self.methods.get(name).cloned().map(|m| m.into())
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}
