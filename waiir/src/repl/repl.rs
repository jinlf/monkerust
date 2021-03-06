// src/repl/repl.rs

use crate::evaluator::*;
use crate::lexer::*;
use crate::object::*;
use crate::parser::*;
use std::cell::*;
use std::io::*;
use std::rc::*;

const PROMPT: &str = ">> ";

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);
    let env = Rc::new(RefCell::new(new_environment()));

    loop {
        write!(output, "{}", PROMPT).unwrap();
        output.flush().unwrap();
        let mut line = String::new();
        if scanner.read_line(&mut line).is_err() {
            return;
        }
        let l = Lexer::new(line);
        let mut p = Parser::new(l);
        match p.parse_program() {
            Ok(program) => {
                writeln!(output, "{}", evaluate(program, Rc::clone(&env)).inspect()).unwrap();
            }
            Err(errors) => {
                print_parser_errors(output, &errors);
                continue;
            }
        }
    }
}

const MONKEY_FACE: &str = r#"
            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"""""""-./, -' /
   ''-' /_   ^ ^   _\ '-''
       |  \._   _./  |
       \   \ '~' /   /
        '._ '-=-' _.'
           '-----'
"#;

fn print_parser_errors(output: &mut dyn Write, errors: &[String]) {
    writeln!(output, "{}", MONKEY_FACE).unwrap();
    writeln!(output, "Woops! We ran into some monkey business here!").unwrap();
    writeln!(output, " parser errros:").unwrap();
    for msg in errors.iter() {
        writeln!(output, "\t{}", msg).unwrap();
    }
}
