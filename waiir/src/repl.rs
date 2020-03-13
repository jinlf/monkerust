use super::lexer::*;
use super::token::*;
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
        let mut l = Lexer::new(line);

        let mut tok = l.next_token();
        while tok.tk_type != TokenType::EOF {
            fmt.write_fmt(format_args!("{:?}\n", tok)).unwrap();
            tok = l.next_token();
        }
    }
}
