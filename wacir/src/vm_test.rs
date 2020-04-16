// src/vm_test.rs

use super::ast::*;
use super::compiler::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use super::vm::*;
use std::collections::*;

fn parse(input: &str) -> Option<Program> {
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    return p.parse_program();
}

fn test_integer_object(expected: i64, actual: Option<Object>) -> Result<String, String> {
    if let Some(Object::Integer(Integer { value })) = actual {
        if value != expected {
            return Err(format!(
                "object has wrong value. got={}, want={}",
                value, expected
            ));
        }
    } else {
        return Err(format!("object is not Integer. got={:?}", actual));
    }
    Ok(String::new())
}

struct VmTestCase<'a> {
    input: &'a str,
    expected: Object,
}

fn run_vm_tests(tests: Vec<VmTestCase>) {
    for tt in tests.iter() {
        let program = parse(tt.input);
        let mut comp = Compiler::new();
        match comp.compile(Node::Program(program.unwrap())) {
            Ok(_) => {
                let mut vm = Vm::new(comp.bytecode());
                match vm.run() {
                    Ok(_) => {
                        let stack_elem = vm.last_popped_stack_elem();
                        test_expected_object(&tt.expected, stack_elem);
                    }
                    Err(err) => {
                        assert!(false, "vm error: {}", err);
                    }
                }
            }
            Err(err) => {
                assert!(false, "compilre error: {}", err);
            }
        }
    }
}

