use std::collections::HashMap;

use crate::{ast::expr::Name, value::Value};

#[derive(Default)]
pub struct Environment {
    values: HashMap<Name, Value>,
}
impl Environment {
    pub fn define(&mut self, key: Name, value: Value) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Name) -> Option<Value> {
        self.values.get(key).cloned()
    }

    pub fn assign(&mut self, key: &Name, value: &Value) -> bool {
        self.values.get_mut(key).map(|old| *old = value.clone()).is_some()
    }
}

#[cfg(test)]
mod environment_tests {
    use crate::{ast::expr::Name, value::Value};

    use super::Environment;

    #[test]
    fn define_get() {
        let mut env = Environment::default();
        let name = Name::new("x".to_string());
        env.define(name.clone(), Value::Boolean(true));
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(true)))
    }

    #[test]
    fn define_assign_get() {
        let mut env = Environment::default();
        let name = Name::new("x".to_string());
        env.define(name.clone(), Value::Boolean(true));
        let assigned = env.assign(&name, &Value::Boolean(false));
        assert!(assigned);
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(false)))
    }

    #[test]
    fn assign_unasigned() {
        let mut env = Environment::default();
        let name = Name::new("x".to_string());
        let assigned = env.assign(&name, &Value::Boolean(false));
        assert!(!assigned);
        let returned = env.get(&name);
        assert_eq!(returned, None)
    }
}
