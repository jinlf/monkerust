// src/symbol_table.rs

use std::cell::*;
use std::collections::*;
use std::rc::*;

#[derive(Debug, PartialEq, Clone)]
pub enum SymbolScope {
    GlobalScope,
    LocalScope,
    BuiltinScope,
    FreeScope,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Symbol {
    pub name: String,
    pub scope: SymbolScope,
    pub index: isize,
}

pub struct SymbolTable {
    pub outer: Option<Rc<RefCell<SymbolTable>>>,
    pub store: HashMap<String, Symbol>,
    pub num_definitions: usize,
    pub free_symbols: Vec<Symbol>,
}
impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable {
            outer: None,
            store: HashMap::new(),
            num_definitions: 0,
            free_symbols: Vec::new(),
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
            index: self.num_definitions as isize,
        };
        self.store.insert(String::from(name), symbol);
        self.num_definitions += 1;
        self.store[name].clone()
    }

    pub fn resolve(&mut self, name: &str) -> Option<Symbol> {
        match self.store.get(name) {
            Some(s) => return Some(s.clone()),
            None => {}
        };
        let mut need_define_free = false;
        let mut saved_obj: Option<Symbol> = None;
        {
            if let Some(outer) = self.outer.as_ref() {
                if let Some(obj) = outer.borrow_mut().resolve(name) {
                    match obj.scope {
                        SymbolScope::GlobalScope | SymbolScope::BuiltinScope => {
                            return Some(obj);
                        }
                        _ => {
                            need_define_free = true;
                            saved_obj = Some(obj);
                        }
                    }
                }
            }
        }
        if need_define_free {
            return Some(self.define_free(saved_obj.unwrap()));
        }

        return None;
    }

    pub fn new_enclosed_symbol_table(outer: Rc<RefCell<SymbolTable>>) -> SymbolTable {
        SymbolTable {
            outer: Some(Rc::clone(&outer)),
            store: HashMap::new(),
            num_definitions: 0,
            free_symbols: Vec::new(),
        }
    }

    pub fn define_builtin(&mut self, index: usize, name: &str) -> Symbol {
        let symbol = Symbol {
            name: String::from(name),
            scope: SymbolScope::BuiltinScope,
            index: index as isize,
        };
        self.store.insert(String::from(name), symbol.clone());
        symbol
    }

    pub fn define_free(&mut self, original: Symbol) -> Symbol {
        let original_name = original.name.clone();
        self.free_symbols.push(original);

        let symbol = Symbol {
            name: original_name.clone(),
            scope: SymbolScope::FreeScope,
            index: self.free_symbols.len() as isize - 1,
        };
        self.store.insert(original_name, symbol.clone());
        symbol
    }
}
