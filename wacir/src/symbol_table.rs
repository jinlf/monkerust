// src/symbol_table.rs

use std::collections::*;

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolScope {
    GlobalScope,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: i64,
}

pub struct SymbolTable {
    pub store: HashMap<String, Symbol>,
    pub num_definitions: usize,
}
impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: &str) -> Symbol {
        let symbol = Symbol {
            name: String::from(name),
            scope: SymbolScope::GlobalScope,
            index: self.num_definitions as i64,
        };
        self.store.insert(String::from(name), symbol.clone());
        self.num_definitions += 1;
        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        match self.store.get(name) {
            Some(s) => Some(s.clone()),
            None => None,
        }
    }
}
