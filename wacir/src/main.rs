// src/main.rs

#[macro_use]
extern crate lazy_static;

mod ast;
mod builtins;
#[macro_use]
mod code;
mod compiler;
mod environment;
mod evaluator;
mod frame;
mod lexer;
mod object;
mod parser;
mod repl;
mod symbol_table;
mod token;
mod vm;

fn main() {
    println!("Hello, This is the Monkey programming language!");
    println!("Feel free to type in commands");
    repl::start(&mut std::io::stdin(), &mut std::io::stdout());
}
