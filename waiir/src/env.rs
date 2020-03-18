use super::object::*;
use std::cell::*;
use std::collections::*;
use std::rc::*;

pub fn new_enclosed_env(outer: Option<Rc<RefCell<Env>>>) -> Env {
    let mut env = new_env();
    env.outer = outer;
    env
}

pub fn new_env() -> Env {
    let s: HashMap<String, Option<Object>> = HashMap::new();
    Env {
        store: s,
        outer: None,
    }
}

#[derive(Debug)]
pub struct Env {
    store: HashMap<String, Option<Object>>,
    outer: Option<Rc<RefCell<Env>>>,
}
impl Env {
    pub fn get(&self, name: String) -> (Option<Object>, bool) {
        println!("***** env::get: {} {}", name, self.outer.is_some());
        if let Some(obj) = self.store.get(&name) {
            (obj.clone(), true)
        } else if let Some(o) = &self.outer {
            return o.borrow().get(name);
        } else {
            (None, false)
        }
    }

    pub fn set(&mut self, name: String, val: Option<Object>) -> Option<Object> {
        println!("******* env::set: {} => {:?}", name, val);
        self.store.insert(name, val.clone());
        val
    }
}
