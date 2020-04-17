#![feature(test)]

#[macro_use]
extern crate lazy_static;

pub mod ast;
pub mod builtins;
#[macro_use]
pub mod code;
pub mod compiler;
pub mod environment;
pub mod evaluator;
pub mod frame;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod repl;
pub mod symbol_table;
pub mod token;
pub mod vm;

#[cfg(test)]
mod benchmark_test;
// mod ast_test;
// mod code_test;
// mod compiler_test;
// mod evaluator_test;
// mod lexer_test;
// mod object_test;
// mod parser_test;
// mod symbol_table_test;
// mod vm_test;