fn test_expected_object(expected: &Object, actual: Option<Object>) {
    if let Object::Integer(Integer { value }) = expected {
        match test_integer_object(*value, actual) {
            Ok(_) => {}
            Err(err) => {
                assert!(false, "test_integer_object failed: {}", err);
            }
        }
    } else if let Object::Boolean(Boolean { value }) = expected {
        match test_boolean_object(*value, actual) {
            Ok(_) => {}
            Err(err) => {
                assert!(false, "test_boolean_object failed: {}", err);
            }
        }
    } else if let Object::Null(_) = expected {
        if let Some(Object::Null(_)) = actual {
        } else {
            assert!(false, "object is not Null: {:?}", actual);
        }
    } else if let Object::StringObj(StringObj { value }) = expected {
        match test_string_object(value, actual) {
            Ok(_) => {}
            Err(err) => {
                assert!(false, "test_string_object failed: {}", err);
            }
        }
    } else if let Object::Array(Array { elements }) = expected {
        let expected_elements = elements;
        if let Some(Object::Array(Array { elements })) = actual {
            let actual_elements = elements;
            assert!(
                expected_elements.len() == actual_elements.len(),
                "wrong num of elements. want={}, got={}",
                expected_elements.len(),
                actual_elements.len()
            );

            for (i, expected_elem) in expected_elements.iter().enumerate() {
                if let Object::Integer(Integer { value }) = expected_elem {
                    match test_integer_object(*value, Some(actual_elements[i].clone())) {
                        Ok(_) => {}
                        Err(err) => {
                            assert!(false, "test_integer_object failed: {}", err);
                        }
                    }
                } else {
                    assert!(false, "error");
                }
            }
        } else {
            assert!(false, "object not Array: {:?}", actual);
        }
    } else if let Object::Hash(Hash { pairs }) = expected {
        let expected_pairs = pairs;
        if let Some(Object::Hash(Hash { pairs })) = actual {
            let actual_pairs = pairs;
            assert!(
                expected_pairs.len() == actual_pairs.len(),
                "hash has wrong number of pairs. want={}, got={}",
                expected_pairs.len(),
                actual_pairs.len()
            );
            for (expected_key, expected_value) in expected_pairs.iter() {
                if let HashKey::Integer(Integer { value: _ }) = *expected_key {
                    if actual_pairs.contains_key(expected_key) {
                        if let Object::Integer(Integer { value }) = *expected_value {
                            match test_integer_object(
                                value,
                                Some(actual_pairs[expected_key].clone()),
                            ) {
                                Err(err) => {
                                    assert!(false, "test_integer_object failed: {}", err);
                                }
                                _ => {}
                            }
                        } else {
                            assert!(false, "error");
                        }
                    } else {
                        assert!(false, "no pair for given key in pairs");
                    }
                } else {
                    assert!(false, "error");
                }
            }
        } else if let Object::ErrorObj(ErrorObj { message }) = expected {
            let expected_message = message;
            if let Some(Object::ErrorObj(ErrorObj { message })) = actual {
                assert!(
                    expected_message == &message,
                    "wrong error message. expected={:?}, got={:?}",
                    expected_message,
                    message
                );
            } else {
                assert!(false, "object is not Error: {:?}", actual);
            }
        } else {
            assert!(false, "object is not Hash. got={:?}", actual);
        }
    }
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        VmTestCase {
            input: "1",
            expected: Object::Integer(Integer { value: 1 }),
        },
        VmTestCase {
            input: "2",
            expected: Object::Integer(Integer { value: 2 }),
        },
        VmTestCase {
            input: "1 + 2",
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: "1 - 2",
            expected: Object::Integer(Integer { value: -1 }),
        },
        VmTestCase {
            input: "1 * 2",
            expected: Object::Integer(Integer { value: 2 }),
        },
        VmTestCase {
            input: "4 / 2",
            expected: Object::Integer(Integer { value: 2 }),
        },
        VmTestCase {
            input: "50 / 2 * 2 + 10 - 5",
            expected: Object::Integer(Integer { value: 55 }),
        },
        VmTestCase {
            input: "5 * (2 + 10)",
            expected: Object::Integer(Integer { value: 60 }),
        },
        VmTestCase {
            input: "5 + 5 + 5 + 5 - 10",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "2 * 2 * 2 * 2 * 2",
            expected: Object::Integer(Integer { value: 32 }),
        },
        VmTestCase {
            input: "5 * 2 + 10",
            expected: Object::Integer(Integer { value: 20 }),
        },
        VmTestCase {
            input: "5 + 2 * 10",
            expected: Object::Integer(Integer { value: 25 }),
        },
        VmTestCase {
            input: "5 * (2 + 10)",
            expected: Object::Integer(Integer { value: 60 }),
        },
        VmTestCase {
            input: "-5",
            expected: Object::Integer(Integer { value: -5 }),
        },
        VmTestCase {
            input: "-10",
            expected: Object::Integer(Integer { value: -10 }),
        },
        VmTestCase {
            input: "-50 + 100 + -50",
            expected: Object::Integer(Integer { value: 0 }),
        },
        VmTestCase {
            input: "(5 + 10 * 2 + 15 / 3) * 2 + -10)",
            expected: Object::Integer(Integer { value: 50 }),
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_boolean_expressions() {
    let tests = vec![
        VmTestCase {
            input: "true",
            expected: TRUE,
        },
        VmTestCase {
            input: "false",
            expected: FALSE,
        },
        VmTestCase {
            input: "1 < 2",
            expected: TRUE,
        },
        VmTestCase {
            input: "1 > 2",
            expected: FALSE,
        },
        VmTestCase {
            input: "1 < 1",
            expected: FALSE,
        },
        VmTestCase {
            input: "1 > 1",
            expected: FALSE,
        },
        VmTestCase {
            input: "1 == 1",
            expected: TRUE,
        },
        VmTestCase {
            input: "1 != 1",
            expected: FALSE,
        },
        VmTestCase {
            input: "1 == 2",
            expected: FALSE,
        },
        VmTestCase {
            input: "1 != 2",
            expected: TRUE,
        },
        VmTestCase {
            input: "true == true",
            expected: TRUE,
        },
        VmTestCase {
            input: "false == false",
            expected: TRUE,
        },
        VmTestCase {
            input: "true == false",
            expected: FALSE,
        },
        VmTestCase {
            input: "true != false",
            expected: TRUE,
        },
        VmTestCase {
            input: "false != true",
            expected: TRUE,
        },
        VmTestCase {
            input: "(1 < 2) == true",
            expected: TRUE,
        },
        VmTestCase {
            input: "(1 < 2) == false",
            expected: FALSE,
        },
        VmTestCase {
            input: "(1 > 2) == true",
            expected: FALSE,
        },
        VmTestCase {
            input: "(1 > 2) == false",
            expected: TRUE,
        },
        VmTestCase {
            input: "!true",
            expected: FALSE,
        },
        VmTestCase {
            input: "!false",
            expected: TRUE,
        },
        VmTestCase {
            input: "!5",
            expected: FALSE,
        },
        VmTestCase {
            input: "!!true",
            expected: TRUE,
        },
        VmTestCase {
            input: "!!false",
            expected: FALSE,
        },
        VmTestCase {
            input: "!!5",
            expected: TRUE,
        },
        VmTestCase {
            input: "!(if (false) { 5; })",
            expected: TRUE,
        },
    ];

    run_vm_tests(tests);
}

fn test_boolean_object(expected: bool, actual: Option<Object>) -> Result<String, String> {
    if let Some(Object::Boolean(Boolean { value })) = actual {
        if value != expected {
            return Err(format!(
                "object has wrong value. got={}, want={}",
                value, expected
            ));
        }
    } else {
        return Err(format!("object is not Boolean. got={:?}", actual));
    }
    Ok(String::new())
}

#[test]
fn test_conditionals() {
    let tests = vec![
        VmTestCase {
            input: "if (true) { 10 }",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "if (true) { 10 } else { 20 }",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "if (false) { 10 } else { 20 }",
            expected: Object::Integer(Integer { value: 20 }),
        },
        VmTestCase {
            input: "if (1) { 10 }",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "if (1 < 2) { 10 }",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "if (1 < 2) { 10 } else { 20 }",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "if (1 > 2) { 10 } else { 20 }",
            expected: Object::Integer(Integer { value: 20 }),
        },
        VmTestCase {
            input: "if (1 > 2) { 10 }",
            expected: NULL,
        },
        VmTestCase {
            input: "if (false) { 10 }",
            expected: NULL,
        },
        VmTestCase {
            input: "if ((if (false) { 10 })) { 10 } else { 20 }",
            expected: Object::Integer(Integer { value: 20 }),
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_global_let_statements() {
    let tests = vec![
        VmTestCase {
            input: "let one = 1; one",
            expected: Object::Integer(Integer { value: 1 }),
        },
        VmTestCase {
            input: "let one = 1; let two = 2; one + two",
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: "let one = 1; let two = one + one; one + two",
            expected: Object::Integer(Integer { value: 3 }),
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_string_expressions() {
    let tests = vec![
        VmTestCase {
            input: "\"monkey\"",
            expected: Object::StringObj(StringObj {
                value: String::from("monkey"),
            }),
        },
        VmTestCase {
            input: "\"mon\" + \"key\"",
            expected: Object::StringObj(StringObj {
                value: String::from("monkey"),
            }),
        },
        VmTestCase {
            input: "\"mon\" + \"key\" + \"banana\"",
            expected: Object::StringObj(StringObj {
                value: String::from("monkeybanana"),
            }),
        },
    ];

    run_vm_tests(tests);
}

fn test_string_object(expected: &str, actual: Option<Object>) -> Result<String, String> {
    if let Some(Object::StringObj(StringObj { value })) = actual {
        if value != expected {
            return Err(format!(
                "object has wrong value. got={}, want={}",
                value, expected
            ));
        }
    } else {
        return Err(format!("object is not String. got={:?}", actual));
    }
    Ok(String::new())
}

#[test]
fn test_array_literals() {
    let tests = vec![
        VmTestCase {
            input: "[]",
            expected: Object::Array(Array {
                elements: Vec::new(),
            }),
        },
        VmTestCase {
            input: "[1, 2, 3]",
            expected: Object::Array(Array {
                elements: vec![
                    Object::Integer(Integer { value: 1 }),
                    Object::Integer(Integer { value: 2 }),
                    Object::Integer(Integer { value: 3 }),
                ],
            }),
        },
        VmTestCase {
            input: "[1 + 2, 3 * 4, 5 + 6]",
            expected: Object::Array(Array {
                elements: vec![
                    Object::Integer(Integer { value: 3 }),
                    Object::Integer(Integer { value: 12 }),
                    Object::Integer(Integer { value: 11 }),
                ],
            }),
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_hash_literals() {
    let mut pairs2: HashMap<HashKey, Object> = HashMap::new();
    pairs2.insert(
        HashKey::Integer(Integer { value: 1 }),
        Object::Integer(Integer { value: 2 }),
    );
    pairs2.insert(
        HashKey::Integer(Integer { value: 2 }),
        Object::Integer(Integer { value: 3 }),
    );

    let mut pairs3: HashMap<HashKey, Object> = HashMap::new();
    pairs3.insert(
        HashKey::Integer(Integer { value: 2 }),
        Object::Integer(Integer { value: 4 }),
    );
    pairs3.insert(
        HashKey::Integer(Integer { value: 6 }),
        Object::Integer(Integer { value: 16 }),
    );

    let tests = vec![
        // VmTestCase {
        //     input: "{}",
        //     expected: Object::Hash(Hash {
        //         pairs: HashMap::new(),
        //     }),
        // },
        VmTestCase {
            input: "{1: 2, 2: 3}",
            expected: Object::Hash(Hash { pairs: pairs2 }),
        },
        // VmTestCase {
        //     input: "{1 + 1: 2 * 2, 3 + 3: 4 * 4}",
        //     expected: Object::Hash(Hash { pairs: pairs3 }),
        // },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_index_expressions() {
    let tests = vec![
        VmTestCase {
            input: "[1, 2, 3][1]",
            expected: Object::Integer(Integer { value: 2 }),
        },
        VmTestCase {
            input: "[1, 2, 3][0 + 2]",
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: "[[1, 1, 1]][0][0]",
            expected: Object::Integer(Integer { value: 1 }),
        },
        VmTestCase {
            input: "[][0]",
            expected: NULL,
        },
        VmTestCase {
            input: "[1, 2, 3][99]",
            expected: NULL,
        },
        VmTestCase {
            input: "[1][-1]",
            expected: NULL,
        },
        VmTestCase {
            input: "{1: 1, 2: 2}[1]",
            expected: Object::Integer(Integer { value: 1 }),
        },
        VmTestCase {
            input: "{1: 1, 2:2}[2]",
            expected: Object::Integer(Integer { value: 2 }),
        },
        VmTestCase {
            input: "{1: 1}[0]",
            expected: NULL,
        },
        VmTestCase {
            input: "{}[0]",
            expected: NULL,
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_calling_functions_without_arguments() {
    let tests = vec![
        VmTestCase {
            input: "let fivePlusTen = fn() { 5 + 10; };
            fivePlusTen();
            ",
            expected: Object::Integer(Integer { value: 15 }),
        },
        VmTestCase {
            input: "let one = fn() { 1; };
            let two = fn() { 2; };
            one() + two();
            ",
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: "let a = fn() { 1 };
            let b = fn() { a() + 1 };
            let c = fn() { b() + 1 };
            c();
            ",
            expected: Object::Integer(Integer { value: 3 }),
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_functions_with_return_statement() {
    let tests = vec![
        VmTestCase {
            input: "let earlyExit = fn() { return 99; 100; };
            earlyExit();
            ",
            expected: Object::Integer(Integer { value: 99 }),
        },
        VmTestCase {
            input: "let earlyExit = fn() { return 99; return 100; };
            earlyExit();
            ",
            expected: Object::Integer(Integer { value: 99 }),
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_functions_without_return_value() {
    let tests = vec![
        VmTestCase {
            input: "let noReturn = fn() { };
            noReturn();
            ",
            expected: NULL,
        },
        VmTestCase {
            input: "let noReturn = fn() { };
            let noReturnTwo = fn() { noReturn(); };
            noReturn();
            noReturnTwo();
            ",
            expected: NULL,
        },
    ];

    run_vm_tests(tests);
}

#[test]
fn test_first_class_functions() {
    let tests = vec![VmTestCase {
        input: "
            let returnsOne = fn() { 1; };
            let returnsOneReturner = fn() { returnsOne; };
            returnsOneReturner()();
            ",
        expected: Object::Integer(Integer { value: 1 }),
    }];
    run_vm_tests(tests);
}

#[test]
fn test_calling_functions_with_bindings() {
    let tests = vec![
        VmTestCase {
            input: "let one = fn() { let one = 1; one };
            one();
            ",
            expected: Object::Integer(Integer { value: 1 }),
        },
        VmTestCase {
            input: "let oneAndTwo = fn() { let one = 1; let two = 2; one + two; };
            oneAndTwo();
            ",
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: "let oneAndTwo = fn() { let one = 1; let two = 2; one + two; };
            let threeAndFour = fn () { let three = 3; let four = 4; three + four; };
            oneAndTwo() + threeAndFour();
            ",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "let firstFoobar = fn() { let foobar = 50; foobar; };
            let secondFoobar = fn() { let foobar = 100; foobar; };
            firstFoobar() + secondFoobar();
                ",
            expected: Object::Integer(Integer { value: 150 }),
        },
        VmTestCase {
            input: "let globalSeed = 50;
            let minusOne = fn() { 
                let num = 1;
                globalSeed - num;
            };
            let minusTwo = fn() {
                let num = 2;
                globalSeed - num;                
            }
            minusOne() + minusTwo();            
            ",
            expected: Object::Integer(Integer { value: 97 }),
        },
    ];
    run_vm_tests(tests);
}

#[test]
fn test_calling_functions_with_arguments_and_bindings() {
    let tests = vec![
        VmTestCase {
            input: "let identity = fn(a) { a; };
            identity(4);",
            expected: Object::Integer(Integer { value: 4 }),
        },
        VmTestCase {
            input: "let sum = fn(a, b) { a + b; };
            sum(1, 2);",
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: "let sum = fn(a, b) {
                let c = a + b;
                c;
            };
            sum(1, 2);",
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: "let sum = fn(a, b) {
                let c = a + b;
                c;
            };
            sum(1, 2) + sum(3, 4);",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "let sum = fn(a, b) {
                let c = a + b;
                c;
            };
            let outer = fn() {
                sum(1, 2) + sum(3, 4);
            };
            outer();
            ",
            expected: Object::Integer(Integer { value: 10 }),
        },
        VmTestCase {
            input: "let globalNum = 10;
            
            let sum = fn(a, b) {
                let c = a + b;
                c + globalNum;
            };
            
            let outer = fn() {
                sum(1, 2) + sum(3, 4) + globalNum;
            };
            
            outer() + globalNum;
            ",
            expected: Object::Integer(Integer { value: 50 }),
        },
    ];
    run_vm_tests(tests);
}

#[test]
fn test_calling_functions_with_wrong_arguments() {
    let tests = vec![
        VmTestCase {
            input: "fn() { 1; } (1);",
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("wrong number of arguments: want=0, got=1"),
            }),
        },
        VmTestCase {
            input: "fn(a) { a; }();",
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("wrong number of arguments: want=1, got=0"),
            }),
        },
        VmTestCase {
            input: "fn(a, b) { a + b; }(1);",
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("wrong number of arguments: want=2, got=1"),
            }),
        },
    ];

    for tt in tests.iter() {
        let program = parse(tt.input);

        let mut comp = Compiler::new();
        match comp.compile(Node::Program(program.unwrap())) {
            Err(err) => {
                assert!(false, "compiler error: {}", err);
            }
            _ => {}
        }

        let mut vm = Vm::new(comp.bytecode());
        match vm.run() {
            Err(err) => {
                assert!(
                    Object::ErrorObj(ErrorObj {
                        message: err.clone()
                    }) == tt.expected,
                    "wrong Vm error: want={:?}, got={}",
                    tt.expected,
                    err
                );
            }
            _ => {
                assert!(false, "expected VM error but resulted in none.");
            }
        }
    }
}

#[test]
fn test_builtin_functions() {
    let tests = vec![
        VmTestCase {
            input: r#"len("")"#,
            expected: Object::Integer(Integer { value: 0 }),
        },
        VmTestCase {
            input: r#"len("four")"#,
            expected: Object::Integer(Integer { value: 4 }),
        },
        VmTestCase {
            input: r#"len("hello world")"#,
            expected: Object::Integer(Integer { value: 11 }),
        },
        VmTestCase {
            input: r#"len(1)"#,
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("argument to `len` not supported, got INTEGER"),
            }),
        },
        VmTestCase {
            input: r#"len("one", "two")"#,
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("wrong number of arguments. got=2, want=1"),
            }),
        },
        VmTestCase {
            input: r#"len([1, 2, 3])"#,
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: r#"len([])"#,
            expected: Object::Integer(Integer { value: 0 }),
        },
        VmTestCase {
            input: r#"puts("hello", "wrold!")"#,
            expected: NULL,
        },
        VmTestCase {
            input: r#"first([1, 2, 3])"#,
            expected: Object::Integer(Integer { value: 1 }),
        },
        VmTestCase {
            input: r#"first([])"#,
            expected: NULL,
        },
        VmTestCase {
            input: r#"first(1)"#,
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("argument to `first` must be ARRAY, got INTEGER"),
            }),
        },
        VmTestCase {
            input: r#"last([1, 2, 3])"#,
            expected: Object::Integer(Integer { value: 3 }),
        },
        VmTestCase {
            input: r#"last([])"#,
            expected: NULL,
        },
        VmTestCase {
            input: r#"last(1)"#,
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("argument to `last` must be ARRAY, got INTEGER"),
            }),
        },
        VmTestCase {
            input: r#"rest([1, 2, 3])"#,
            expected: Object::Array(Array {
                elements: vec![
                    Object::Integer(Integer { value: 2 }),
                    Object::Integer(Integer { value: 3 }),
                ],
            }),
        },
        VmTestCase {
            input: r#"rest([])"#,
            expected: NULL,
        },
        VmTestCase {
            input: r#"push([], 1)"#,
            expected: Object::Array(Array {
                elements: vec![Object::Integer(Integer { value: 1 })],
            }),
        },
        VmTestCase {
            input: r#"push(1, 1)"#,
            expected: Object::ErrorObj(ErrorObj {
                message: String::from("argument to `push` must be ARRAY, got INTEGER"),
            }),
        },
    ];
    run_vm_tests(tests);
}
