// src/environment.rs

use crate::object::*;
use std::cell::*;
use std::collections::*;
use std::rc::*;

pub fn new_enclosed_environment(outer: Option<Rc<RefCell<Environment>>>) -> Environment {
    let mut env = new_environment();
    env.outer = outer;
    env
}

pub fn new_environment() -> Environment {
    Environment {
        store: HashMap::new(),
        outer: None,
    }
}

#[derive(Debug)]
pub struct Environment {
    pub store: HashMap<String, Object>,
    pub outer: Option<Rc<RefCell<Environment>>>,
}
impl Environment {
    pub fn get(&self, name: &str) -> Option<Object> {
        if let Some(v) = self.store.get(name) {
            return Some(v.clone());
        } else if let Some(o) = &self.outer {
            return o.borrow().get(name);
        }
        None
    }
    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val.clone());
        val
    }
}
