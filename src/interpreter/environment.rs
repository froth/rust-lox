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
        self.values.get(key).cloned()
    }

    pub fn assign(&mut self, key: &Name, value: &Value) -> bool {
        self.values
            .get_mut(key)
            .map(|old| *old = value.clone())
            .is_some()
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
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        ast::name::Name,
        interpreter::{environment::Environment, value::Value},
    };

    fn create_depth(depth: usize, top: Environment) -> Environment {
        assert!(depth > 0);
        let mut global = top;
        for _ in 1..depth {
            global = Environment::from_parent(Rc::new(RefCell::new(global)));
        }
        global
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
    fn assign_and_get_from_depth() {
        let name: Name = "a".into();
        let value = Value::Boolean(true);
        let mut top = Environment::new();
        top.define(&name, Value::Nil);
        let mut env = create_depth(8, top);
        env.assign_at(7, &name, &value);
        let ret = env.get_at(7, &name).unwrap();
        assert_eq!(value, ret)
    }
}
