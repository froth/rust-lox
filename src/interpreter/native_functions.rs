use std::{collections::HashMap, time::SystemTime};

use crate::ast::name::Name;

use super::{callable::Callable, value::Value, Interpreter, Result};

pub fn native_functions() -> HashMap<Name, Callable> {
    let mut builtins = HashMap::new();
    builtins.insert(
        "clock".into(),
        Callable::Native {
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
