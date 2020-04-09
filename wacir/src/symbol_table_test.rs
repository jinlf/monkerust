// src/symbol_table_test.rs

use super::symbol_table::*;
use std::collections::*;

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

    let mut global = SymbolTable::new();
    let a = global.define("a");
    assert!(
        a == expected["a"],
        "expected a={:?}, got={:?}",
        expected["a"],
        a
    );
    let b = global.define("b");
    assert!(
        b == expected["b"],
        "expected a={:?}, got={:?}",
        expected["b"],
        b
    );
}

#[test]
fn test_resolve_global() {
    let mut global = SymbolTable::new();
    global.define("a");
    global.define("b");

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
        if let Some(result) = global.resolve(&sym.name) {
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
