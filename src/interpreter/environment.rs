use std::collections::HashMap;

use crate::{ast::expr::Name, value::Value};

pub struct Environment {
    values: HashMap<Name, Value>,
}
impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, key: Name, value: Value) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Name) -> Option<Value> {
        self.values.get(key).cloned()
    }

    pub fn assign(&mut self, key: Name, value: Value)-> Option<()> {
        self.values.get_mut(&key).map(|old| *old = value)
    }
}

#[cfg(test)]
mod environment_tests {
    use crate::{ast::expr::Name, value::Value};

    use super::Environment;

    #[test]
    fn define_get() {
        let mut env = Environment::new();
        let name = Name::new("x".to_string());
        env.define(name.clone(), Value::Boolean(true));
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(true)))
    }

    #[test]
    fn define_assign_get() {
        let mut env = Environment::new();
        let name = Name::new("x".to_string());
        env.define(name.clone(), Value::Boolean(true));
        let assigned = env.assign(name.clone(), Value::Boolean(false));
        assert_eq!(assigned, Some(()));
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(false)))
    }

    #[test]
    fn assign_unasigned() {
        let mut env = Environment::new();
        let name = Name::new("x".to_string());
        let assigned = env.assign(name.clone(), Value::Boolean(false));
        assert_eq!(assigned, None);
        let returned = env.get(&name);
        assert_eq!(returned, None)
    }
}