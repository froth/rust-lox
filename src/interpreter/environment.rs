use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::ast::name::Name;

use super::{native_functions::native_functions, value::Value};

#[derive(Debug, PartialEq)]
pub struct Environment {
    pub parent: Option<Rc<RefCell<Environment>>>,
    values: HashMap<Name, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            parent: None,
            values: HashMap::new(),
        }
    }

    pub fn from_parent(parent: Rc<RefCell<Environment>>) -> Self {
        Environment {
            parent: Some(parent),
            values: HashMap::new(),
        }
    }

    pub fn with_native_functions() -> Self {
        let mut env = Self::new();
        native_functions()
            .into_iter()
            .for_each(|(k, v)| env.define(&k, Value::Callable(v)));
        env
    }

    pub fn define(&mut self, key: &Name, value: Value) {
        self.values.insert(key.clone(), value);
    }

    pub fn get(&self, key: &Name) -> Option<Value> {
        self.values
            .get(key)
            .cloned()
            .or(self.parent.as_ref().and_then(|p| p.borrow().get(key)))
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
                .map(|p| p.borrow_mut().assign(key, value))
                .unwrap_or(false)
        }
    }

    pub fn get_at(&self, distance: usize, name: &Name) -> Option<Value> {
        if distance == 0 {
            self.get(name)
        } else {
            self.parent
                .as_ref()
                .and_then(|p| p.borrow().get_at(distance - 1, name))
        }
    }

    pub fn assign_at(&mut self, distance: usize, name: &Name, value: &Value) -> bool {
        if distance == 0 {
            self.assign(name, value)
        } else {
            self.parent
                .as_ref()
                .map(|p| p.borrow_mut().assign_at(distance - 1, name, value))
                .expect("guaranteed by resolver")
        }
    }
}

#[cfg(test)]
mod environment_tests {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use crate::interpreter::{environment::Environment, value::Value};

    pub fn local(parent: Environment) -> Environment {
        Environment {
            parent: Some(Rc::new(RefCell::new(parent))),
            values: HashMap::default(),
        }
    }

    #[test]
    fn define_get() {
        let mut env = Environment::new();
        let name = "x".into();
        env.define(&name, Value::Boolean(true));
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(true)))
    }

    #[test]
    fn define_assign_get() {
        let mut env = Environment::new();
        let name = "x".into();
        env.define(&name, Value::Boolean(true));
        let assigned = env.assign(&name, &Value::Boolean(false));
        assert!(assigned);
        let returned = env.get(&name);
        assert_eq!(returned, Some(Value::Boolean(false)))
    }

    #[test]
    fn assign_unasigned() {
        let mut env = Environment::new();
        let name = "x".into();
        let assigned = env.assign(&name, &Value::Boolean(false));
        assert!(!assigned);
        let returned = env.get(&name);
        assert_eq!(returned, None)
    }

    #[test]
    fn define_get_from_parent() {
        let mut global = Environment::new();
        let name = "x".into();
        global.define(&name, Value::Boolean(true));
        let local = local(global);
        let returned = local.get(&name);
        assert_eq!(returned, Some(Value::Boolean(true)))
    }

    #[test]
    fn define_assign_to_parent() {
        let mut global = Environment::new();
        let name = "x".into();
        global.define(&name, Value::Nil);
        let mut local = local(global);
        let assigned = local.assign(&name, &Value::Boolean(false));
        assert!(assigned);
        let parent: Rc<RefCell<Environment>> = local.parent.unwrap();
        let returned = parent.borrow_mut().get(&name);
        assert_eq!(returned, Some(Value::Boolean(false)))
    }

    #[test]
    fn shadowing() {
        let mut global = Environment::new();
        let name = "x".into();
        global.define(&name, Value::Nil);
        let mut local = local(global);
        local.define(&name, Value::Boolean(false));
        let loc_return = local.get(&name);
        let parent = local.parent.unwrap();
        let glob_return = parent.borrow_mut().get(&name);
        assert_eq!(loc_return, Some(Value::Boolean(false)));
        assert_eq!(glob_return, Some(Value::Nil))
    }
}
