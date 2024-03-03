use std::collections::HashMap;

use crate::{ast::expr::Name, value::Value};

pub trait Environment {
    fn define(&mut self, key: Name, value: Value);
    fn get(&self, key: &Name) -> Option<Value>;
    fn assign(&mut self, key: &Name, value: &Value) -> bool;
}
#[derive(Default)]
pub struct GlobalEnvironment {
    values: HashMap<Name, Value>,
}

pub struct LocalEnvironment<'a> {
    parent: &'a mut dyn Environment,
    values: HashMap<Name, Value>,
}

impl<'a> LocalEnvironment<'a> {
    pub fn new(parent: &'a mut dyn Environment) -> Self {
        Self {
            parent,
            values: HashMap::default(),
        }
    }
}

impl Environment for GlobalEnvironment {
    fn define(&mut self, key: Name, value: Value) {
        self.values.insert(key, value);
    }

    fn get(&self, key: &Name) -> Option<Value> {
        self.values.get(key).cloned()
    }

    fn assign(&mut self, key: &Name, value: &Value) -> bool {
        self.values
            .get_mut(key)
            .map(|old| *old = value.clone())
            .is_some()
    }
}

impl Environment for LocalEnvironment<'_> {
    fn define(&mut self, key: Name, value: Value) {
        self.values.insert(key, value);
    }

    fn get(&self, key: &Name) -> Option<Value> {
        self.values.get(key).cloned().or(self.parent.get(key))
    }

    fn assign(&mut self, key: &Name, value: &Value) -> bool {
        let local_assigned = self
            .values
            .get_mut(key)
            .map(|old| *old = value.clone())
            .is_some();
        if local_assigned {
            true
        } else {
            let environment = &mut self.parent;
            environment.assign(key, value)
        }
    }
}

#[cfg(test)]
mod environment_tests {
    use crate::{
        ast::expr::Name,
        interpreter::environment::{Environment, LocalEnvironment},
        value::Value,
    };

    use super::GlobalEnvironment;

    #[test]
    fn define_get() {
        let mut env = GlobalEnvironment::default();
        let name = Name::new("x".to_string());
        env.define(name.clone(), Value::Boolean(true));
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(true)))
    }

    #[test]
    fn define_assign_get() {
        let mut env = GlobalEnvironment::default();
        let name = Name::new("x".to_string());
        env.define(name.clone(), Value::Boolean(true));
        let assigned = env.assign(&name, &Value::Boolean(false));
        assert!(assigned);
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(false)))
    }

    #[test]
    fn assign_unasigned() {
        let mut env = GlobalEnvironment::default();
        let name = Name::new("x".to_string());
        let assigned = env.assign(&name, &Value::Boolean(false));
        assert!(!assigned);
        let returned = env.get(&name);
        assert_eq!(returned, None)
    }

    #[test]
    fn define_get_from_parent() {
        let mut global = GlobalEnvironment::default();
        let name = Name::new("x".to_string());
        global.define(name.clone(), Value::Boolean(true));
        let local = LocalEnvironment::new(&mut global);
        let returned = local.get(&name);
        assert_eq!(returned, Some(Value::Boolean(true)))
    }

    #[test]
    fn define_assign_to_parent() {
        let mut global = GlobalEnvironment::default();
        let name = Name::new("x".to_string());
        global.define(name.clone(), Value::Nil);
        let mut local = LocalEnvironment::new(&mut global);
        let assigned = local.assign(&name, &Value::Boolean(false));
        assert!(assigned);
        let returned = global.get(&name);
        assert_eq!(returned, Some(Value::Boolean(false)))
    }

    #[test]
    fn shadowing() {
        let mut global = GlobalEnvironment::default();
        let name = Name::new("x".to_string());
        global.define(name.clone(), Value::Nil);
        let mut local = LocalEnvironment::new(&mut global);
        local.define(name.clone(), Value::Boolean(false));
        let loc_return = local.get(&name);
        let glob_return = global.get(&name);
        assert_eq!(loc_return, Some(Value::Boolean(false)));
        assert_eq!(glob_return, Some(Value::Nil))
    }
}
