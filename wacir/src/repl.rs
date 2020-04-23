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
use super::builtins::*;
use super::compiler::*;
use super::symbol_table::*;
use super::vm::*;
use std::cell::*;
use std::rc::*;

const PROMPT: &str = ">> ";

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);
    // let env = Rc::new(RefCell::new(new_environment()));
    let mut constants: Rc<RefCell<Vec<Object>>> = Rc::new(RefCell::new(Vec::new()));
    let globals: Rc<RefCell<[Option<Object>; GLOBALS_SIZE]>> =
        Rc::new(RefCell::new(init_globals()));
    let mut st = SymbolTable::new();
    for (i, v) in get_builtin_names().iter().enumerate() {
        st.define_builtin(i, v);
    }
    let symbol_table = Rc::new(RefCell::new(st));

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
        let mut comp = Compiler::new_with_state(Rc::clone(&symbol_table), Rc::clone(&constants));
        match comp.compile(Node::Program(program.unwrap())) {
            Ok(_) => {
                let code = comp.bytecode();
                constants = code.constants;
                let mut machine = Vm::new_with_globals_store(comp.bytecode(), Rc::clone(&globals));
                match machine.run() {
                    Ok(_) => {
                        let last_popped = machine.last_popped_stack_elem();
                        writeln!(output, "{}", last_popped.as_ref().unwrap().inspect()).unwrap();
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
