// src/main.rs

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

fn main() {
    println!("Hello, This is the Monkey programming language!");
    println!("Feel free to type in commands");
    repl::start(&mut std::io::stdin(), &mut std::io::stdout());
}
