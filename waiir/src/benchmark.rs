#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

mod ast;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod repl;
mod token;

use crate::lexer::*;
use crate::object::*;
use crate::parser::*;
use std::cell::*;
use std::rc::*;

fn main() {
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
fibonacci(30);
    ";

  let l = Lexer::new(String::from(input));
  let mut p = Parser::new(l);

  match p.parse_program() {
    Ok(program) => {
      let env = Rc::new(RefCell::new(new_environment()));
      let result = evaluator::evaluate(program, Rc::clone(&env));
      println!("result={}", result.inspect());
    }
    Err(err) => panic!("{:?}", err),
  }
}
