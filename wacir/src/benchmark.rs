// src/benchmark.rs

#[macro_use]
extern crate lazy_static;

mod ast;
#[macro_use]
mod code;
mod compiler;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod repl;
mod token;
mod vm;

use ast::*;
use compiler::*;
use evaluator::*;
use lexer::*;
use object::*;
use parser::*;
use std::cell::*;
use std::rc::*;
use std::time::*;
use vm::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let engine: String;
    if args.len() != 3 {
        println!("use -engine 'vm' or 'eval'");
        return;
    }
    match &args[1][..] {
        "-engine" => match &args[2][..] {
            "vm" | "eval" => engine = String::from(&args[2]),
            _ => {
                println!("use 'vm or 'eval'");
                return;
            }
        },
        _ => {
            println!("use 'vm or 'eval'");
            return;
        }
    }

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
    fibonacci(20);
    ";

    let result: Object;

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(program) => {
            let duration: u128;

            match &engine[..] {
                "vm" => {
                    let mut comp = Compiler::new();
                    match comp.compile(Node::Program(program)) {
                        Err(err) => {
                            println!("compiler error: {}", err);
                            return;
                        }
                        _ => {}
                    }
                    let mut machine = Vm::new(comp.bytecode());
                    let now = SystemTime::now();
                    match machine.run() {
                        Err(err) => {
                            println!("vm error: {}", err);
                            return;
                        }
                        _ => {}
                    }
                    duration = now.elapsed().unwrap().as_millis();
                    result = machine.last_popped_stack_elem().unwrap();
                }
                "eval" => {
                    let env = Rc::new(RefCell::new(new_environment()));
                    let now = SystemTime::now();
                    result = evaluate(Node::Program(program), Rc::clone(&env));
                    duration = now.elapsed().unwrap().as_millis();
                }
                _ => {
                    println!("use 'vm or 'eval'");
                    return;
                }
            }
            println!(
                "engine={}, result={}, duration={:?}",
                engine,
                result.inspect(),
                duration
            );
        }
        Err(errors) => panic!("{:?}", errors),
    }
}
