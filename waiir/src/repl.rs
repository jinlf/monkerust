use super::env::*;
use super::evaluator::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use std::cell::*;
use std::io::*;
use std::rc::*;

const PROMPT: &str = ">> ";

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

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);
    let env = Rc::new(RefCell::new(new_env()));
    loop {
        writeln!(output, "{}", PROMPT).unwrap();
        let mut line = String::new();
        if scanner.read_line(&mut line).is_err() {
            return;
        }
        let l = Lexer::new(&line);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        if let Some(prog) = program {
            if p.get_errors().len() != 0 {
                print_parser_errors(output, p.get_errors());
                continue;
            }

            let evaluated = eval(prog, Rc::clone(&env));
            if evaluated.is_some() {
                writeln!(output, "{}", evaluated.unwrap().inspect()).unwrap();
            }
        } else {
            writeln!(output, "parse error").unwrap();
        }
    }

    fn print_parser_errors(output: &mut dyn Write, errors: Vec<String>) {
        write!(output, "{}", MONKEY_FACE).unwrap();
        writeln!(output, "Woops! We ran into some monkey business here!").unwrap();
        writeln!(output, " parse errors:").unwrap();
        for msg in errors {
            writeln!(output, "\t{}", msg).unwrap();
        }
    }
}
