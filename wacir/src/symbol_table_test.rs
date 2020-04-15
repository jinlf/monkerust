// src/symbol_table_test.rs

use super::symbol_table::*;
use std::cell::*;
use std::collections::*;
use std::rc::*;

#[test]
fn test_define() {
    let mut expected: HashMap<&str, Symbol> = HashMap::new();
    expected.insert(
        "a",
        Symbol {
            name: String::from("a"),
            scope: SymbolScope::GlobalScope,
            index: 0,
        },
    );
    expected.insert(
        "b",
        Symbol {
            name: String::from("b"),
            scope: SymbolScope::GlobalScope,
            index: 1,
        },
    );
    expected.insert(
        "c",
        Symbol {
            name: String::from("c"),
            scope: SymbolScope::LocalScope,
            index: 0,
        },
    );
    expected.insert(
        "d",
        Symbol {
            name: String::from("d"),
            scope: SymbolScope::LocalScope,
            index: 1,
        },
    );
    expected.insert(
        "e",
        Symbol {
            name: String::from("e"),
            scope: SymbolScope::LocalScope,
            index: 0,
        },
    );
    expected.insert(
        "f",
        Symbol {
            name: String::from("f"),
            scope: SymbolScope::LocalScope,
            index: 1,
        },
    );

    let global = Rc::new(RefCell::new(SymbolTable::new()));

    let a = global.borrow_mut().define("a");
    assert!(
        a == expected["a"],
        "expected a={:?}, got={:?}",
        expected["a"],
        a
    );
    let b = global.borrow_mut().define("b");
    assert!(
        b == expected["b"],
        "expected b={:?}, got={:?}",
        expected["b"],
        b
    );

    let first_local = Rc::new(RefCell::new(SymbolTable::new_enclosed_symbol_table(
        Rc::clone(&global),
    )));

    let c = first_local.borrow_mut().define("c");
    assert!(
        c == expected["c"],
        "expected c={:?}, got={:?}",
        expected["c"],
        c
    );
    let d = first_local.borrow_mut().define("d");
    assert!(
        d == expected["d"],
        "expected d={:?}, got={:?}",
        expected["d"],
        d
    );

    let second_local = Rc::new(RefCell::new(SymbolTable::new_enclosed_symbol_table(
        Rc::clone(&global),
    )));
    let e = second_local.borrow_mut().define("e");
    assert!(
        e == expected["e"],
        "expected e={:?}, got={:?}",
        expected["e"],
        e
    );
    let f = second_local.borrow_mut().define("f");
    assert!(
        f == expected["f"],
        "expected f={:?}, got={:?}",
        expected["f"],
        f
    );
}

#[test]
fn test_resolve_global() {
    let global = Rc::new(RefCell::new(SymbolTable::new()));
    global.borrow_mut().define("a");
    global.borrow_mut().define("b");

    let expected = vec![
        Symbol {
            name: String::from("a"),
            scope: SymbolScope::GlobalScope,
            index: 0,
        },
        Symbol {
            name: String::from("b"),
            scope: SymbolScope::GlobalScope,
            index: 1,
        },
    ];
    for sym in expected.iter() {
        if let Some(result) = global.borrow().resolve(&sym.name) {
            assert!(
                result == *sym,
                "expected {} to resolve to {:?}, got={:?}",
                sym.name,
                sym,
                result
            );
        } else {
            assert!(false, "name {} not resolvable", sym.name);
        }
    }
}

#[test]
fn test_resolve_local() {
    let global = Rc::new(RefCell::new(SymbolTable::new()));
    global.borrow_mut().define("a");
    global.borrow_mut().define("b");

    let local = Rc::new(RefCell::new(SymbolTable::new_enclosed_symbol_table(
        Rc::clone(&global),
    )));
    local.borrow_mut().define("c");
    local.borrow_mut().define("d");

    let expected = vec![
        Symbol {
            name: String::from("a"),
            scope: SymbolScope::GlobalScope,
            index: 0,
        },
        Symbol {
            name: String::from("b"),
            scope: SymbolScope::GlobalScope,
            index: 1,
        },
        Symbol {
            name: String::from("c"),
            scope: SymbolScope::LocalScope,
            index: 0,
        },
        Symbol {
            name: String::from("d"),
            scope: SymbolScope::LocalScope,
            index: 1,
        },
    ];

    for sym in expected {
        if let Some(result) = local.borrow().resolve(&sym.name) {
            assert!(
                result == sym,
                "expected {} to resolve to {:?}, got={:?}",
                sym.name,
                sym,
                result
            );
        } else {
            assert!(false, "name {} not resolvable", sym.name);
        }
    }
}

#[test]
fn test_resolve_nested_local() {
    let global = Rc::new(RefCell::new(SymbolTable::new()));
    global.borrow_mut().define("a");
    global.borrow_mut().define("b");

    let first_local = Rc::new(RefCell::new(SymbolTable::new_enclosed_symbol_table(
        Rc::clone(&global),
    )));
    first_local.borrow_mut().define("c");
    first_local.borrow_mut().define("d");

    let second_local = Rc::new(RefCell::new(SymbolTable::new_enclosed_symbol_table(
        Rc::clone(&first_local),
    )));
    second_local.borrow_mut().define("e");
    second_local.borrow_mut().define("f");

    let tests: Vec<(Rc<RefCell<SymbolTable>>, Vec<Symbol>)> = vec![
        (
            first_local,
            vec![
                Symbol {
                    name: String::from("a"),
                    scope: SymbolScope::GlobalScope,
                    index: 0,
                },
                Symbol {
                    name: String::from("b"),
                    scope: SymbolScope::GlobalScope,
                    index: 1,
                },
                Symbol {
                    name: String::from("c"),
                    scope: SymbolScope::LocalScope,
                    index: 0,
                },
                Symbol {
                    name: String::from("d"),
                    scope: SymbolScope::LocalScope,
                    index: 1,
                },
            ],
        ),
        (
            second_local,
            vec![
                Symbol {
                    name: String::from("a"),
                    scope: SymbolScope::GlobalScope,
                    index: 0,
                },
                Symbol {
                    name: String::from("b"),
                    scope: SymbolScope::GlobalScope,
                    index: 1,
                },
                Symbol {
                    name: String::from("e"),
                    scope: SymbolScope::LocalScope,
                    index: 0,
                },
                Symbol {
                    name: String::from("f"),
                    scope: SymbolScope::LocalScope,
                    index: 1,
                },
            ],
        ),
    ];

    for tt in tests.iter() {
        for sym in tt.1.iter() {
            if let Some(result) = tt.0.borrow().resolve(&sym.name) {
                assert!(
                    result == *sym,
                    "expected {} to resolve to {:?}, got={:?}",
                    sym.name,
                    sym,
                    result
                );
            } else {
                assert!(false, "name {} not resolvable", sym.name);
            }
        }
    }
}
