use std::{collections::HashMap, fmt::Display, time::SystemTime};

use crate::ast::name::Name;

use super::{value::Value, Interpreter, Result};

#[derive(Debug, Clone, PartialEq)]
pub struct Native {
    pub function: fn(interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value>,
    arity: usize,
    name: String,
}

impl Native {
    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Result<Value> {
        (self.function)(interpreter, arguments)
    }
}

impl Display for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fun {} ({} arguments)>", self.name, self.arity)
    }
}

pub fn native_functions() -> HashMap<Name, Native> {
    let mut builtins = HashMap::new();
    builtins.insert(
        "clock".into(),
        Native {
            function: clock,
            arity: 0,
            name: "clock".to_string(),
        },
    );
    builtins
}

fn clock(_: &mut Interpreter, _: Vec<Value>) -> Result<Value> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    Ok(Value::Number(now.as_secs_f64()))
}
