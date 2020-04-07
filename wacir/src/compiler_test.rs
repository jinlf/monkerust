// src/compiler_test.rs

use super::ast::*;
use super::code::*;
use super::compiler::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use std::any::Any;

struct CompilerTestCase<'a> {
    input: &'a str,
    expected_constants: Vec<Box<dyn Any>>,
    expected_instructions: Vec<Instructions>,
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        CompilerTestCase {
            input: "1 + 2",
            expected_constants: vec![Box::new(1 as i32), Box::new(2 as i32)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpAdd, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1; 2",
            expected_constants: vec![Box::new(1 as i32), Box::new(2 as i32)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 - 2",
            expected_constants: vec![Box::new(1 as i32), Box::new(2 as i32)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpSub, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 * 2",
            expected_constants: vec![Box::new(1 as i32), Box::new(2 as i32)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpMul, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "2 / 1",
            expected_constants: vec![Box::new(2 as i32), Box::new(1 as i32)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpDiv, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];
    run_compiler_tests(tests);
}

fn run_compiler_tests(tests: Vec<CompilerTestCase>) {
    for tt in tests.iter() {
        let program = parse(tt.input);

        let mut compiler = Compiler::new();

        match compiler.compile(Node::Program(program.unwrap())) {
            Ok(_) => {
                let bytecode = compiler.bytecode();
                test_instructions(&tt.expected_instructions, &bytecode.instuctions);
                test_constants(&tt.expected_constants, &bytecode.constants);
            }
            Err(e) => assert!(false, "compiler error: {}", e),
        }
    }
}

fn parse(input: &str) -> Option<Program> {
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    return p.parse_program();
}

fn test_instructions(expected: &Vec<Instructions>, actual: &Instructions) {
    let concatted = concat_instructions(expected);

    assert!(
        actual.0.len() == concatted.0.len(),
        "wrong instructions length. \nwant={:?}\ngot={:?}",
        concatted.string(),
        actual.string()
    );
    for (i, ins) in concatted.0.iter().enumerate() {
        assert!(
            actual.0[i] == *ins,
            "wrong instruction at {}.\nwant={:?}\ngot={:?}",
            i,
            concatted,
            actual,
        );
    }
}

fn concat_instructions(s: &Vec<Instructions>) -> Instructions {
    let mut out = Instructions::new();
    for ins in s.iter() {
        out.0.extend_from_slice(&ins.0);
    }
    out
}

fn test_constants(expected: &Vec<Box<dyn Any>>, actual: &Vec<Object>) {
    assert!(
        expected.len() == actual.len(),
        "wrong number of constants. got={:?}, want={:?}",
        actual.len(),
        expected.len(),
    );

    for (i, constant) in expected.iter().enumerate() {
        if let Some(iv) = (*constant).downcast_ref::<i32>() {
            test_integer_object(*iv as i64, actual[i].clone());
        }
    }
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

#[test]
fn test_boolean_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "true",
            expected_constants: Vec::new(),
            expected_instructions: vec![
                make(Opcode::OpTrue, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "false",
            expected_constants: Vec::new(),
            expected_instructions: vec![
                make(Opcode::OpFalse, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}
