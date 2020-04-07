// src/vm_test.rs

use super::ast::*;
use super::compiler::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use super::vm::*;
use std::any::Any;

fn parse(input: &str) -> Option<Program> {
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    return p.parse_program();
}

fn test_integer_object(expected: i64, actual: Option<Object>) {
    if let Some(Object::Integer(Integer { value })) = actual {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        assert!(false, "object is not Integer. got={:?}", actual);
    }
}

struct VmTestCase<'a> {
    input: &'a str,
    expected: Box<dyn Any>,
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

fn test_expected_object(expectd: &Box<dyn Any>, actual: Option<Object>) {
    if let Some(v) = expectd.as_ref().downcast_ref::<i64>() {
        test_integer_object(*v as i64, actual);
    }
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        VmTestCase {
            input: "1",
            expected: Box::new(1 as i64),
        },
        VmTestCase {
            input: "2",
            expected: Box::new(2 as i64),
        },
        VmTestCase {
            input: "1 + 2",
            expected: Box::new(3 as i64),
        },
        VmTestCase {
            input: "1 - 2",
            expected: Box::new(-1 as i64),
        },
        VmTestCase {
            input: "1 * 2",
            expected: Box::new(2 as i64),
        },
        VmTestCase {
            input: "4 / 2",
            expected: Box::new(2 as i64),
        },
        VmTestCase {
            input: "50 / 2 * 2 + 10 - 5",
            expected: Box::new(55 as i64),
        },
        VmTestCase {
            input: "5 * (2 + 10)",
            expected: Box::new(60 as i64),
        },
        VmTestCase {
            input: "5 + 5 + 5 + 5 - 10",
            expected: Box::new(10 as i64),
        },
        VmTestCase {
            input: "2 * 2 * 2 * 2 * 2",
            expected: Box::new(32 as i64),
        },
        VmTestCase {
            input: "5 * 2 + 10",
            expected: Box::new(20 as i64),
        },
        VmTestCase {
            input: "5 + 2 * 10",
            expected: Box::new(25 as i64),
        },
        VmTestCase {
            input: "5 * (2 + 10)",
            expected: Box::new(60 as i64),
        },
    ];

    run_vm_tests(tests);
}
