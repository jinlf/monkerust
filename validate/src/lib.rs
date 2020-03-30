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

#[cfg(test)]
mod ast_test;
#[cfg(test)]
mod evaluator_test;
#[cfg(test)]
mod lexer_test;
#[cfg(test)]
mod object_test;
#[cfg(test)]
mod parser_test;
