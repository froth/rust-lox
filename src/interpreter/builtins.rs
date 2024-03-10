use std::{collections::HashMap, time::SystemTime};

use crate::ast::expr::Name;

use super::{callable::Callable, value::Value, Interpreter, Result};
pub fn builtins() -> HashMap<Name, Callable> {
    let mut builtins = HashMap::new();
    builtins.insert("clock".into(), Callable::Builtin(clock));
    builtins
}

fn clock(_: &mut Interpreter, _: Vec<Value>) -> Result<Value> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    Ok(Value::Number(now.as_secs_f64()))
}
