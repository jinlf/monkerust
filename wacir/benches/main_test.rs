#![feature(test)]

extern crate test;

use std::cell::*;
use std::rc::*;
use std::time::*;

use wacir::ast::*;
use wacir::compiler::*;
use wacir::environment::*;
use wacir::evaluator::*;
use wacir::lexer::*;
use wacir::object::*;
use wacir::parser::*;
use wacir::vm::*;

use test::Bencher;

#[bench]
fn bench_parse(b: &mut test::Bencher) {
    b.iter(|| main_parse());
}

#[bench]
fn bench_compile(b: &mut test::Bencher) {
    b.iter(|| main_compile());
}

#[bench]
fn bench_run(b: &mut test::Bencher) {
    b.iter(|| main_run());
}

#[bench]
fn bench_all(b: &mut test::Bencher) {
    b.iter(|| main_all());
}

fn main_parse() {
    let input = "
    let fibonacci = fn(x) {
        if (x == 0) { 
            0
        } else {
            if (x == 1) {
                return 1;
            } else {
                fibonacci(x - 1) + fibonacci(x - 2); 
            }
        } 
    };
    fibonacci(22);
    ";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
}

fn main_compile() {
    let input = "
    let fibonacci = fn(x) {
        if (x == 0) { 
            0
        } else {
            if (x == 1) {
                return 1;
            } else {
                fibonacci(x - 1) + fibonacci(x - 2); 
            }
        } 
    };
    fibonacci(22);
    ";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();

    let mut comp = Compiler::new();
    match comp.compile(Node::Program(program.unwrap())) {
        Err(err) => {
            println!("compiler error: {}", err);
            return;
        }
        _ => {}
    }
}

fn main_run() {
    let input = "
    let fibonacci = fn(x) {
        if (x == 0) { 
            0
        } else {
            if (x == 1) {
                return 1;
            } else {
                fibonacci(x - 1) + fibonacci(x - 2); 
            }
        } 
    };
    fibonacci(22);
    ";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();

    let mut comp = Compiler::new();
    match comp.compile(Node::Program(program.unwrap())) {
        Err(err) => {
            println!("compiler error: {}", err);
            return;
        }
        _ => {}
    }

    let mut machine = Vm::new(comp.bytecode());
    match machine.run() {
        Err(err) => {
            println!("vm error: {}", err);
            return;
        }
        _ => {}
    }
}

fn main_all() {
    let input = "
    let fibonacci = fn(x) {
        if (x == 0) { 
            0
        } else {
            if (x == 1) {
                return 1;
            } else {
                fibonacci(x - 1) + fibonacci(x - 2); 
            }
        } 
    };
    fibonacci(22);
    ";

    let result: Option<Object>;

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();

    let mut comp = Compiler::new();
    match comp.compile(Node::Program(program.unwrap())) {
        Err(err) => {
            println!("compiler error: {}", err);
            return;
        }
        _ => {}
    }

    let mut machine = Vm::new(comp.bytecode());
    match machine.run() {
        Err(err) => {
            println!("vm error: {}", err);
            return;
        }
        _ => {}
    }
    result = machine.last_popped_stack_elem();
}
