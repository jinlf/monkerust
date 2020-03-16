use super::lexer::*;
use super::parser::*;
use std::io::*;

const PROMPT: &str = ">> ";

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
        if let Some(p) = program {
            fmt.write_fmt(format_args!("{:?}\n", p)).unwrap();
        } else {
            fmt.write_fmt(format_args!("parse error\n")).unwrap();
        }
    }
}
