use std::collections::HashMap;

use crate::{ast::expr::Name, value::Value};

pub struct Environment {
    values: HashMap<Name, Value>
}
impl Environment {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }
}