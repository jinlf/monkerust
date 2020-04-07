// src/repl.rs

use super::ast::*;
// use super::environment::*;
// use super::evaluator::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
// use std::cell::*;
use std::io::*;
// use std::rc::*;
use super::compiler::*;
use super::vm::*;

const PROMPT: &str = ">> ";

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);
    // let env = Rc::new(RefCell::new(new_environment()));

    loop {
        write!(output, "{}", PROMPT).unwrap();
        output.flush().unwrap();
        let mut line = String::new();
        if scanner.read_line(&mut line).is_err() {
            return;
        }
        let l = Lexer::new(&line);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        if p.errors.len() != 0 {
            print_parser_errors(output, &p.errors);
            continue;
        }
        let mut comp = Compiler::new();
        match comp.compile(Node::Program(program.unwrap())) {
            Ok(_) => {
                let mut machine = Vm::new(comp.bytecode());
                match machine.run() {
                    Ok(_) => {
                        let last_popped = machine.last_popped_stack_elem();
                        writeln!(output, "{}", last_popped.unwrap().inspect()).unwrap();
                    }
                    Err(err) => {
                        writeln!(output, "Woops! Executing bytecode failed:\n {}", err).unwrap();
                        continue;
                    }
                }
            }
            Err(err) => {
                writeln!(output, "Woops! Compilation failed:\n {}", err).unwrap();
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

fn print_parser_errors(output: &mut dyn Write, errors: &Vec<String>) {
    writeln!(output, "{}", MONKEY_FACE).unwrap();
    writeln!(output, "Woops! We ran into some monkey business here!").unwrap();
    writeln!(output, " parser errros:").unwrap();
    for msg in errors.iter() {
        writeln!(output, "\t{}", msg).unwrap();
    }
}
