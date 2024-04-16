use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::ast::name::Name;

use super::{function::Function, value::Value, Interpreter, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    class: Class, // TODO: reference or RC?
    fields: Rc<RefCell<HashMap<Name, Value>>>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Self {
            class,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Name) -> Option<Value> {
        self.fields.borrow().get(name).cloned().or(self
            .class
            .find_method(name)
            .map(|method| method.bind(self).into()))
    }

    pub fn set(&self, name: &Name, value: Value) {
        self.fields.borrow_mut().insert(name.clone(), value);
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

    pub fn find_method(&self, name: &Name) -> Option<Function> {
        self.methods.get(name).cloned()
    }

    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        let instance = Instance::new(self.clone());
        self.find_method(&Name::init())
            .map(|i| i.bind(&instance).call(interpreter, arguments))
            .transpose()?;
        Ok(Value::Instance(instance))
    }

    pub fn arity(&self) -> usize {
        self.find_method(&Name::init())
            .map(|m| m.arity())
            .unwrap_or(0)
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}
