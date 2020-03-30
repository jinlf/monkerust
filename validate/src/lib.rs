// src/lib.rs

pub mod ast;
pub mod builtins;
pub mod environment;
pub mod evaluator;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod repl;
pub mod token;

mod ast_test;
mod evaluator_test;
mod lexer_test;
mod object_test;
mod parser_test;
