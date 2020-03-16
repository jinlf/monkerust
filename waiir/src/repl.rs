use super::ast::*;
use super::evaluator::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;
use std::io::*;

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
    let mut fmt = LineWriter::new(output);
    loop {
        fmt.write_fmt(format_args!("{}", PROMPT)).unwrap();
        fmt.flush().unwrap();
        let mut line = String::new();
        if scanner.read_line(&mut line).is_err() {
            return;
        }
        let l = Lexer::new(line);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        if let Some(prog) = program {
            if p.get_errors().len() != 0 {
                print_parser_errors(&mut fmt, p.get_errors());
                continue;
            }

            let evaluated = eval(prog);
            if evaluated.is_some() {
                fmt.write_fmt(format_args!("{}\n", evaluated.unwrap().inspect()))
                    .unwrap();
            }
        } else {
            fmt.write_fmt(format_args!("parse error\n")).unwrap();
        }
    }

    fn print_parser_errors(fmt: &mut LineWriter<&mut dyn Write>, errors: Vec<String>) {
        fmt.write_fmt(format_args!("{}", MONKEY_FACE)).unwrap();
        fmt.write_fmt(format_args!(
            "Woops! We ran into some monkey business here!\n"
        ))
        .unwrap();
        fmt.write_fmt(format_args!(" parse errors:\n")).unwrap();
        for msg in errors {
            fmt.write_fmt(format_args!("\t{}\n", msg)).unwrap();
        }
    }
}
