// src/symbol_table.rs

use std::cell::*;
use std::collections::*;
use std::rc::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolScope {
    GlobalScope,
    LocalScope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: i64,
}

pub struct SymbolTable {
    pub outer: Option<Rc<RefCell<SymbolTable>>>,
    pub store: HashMap<String, Symbol>,
    pub num_definitions: usize,
}
impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            outer: None,
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: &str) -> Symbol {
        let mut scope = SymbolScope::GlobalScope;
        if self.outer.is_some() {
            scope = SymbolScope::LocalScope;
        }
        let symbol = Symbol {
            name: String::from(name),
            scope: scope,
            index: self.num_definitions as i64,
        };
        self.store.insert(String::from(name), symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        match self.store.get(name) {
            Some(s) => Some(s.clone()),
            None => {
                if let Some(outer) = &self.outer {
                    return outer.borrow().resolve(name);
                }
                None
            }
        }
    }

    pub fn new_enclosed_symbol_table(outer: Rc<RefCell<SymbolTable>>) -> SymbolTable {
        SymbolTable {
            outer: Some(Rc::clone(&outer)),
            store: HashMap::new(),
            num_definitions: 0,
        }
    }
}
