// src/repl.rs

use crate::ast::*;
use crate::compiler::*;
use crate::evaluator::*;
use crate::lexer::*;
use crate::object::*;
use crate::parser::*;
use crate::vm::*;
use std::cell::*;
use std::io::*;
use std::rc::*;

const PROMPT: &str = ">> ";

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);
    let mut constants: Rc<RefCell<Vec<Object>>> = Rc::new(RefCell::new(Vec::new()));
    let globals: Rc<RefCell<Vec<Option<Object>>>> = Rc::new(RefCell::new(vec![None; GLOBALS_SIZE]));
    let symbol_table: Rc<RefCell<SymbolTable>> = Rc::new(RefCell::new(SymbolTable::new()));

    for (i, v) in get_builtin_names().iter().enumerate() {
        symbol_table.borrow_mut().define_builtin(i, v);
    }

    loop {
        write!(output, "{}", PROMPT).unwrap();
        output.flush().unwrap();
        let mut line = String::new();
        if scanner.read_line(&mut line).is_err() {
            return;
        }
        let l = Lexer::new(&line);
        let mut p = Parser::new(l);
        match p.parse_program() {
            Err(errors) => {
                print_parser_errors(output, &errors);
                continue;
            }
            Ok(program) => {
                let mut comp =
                    Compiler::new_with_state(Rc::clone(&symbol_table), Rc::clone(&constants));
                match comp.compile(Node::Program(program)) {
                    Ok(_) => {
                        let code = comp.bytecode();
                        constants = Rc::clone(&code.constants);
                        let mut machine = Vm::new_with_globals_store(code, Rc::clone(&globals));
                        match machine.run() {
                            Ok(_) => {
                                if let Some(last_popped) = machine.last_popped_stack_elem {
                                    writeln!(output, "{}", last_popped.inspect()).unwrap();
                                }
                            }
                            Err(err) => {
                                writeln!(output, "{}", MONKEY_FACE).unwrap();
                                writeln!(output, "Woops! Executing bytecode failed:\n {}", err)
                                    .unwrap();
                            }
                        }
                    }
                    Err(err) => {
                        writeln!(output, "{}", MONKEY_FACE).unwrap();
                        writeln!(output, "Woops! Compilation failed:\n {}", err).unwrap();
                    }
                }
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
