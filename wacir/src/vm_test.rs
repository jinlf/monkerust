// src/vm_test.rs

use super::lexer::*;
use super::parser::*;
use super::ast::*;
use super::object::*;
use super::compiler::*;
use super::vm::*;
use std::any::Any;

fn parse(input: &str) -> Option<Program> {
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    return p.parse_program();
}

fn test_integer_object(expected: i64, actual: Object) {
    if let Object::Integer(Integer { value }) = actual {
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
                    Ok(_)=> {
                        let stack_elem = vm.stack_top();
                        test_expected_object(&tt.expected, stack_elem.unwrap());
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

fn test_expected_object(expectd: &Box<dyn Any>, actual: Object) {
    if let Some(v) = expectd.as_ref().downcast_ref::<isize>() {
        test_integer_object(*v as i64, actual);
    }
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        VmTestCase{
            input: "1",
            expected: Box::new(1 as isize),
        },
        VmTestCase{
            input:"2",
            expected: Box::new(2 as isize),
        },
        VmTestCase{
            input:"1 + 2",
            expected: Box::new(2 as isize),     //FIXME
        }
    ];

    run_vm_tests(tests);
}

