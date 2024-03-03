use std::collections::HashMap;

use crate::{ast::expr::Name, value::Value};

#[derive(Default)]
pub struct Environment {
    pub parent: Option<Box<Environment>>,
    values: HashMap<Name, Value>,
}

impl Environment {
    pub fn define(&mut self, key: Name, value: Value) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &Name) -> Option<Value> {
        self.values
            .get(key)
            .cloned()
            .or(self.parent.as_ref().and_then(|p| p.get(key)))
    }

    pub fn assign(&mut self, key: &Name, value: &Value) -> bool {
        let local_assigned = self
            .values
            .get_mut(key)
            .map(|old| *old = value.clone())
            .is_some();
        if local_assigned {
            true
        } else {
            self.parent
                .as_mut()
                .map(|p| p.assign(key, value))
                .unwrap_or(false)
        }
    }
}

#[cfg(test)]
mod environment_tests {
    use std::collections::HashMap;

    use crate::{ast::expr::Name, interpreter::environment::Environment, value::Value};

    pub fn local(parent: Environment) -> Environment {
        Environment {
            parent: Some(Box::new(parent)),
            values: HashMap::default(),
        }
    }

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

    #[test]
    fn define_get_from_parent() {
        let mut global = Environment::default();
        let name = Name::new("x".to_string());
        global.define(name.clone(), Value::Boolean(true));
        let local = local(global);
        let returned = local.get(&name);
        assert_eq!(returned, Some(Value::Boolean(true)))
    }

    #[test]
    fn define_assign_to_parent() {
        let mut global = Environment::default();
        let name = Name::new("x".to_string());
        global.define(name.clone(), Value::Nil);
        let mut local = local(global);
        let assigned = local.assign(&name, &Value::Boolean(false));
        assert!(assigned);
        global = *local.parent.unwrap();
        let returned = global.get(&name);
        assert_eq!(returned, Some(Value::Boolean(false)))
    }

    #[test]
    fn shadowing() {
        let mut global = Environment::default();
        let name = Name::new("x".to_string());
        global.define(name.clone(), Value::Nil);
        let mut local = local(global);
        local.define(name.clone(), Value::Boolean(false));
        let loc_return = local.get(&name);
        global = *local.parent.unwrap();
        let glob_return = global.get(&name);
        assert_eq!(loc_return, Some(Value::Boolean(false)));
        assert_eq!(glob_return, Some(Value::Nil))
    }
}
