// src/compiler_test.rs

use super::ast::*;
use super::code::*;
use super::compiler::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use std::any::Any;
use std::cell::*;
use std::rc::*;

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
        CompilerTestCase {
            input: "-1",
            expected_constants: vec![Box::new(1 as i32)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpMinus, &Vec::new()),
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
                test_constants(&tt.expected_constants, Rc::clone(&bytecode.constants));
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

fn test_constants(expected: &Vec<Box<dyn Any>>, actual: Rc<RefCell<Vec<Object>>>) {
    assert!(
        expected.len() == actual.borrow().len(),
        "wrong number of constants. got={:?}, want={:?}",
        actual.borrow().len(),
        expected.len(),
    );

    for (i, constant) in expected.iter().enumerate() {
        if let Some(iv) = (*constant).downcast_ref::<i32>() {
            test_integer_object(*iv as i64, actual.borrow()[i].clone());
        } else if let Some(sv) = (*constant).downcast_ref::<&str>() {
            test_string_object(sv, actual.borrow()[i].clone());
        } else if let Some(expected_instructions) = (*constant).downcast_ref::<Vec<Instructions>>()
        {
            if let Object::CompiledFunction(CompiledFunction {
                instructions,
                num_locals: _,
                num_parameters: _,
            }) = actual.borrow()[i].clone()
            {
                test_instructions(expected_instructions, &instructions);
            } else {
                assert!(
                    false,
                    "constant {} - not a function: {:?}",
                    i,
                    actual.borrow()[i]
                )
            }
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
        CompilerTestCase {
            input: "1 > 2",
            expected_constants: vec![Box::new(1 as i64), Box::new(2 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpGreaterThan, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 < 2",
            expected_constants: vec![Box::new(2 as i64), Box::new(1 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpGreaterThan, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 == 2",
            expected_constants: vec![Box::new(1 as i64), Box::new(2 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpEqual, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 != 2",
            expected_constants: vec![Box::new(1 as i64), Box::new(2 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpNotEqual, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "true == false",
            expected_constants: Vec::new(),
            expected_instructions: vec![
                make(Opcode::OpTrue, &Vec::new()),
                make(Opcode::OpFalse, &Vec::new()),
                make(Opcode::OpEqual, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "true != false",
            expected_constants: Vec::new(),
            expected_instructions: vec![
                make(Opcode::OpTrue, &Vec::new()),
                make(Opcode::OpFalse, &Vec::new()),
                make(Opcode::OpNotEqual, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "!true",
            expected_constants: Vec::new(),
            expected_instructions: vec![
                make(Opcode::OpTrue, &Vec::new()),
                make(Opcode::OpBang, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}

#[test]
fn test_conditionals() {
    let tests = vec![
        CompilerTestCase {
            input: "if (true) { 10 }; 3333;",
            expected_constants: vec![Box::new(10 as i32), Box::new(3333 as i32)],
            expected_instructions: vec![
                make(Opcode::OpTrue, &Vec::new()),
                make(Opcode::OpJumpNotTruthy, &vec![10]),
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpJump, &vec![11]),
                make(Opcode::OpNull, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "if (true) { 10 } else { 20 }; 3333;",
            expected_constants: vec![
                Box::new(10 as i32),
                Box::new(20 as i32),
                Box::new(3333 as i32),
            ],
            expected_instructions: vec![
                make(Opcode::OpTrue, &Vec::new()),
                make(Opcode::OpJumpNotTruthy, &vec![10]),
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpJump, &vec![13]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}

#[test]
fn test_global_let_statements() {
    let tests = vec![
        CompilerTestCase {
            input: "let one = 1;
            let two = 2;",
            expected_constants: vec![Box::new(1 as i64), Box::new(2 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpSetGlobal, &vec![1]),
            ],
        },
        CompilerTestCase {
            input: "let one = 1;
            one;",
            expected_constants: vec![Box::new(1 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpGetGlobal, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "let one = 1;
            let two = one;
            two;",
            expected_constants: vec![Box::new(1 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpGetGlobal, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![1]),
                make(Opcode::OpGetGlobal, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}

#[test]
fn test_string_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "\"monkey\"",
            expected_constants: vec![Box::new("monkey")],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "\"mon\" + \"key\"",
            expected_constants: vec![Box::new("mon"), Box::new("key")],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpAdd, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}

fn test_string_object(expected: &str, actual: Object) {
    if let Object::StringObj(StringObj { value }) = actual {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        assert!(false, "object is not String. got={:?}", actual);
    }
}

#[test]
fn test_array_literal() {
    let tests = vec![
        CompilerTestCase {
            input: "[]",
            expected_constants: Vec::new(),
            expected_instructions: vec![
                make(Opcode::OpArray, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "[1, 2, 3]",
            expected_constants: vec![Box::new(1 as i64), Box::new(2 as i64), Box::new(3 as i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpArray, &vec![3]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "[1 + 2, 3 - 4, 5 * 6]",
            expected_constants: vec![
                Box::new(1 as i64),
                Box::new(2 as i64),
                Box::new(3 as i64),
                Box::new(4 as i64),
                Box::new(5 as i64),
                Box::new(6 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpAdd, &Vec::new()),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpConstant, &vec![3]),
                make(Opcode::OpSub, &Vec::new()),
                make(Opcode::OpConstant, &vec![4]),
                make(Opcode::OpConstant, &vec![5]),
                make(Opcode::OpMul, &Vec::new()),
                make(Opcode::OpArray, &vec![3]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}

#[test]
fn test_hash_literals() {
    let tests = vec![
        CompilerTestCase {
            input: "{}",
            expected_constants: Vec::new(),
            expected_instructions: vec![
                make(Opcode::OpHash, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "{1: 2, 3: 4, 5: 6}",
            expected_constants: vec![
                Box::new(1 as i64),
                Box::new(2 as i64),
                Box::new(3 as i64),
                Box::new(4 as i64),
                Box::new(5 as i64),
                Box::new(6 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpConstant, &vec![3]),
                make(Opcode::OpConstant, &vec![4]),
                make(Opcode::OpConstant, &vec![5]),
                make(Opcode::OpHash, &vec![6]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "{1: 2 + 3, 4: 5 * 6}",
            expected_constants: vec![
                Box::new(1 as i64),
                Box::new(2 as i64),
                Box::new(3 as i64),
                Box::new(4 as i64),
                Box::new(5 as i64),
                Box::new(6 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpAdd, &Vec::new()),
                make(Opcode::OpConstant, &vec![3]),
                make(Opcode::OpConstant, &vec![4]),
                make(Opcode::OpConstant, &vec![5]),
                make(Opcode::OpMul, &Vec::new()),
                make(Opcode::OpHash, &vec![4]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];
    run_compiler_tests(tests);
}

#[test]
fn test_index_expressions() {
    let tests = vec![
        CompilerTestCase {
            input: "[1, 2, 3][1 + 1]",
            expected_constants: vec![
                Box::new(1 as i64),
                Box::new(2 as i64),
                Box::new(3 as i64),
                Box::new(1 as i64),
                Box::new(1 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpArray, &vec![3]),
                make(Opcode::OpConstant, &vec![3]),
                make(Opcode::OpConstant, &vec![4]),
                make(Opcode::OpAdd, &Vec::new()),
                make(Opcode::OpIndex, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "{1: 2}[2 - 1]",
            expected_constants: vec![
                Box::new(1 as i64),
                Box::new(2 as i64),
                Box::new(2 as i64),
                Box::new(1 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpHash, &vec![2]),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpConstant, &vec![3]),
                make(Opcode::OpSub, &Vec::new()),
                make(Opcode::OpIndex, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}

#[test]
fn test_functions() {
    let tests = vec![
        CompilerTestCase {
            input: "fn() { return 5 + 10 }",
            expected_constants: vec![
                Box::new(5 as i64),
                Box::new(10 as i64),
                Box::new(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpConstant, &vec![1]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() { 1; 2 }",
            expected_constants: vec![
                Box::new(1 as i64),
                Box::new(2 as i64),
                Box::new(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpPop, &Vec::new()),
                    make(Opcode::OpConstant, &vec![1]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() { }",
            expected_constants: vec![Box::new(vec![make(Opcode::OpReturn, &Vec::new())])],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];

    run_compiler_tests(tests);
}

impl PartialEq for super::symbol_table::SymbolTable {
    fn eq(&self, other: &Self) -> bool {
        let addr = self as *const Self as usize;
        let other_addr = other as *const Self as usize;
        addr == other_addr
    }
}

#[test]
fn test_compiler_scopes() {
    let mut compiler = Compiler::new();
    assert!(
        compiler.scope_index == 0,
        "scope_index wrong. got={}, want={}",
        compiler.scope_index,
        0
    );

    let global_symbol_table = Rc::clone(&compiler.symbol_table);

    compiler.emit(Opcode::OpMul, Vec::new());

    compiler.enter_scope();
    assert!(
        compiler.scope_index == 1,
        "scope_index wrong. got={}, want={}",
        compiler.scope_index,
        1
    );

    compiler.emit(Opcode::OpSub, Vec::new());

    assert!(
        compiler.scopes[compiler.scope_index].instructions.0.len() == 1,
        "instructions length wrong. got={}",
        compiler.scopes[compiler.scope_index].instructions.0.len()
    );

    let mut last = compiler.scopes[compiler.scope_index]
        .last_instruction
        .clone()
        .expect("error");
    assert!(
        last.opcode == Opcode::OpSub,
        "last_instruction.opcode wrong. got={:?}, want={:?}",
        last.opcode,
        Opcode::OpSub
    );

    {
        let outer = &compiler.symbol_table.borrow().outer;
        assert!(
            outer.is_some() && (outer.as_ref().unwrap() == &global_symbol_table),
            "compiler did not enclose symble_table"
        );
    }

    compiler.leave_scope();
    assert!(
        compiler.scope_index == 0,
        "scope_index wrong. got={}, want={}",
        compiler.scope_index,
        0
    );

    assert!(
        compiler.symbol_table == global_symbol_table,
        "compiler did not restore global symbol table"
    );

    assert!(
        compiler.symbol_table.borrow().outer.is_none(),
        "compiler modified global symbol table incorrectly"
    );

    compiler.emit(Opcode::OpAdd, Vec::new());

    assert!(
        compiler.scopes[compiler.scope_index].instructions.0.len() == 2,
        "instructions length wrong. got={}",
        compiler.scopes[compiler.scope_index].instructions.0.len()
    );

    last = compiler.scopes[compiler.scope_index]
        .last_instruction
        .clone()
        .expect("error");
    assert!(
        last.opcode == Opcode::OpAdd,
        "last_instruction.opcode wrong. got={:?}, want={:?}",
        last.opcode,
        Opcode::OpAdd
    );

    let previous = compiler.scopes[compiler.scope_index]
        .previous_instruction
        .clone()
        .expect("error");
    assert!(
        previous.opcode == Opcode::OpMul,
        "previous_instruction.opcode wrong. got={:?}, want={:?}",
        previous.opcode,
        Opcode::OpMul
    );
}

#[test]
fn test_function_calls() {
    let tests = vec![
        CompilerTestCase {
            input: "fn() { 24 }();",
            expected_constants: vec![
                Box::new(24 as i64),
                Box::new(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpCall, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "let noArg = fn() { 24 };
        noArg();",
            expected_constants: vec![
                Box::new(24 as i64),
                Box::new(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpGetGlobal, &vec![0]),
                make(Opcode::OpCall, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "let oneArg = fn(a) {};
            oneArg(24);",
            expected_constants: vec![
                Box::new(vec![make(Opcode::OpReturn, &Vec::new())]),
                Box::new(24 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpGetGlobal, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpCall, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "let manyArg = fn(a, b, c) {};
            manyArg(24, 25, 26);",
            expected_constants: vec![
                Box::new(vec![make(Opcode::OpReturn, &Vec::new())]),
                Box::new(24 as i64),
                Box::new(25 as i64),
                Box::new(26 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpGetGlobal, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpConstant, &vec![3]),
                make(Opcode::OpCall, &vec![3]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "let oneArg = fn(a) { a };
            oneArg(24);",
            expected_constants: vec![
                Box::new(vec![
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                Box::new(24 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpGetGlobal, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpCall, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "let manyArg = fn(a, b, c) { a; b; c };
            manyArg(24, 25, 26);",
            expected_constants: vec![
                Box::new(vec![
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpPop, &Vec::new()),
                    make(Opcode::OpGetLocal, &vec![1]),
                    make(Opcode::OpPop, &Vec::new()),
                    make(Opcode::OpGetLocal, &vec![2]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                Box::new(24 as i64),
                Box::new(25 as i64),
                Box::new(26 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpGetGlobal, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpConstant, &vec![3]),
                make(Opcode::OpCall, &vec![3]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];
    run_compiler_tests(tests);
}

#[test]
fn test_let_statement_scopes() {
    let tests = vec![
        CompilerTestCase {
            input: "let num = 55;
            fn() { num }
            ",
            expected_constants: vec![
                Box::new(55 as i64),
                Box::new(vec![
                    make(Opcode::OpGetGlobal, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() {
                        let num = 55;
                        num
            }",
            expected_constants: vec![
                Box::new(55 as i64),
                Box::new(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpSetLocal, &vec![0]),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() {
                        let a = 55;
                        let b = 77;
                        a + b
            }",
            expected_constants: vec![
                Box::new(55 as i64),
                Box::new(77 as i64),
                Box::new(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpSetLocal, &vec![0]),
                    make(Opcode::OpConstant, &vec![1]),
                    make(Opcode::OpSetLocal, &vec![1]),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpGetLocal, &vec![1]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![2]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];
    run_compiler_tests(tests);
}
