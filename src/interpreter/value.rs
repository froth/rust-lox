use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::{callable::Callable, class::Instance, types::Type};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Callable(Callable),
    Instance(Rc<RefCell<Instance>>),
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(bool) => *bool,
            Value::Nil => false,
            _ => true,
        }
    }

    pub fn get_type(&self) -> Type {
        match self {
            Value::Callable(_) => Type::Callable,
            Value::Instance(_) => Type::Instance,
            Value::String(_) => Type::String,
            Value::Number(_) => Type::Number,
            Value::Boolean(_) => Type::Boolean,
            Value::Nil => Type::Nil,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Callable(c) => write!(f, "{}", c),
            Value::Instance(instance) => write!(f, "{}", instance.borrow()),
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "Nil"),
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Number(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}
