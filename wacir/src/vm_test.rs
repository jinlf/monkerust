// src/vm_test.rs

use super::ast::*;
use super::compiler::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use super::vm::*;

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
