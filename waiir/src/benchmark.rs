mod ast;
mod evaluator;
mod lexer;
mod object;
mod parser;
mod repl;
mod token;

use crate::ast::*;
use crate::lexer::*;
use crate::object::*;
use crate::parser::*;
use std::cell::*;
use std::rc::*;
use std::time::*;

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
fibonacci(28);
    ";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);

    match p.parse_program() {
        Ok(program) => {
            let env = Rc::new(RefCell::new(new_environment()));
            let now = SystemTime::now();
            match evaluator::eval(Node::Program(program), Rc::clone(&env)) {
                Ok(result) => {
                    println!(
                        "result={}, duration={:?}",
                        result.inspect(),
                        now.elapsed().unwrap().as_millis()
                    );
                }
                Err(err) => panic!("{}", err),
            }
        }
        Err(err) => panic!("{:?}", err),
    }
}
