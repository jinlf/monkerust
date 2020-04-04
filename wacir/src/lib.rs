// src/lib.rs

pub mod ast;
pub mod builtins;
pub mod code;
pub mod compiler;
pub mod environment;
pub mod evaluator;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod repl;
pub mod token;
pub mod vm;

#[cfg(test)]
mod compiler_test;
// #[cfg(test)]
// mod ast_test;
#[cfg(test)]
mod code_test;
// #[cfg(test)]
// mod evaluator_test;
// #[cfg(test)]
// mod lexer_test;
// #[cfg(test)]
// mod object_test;
// #[cfg(test)]
// mod parser_test;
#[cfg(test)]
mod vm_test;