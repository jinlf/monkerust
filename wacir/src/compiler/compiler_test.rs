// src/compiler_test.rs

use crate::ast::*;
use crate::code::*;
use crate::compiler::*;
use crate::lexer::*;
use crate::object::*;
use crate::parser::*;
use std::cell::*;
use std::rc::*;

enum ExpectedType {
    I64val(i64),
    Sval(String),
    VecInstructions(Vec<Instructions>),
}
impl From<i64> for ExpectedType {
    fn from(v: i64) -> Self {
        ExpectedType::I64val(v)
    }
}
impl From<&str> for ExpectedType {
    fn from(v: &str) -> Self {
        ExpectedType::Sval(String::from(v))
    }
}
impl From<Vec<Instructions>> for ExpectedType {
    fn from(v: Vec<Instructions>) -> Self {
        ExpectedType::VecInstructions(v)
    }
}

struct CompilerTestCase<'a> {
    input: &'a str,
    expected_constants: Vec<ExpectedType>,
    expected_instructions: Vec<Instructions>,
}

#[test]
fn test_integer_arithmetic() {
    let tests = vec![
        CompilerTestCase {
            input: "1 + 2",
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpAdd, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1; 2",
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 - 2",
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpSub, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 * 2",
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpMul, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "2 / 1",
            expected_constants: vec![ExpectedType::from(2i64), ExpectedType::from(1i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpDiv, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "-1",
            expected_constants: vec![ExpectedType::from(1i64)],
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

fn parse(input: &str) -> Result<Program, Vec<String>> {
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

fn test_constants(expected: &Vec<ExpectedType>, actual: Rc<RefCell<Vec<Object>>>) {
    assert!(
        expected.len() == actual.borrow().len(),
        "wrong number of constants. got={:?}, want={:?}",
        actual.borrow().len(),
        expected.len(),
    );

    for (i, constant) in expected.iter().enumerate() {
        match constant {
            ExpectedType::I64val(iv) => test_integer_object(*iv as i64, &actual.borrow()[i]),
            ExpectedType::Sval(sv) => test_string_object(sv, &actual.borrow()[i]),
            ExpectedType::VecInstructions(expected_instructions) => {
                if let Object::CompiledFunction(CompiledFunction {
                    instructions,
                    num_locals: _,
                    num_parameters: _,
                }) = &actual.borrow()[i]
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
}

fn test_integer_object(expected: i64, actual: &Object) {
    if let Object::Integer(Integer { value }) = actual {
        assert!(
            *value == expected,
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
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpGreaterThan, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 < 2",
            expected_constants: vec![ExpectedType::from(2i64), ExpectedType::from(1i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpGreaterThan, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 == 2",
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpConstant, &vec![1]),
                make(Opcode::OpEqual, &Vec::new()),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "1 != 2",
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
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
            expected_constants: vec![ExpectedType::from(10i64), ExpectedType::from(3333i64)],
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
                ExpectedType::from(10i64),
                ExpectedType::from(20i64),
                ExpectedType::from(3333i64),
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
            expected_constants: vec![ExpectedType::from(1i64), ExpectedType::from(2i64)],
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
            expected_constants: vec![ExpectedType::from(1i64)],
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
            expected_constants: vec![ExpectedType::from(1i64)],
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
            expected_constants: vec![ExpectedType::from("monkey")],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "\"mon\" + \"key\"",
            expected_constants: vec![ExpectedType::from("mon"), ExpectedType::from("key")],
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

fn test_string_object(expected: &str, actual: &Object) {
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
            expected_constants: vec![
                ExpectedType::from(1i64),
                ExpectedType::from(2i64),
                ExpectedType::from(3i64),
            ],
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
                ExpectedType::from(1i64),
                ExpectedType::from(2i64),
                ExpectedType::from(3i64),
                ExpectedType::from(4i64),
                ExpectedType::from(5i64),
                ExpectedType::from(6i64),
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
                ExpectedType::from(1i64),
                ExpectedType::from(2i64),
                ExpectedType::from(3i64),
                ExpectedType::from(4i64),
                ExpectedType::from(5i64),
                ExpectedType::from(6i64),
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
                ExpectedType::from(1i64),
                ExpectedType::from(2i64),
                ExpectedType::from(3i64),
                ExpectedType::from(4i64),
                ExpectedType::from(5i64),
                ExpectedType::from(6i64),
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
                ExpectedType::from(1i64),
                ExpectedType::from(2i64),
                ExpectedType::from(3i64),
                ExpectedType::from(1i64),
                ExpectedType::from(1i64),
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
                ExpectedType::from(1i64),
                ExpectedType::from(2i64),
                ExpectedType::from(2i64),
                ExpectedType::from(1i64),
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
                ExpectedType::from(5i64),
                ExpectedType::from(10i64),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpConstant, &vec![1]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![2, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() { 1; 2 }",
            expected_constants: vec![
                ExpectedType::from(1i64),
                ExpectedType::from(2i64),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpPop, &Vec::new()),
                    make(Opcode::OpConstant, &vec![1]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![2, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() { }",
            expected_constants: vec![ExpectedType::from(vec![make(
                Opcode::OpReturn,
                &Vec::new(),
            )])],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![0, 0]),
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
        .as_ref()
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
        .as_ref()
        .expect("error");
    assert!(
        last.opcode == Opcode::OpAdd,
        "last_instruction.opcode wrong. got={:?}, want={:?}",
        last.opcode,
        Opcode::OpAdd
    );

    let previous = compiler.scopes[compiler.scope_index]
        .previous_instruction
        .as_ref()
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
                ExpectedType::from(24i64),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![1, 0]),
                make(Opcode::OpCall, &vec![0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "let noArg = fn() { 24 };
        noArg();",
            expected_constants: vec![
                ExpectedType::from(24i64),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![1, 0]),
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
                ExpectedType::from(vec![make(Opcode::OpReturn, &Vec::new())]),
                ExpectedType::from(24i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![0, 0]),
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
                ExpectedType::from(vec![make(Opcode::OpReturn, &Vec::new())]),
                ExpectedType::from(24i64),
                ExpectedType::from(25i64),
                ExpectedType::from(26i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![0, 0]),
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
                ExpectedType::from(vec![
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                ExpectedType::from(24 as i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![0, 0]),
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
                ExpectedType::from(vec![
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpPop, &Vec::new()),
                    make(Opcode::OpGetLocal, &vec![1]),
                    make(Opcode::OpPop, &Vec::new()),
                    make(Opcode::OpGetLocal, &vec![2]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                ExpectedType::from(24i64),
                ExpectedType::from(25i64),
                ExpectedType::from(26i64),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![0, 0]),
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
                ExpectedType::from(55i64),
                ExpectedType::from(vec![
                    make(Opcode::OpGetGlobal, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpClosure, &vec![1, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() {
                        let num = 55;
                        num
            }",
            expected_constants: vec![
                ExpectedType::from(55i64),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![0]),
                    make(Opcode::OpSetLocal, &vec![0]),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![1, 0]),
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
                ExpectedType::from(55i64),
                ExpectedType::from(77i64),
                ExpectedType::from(vec![
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
                make(Opcode::OpClosure, &vec![2, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];
    run_compiler_tests(tests);
}

#[test]
fn test_builltins() {
    let tests = vec![
        CompilerTestCase {
            input: "len([]); push([], 1);",
            expected_constants: vec![ExpectedType::from(1i64)],
            expected_instructions: vec![
                make(Opcode::OpGetBuiltin, &vec![0]),
                make(Opcode::OpArray, &vec![0]),
                make(Opcode::OpCall, &vec![1]),
                make(Opcode::OpPop, &Vec::new()),
                make(Opcode::OpGetBuiltin, &vec![5]),
                make(Opcode::OpArray, &vec![0]),
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpCall, &vec![2]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "fn() { len([]) }",
            expected_constants: vec![ExpectedType::from(vec![
                make(Opcode::OpGetBuiltin, &vec![0]),
                make(Opcode::OpArray, &vec![0]),
                make(Opcode::OpCall, &vec![1]),
                make(Opcode::OpReturnValue, &Vec::new()),
            ])],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![0, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];
    run_compiler_tests(tests);
}

#[test]
fn test_functions_without_return_value() {
    let tests = vec![CompilerTestCase {
        input: "fn() {}",
        expected_constants: vec![ExpectedType::from(vec![make(
            Opcode::OpReturn,
            &Vec::new(),
        )])],
        expected_instructions: vec![
            make(Opcode::OpClosure, &vec![0, 0]),
            make(Opcode::OpPop, &Vec::new()),
        ],
    }];

    run_compiler_tests(tests);
}

#[test]
fn test_closure() {
    let tests = vec![
        CompilerTestCase {
            input: "
            fn(a) {
                fn(b) {
                    a + b
                }
            }
            ",
            expected_constants: vec![
                ExpectedType::from(vec![
                    make(Opcode::OpGetFree, &vec![0]),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                ExpectedType::from(vec![
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpClosure, &vec![0, 1]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![1, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "
        fn(a) {
            fn(b) {
                fn(c) {
                    a + b + c
                }
            }
        }",
            expected_constants: vec![
                ExpectedType::from(vec![
                    make(Opcode::OpGetFree, &vec![0]),
                    make(Opcode::OpGetFree, &vec![1]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                ExpectedType::from(vec![
                    make(Opcode::OpGetFree, &vec![0]),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpClosure, &vec![0, 2]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                ExpectedType::from(vec![
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpClosure, &vec![1, 1]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpClosure, &vec![2, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
        CompilerTestCase {
            input: "
            let global = 55;
            fn() {
                let a = 66;
                fn() {
                    let b = 77;
                    fn() {
                        let c = 88;
                        global + a + b + c;
                    }
                }
            }",
            expected_constants: vec![
                ExpectedType::from(55i64),
                ExpectedType::from(66i64),
                ExpectedType::from(77i64),
                ExpectedType::from(88i64),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![3]),
                    make(Opcode::OpSetLocal, &vec![0]),
                    make(Opcode::OpGetGlobal, &vec![0]),
                    make(Opcode::OpGetFree, &vec![0]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpGetFree, &vec![1]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpAdd, &Vec::new()),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![2]),
                    make(Opcode::OpSetLocal, &vec![0]),
                    make(Opcode::OpGetFree, &vec![0]),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpClosure, &vec![4, 2]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
                ExpectedType::from(vec![
                    make(Opcode::OpConstant, &vec![1]),
                    make(Opcode::OpSetLocal, &vec![0]),
                    make(Opcode::OpGetLocal, &vec![0]),
                    make(Opcode::OpClosure, &vec![5, 1]),
                    make(Opcode::OpReturnValue, &Vec::new()),
                ]),
            ],
            expected_instructions: vec![
                make(Opcode::OpConstant, &vec![0]),
                make(Opcode::OpSetGlobal, &vec![0]),
                make(Opcode::OpClosure, &vec![6, 0]),
                make(Opcode::OpPop, &Vec::new()),
            ],
        },
    ];
    run_compiler_tests(tests);
}
