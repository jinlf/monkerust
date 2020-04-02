// src/environment.rs

use super::object::*;
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
    pub store: HashMap<String, Option<Object>>,
    pub outer: Option<Rc<RefCell<Environment>>>,
}
impl Environment {
    pub fn get(&self, name: String) -> Option<Option<Object>> {
        if let Some(v) = self.store.get(&name) {
            if let Some(vv) = v {
                return Some(Some(vv.clone()));
            }
        } else if let Some(o) = &self.outer {
            return o.borrow().get(name);
        }
        None
    }
    pub fn set(&mut self, name: String, val: Option<Object>) -> Option<Object> {
        if let Some(v) = val {
            self.store.insert(name, Some(v.clone()));
            Some(v)
        } else {
            self.store.insert(name, None);
            None
        }
    }
}
