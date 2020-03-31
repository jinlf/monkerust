# 读入-解析-打印-循环

如果说我们之前的REPL其实是RLPL（读入-词法分析-打印-循环），那现在我们可以把词法分析器换成解析器，实现RPPL（读入-解析-打印-循环）。

```rust,noplaypen
// src/repl.rs

use super::ast::*;
use super::lexer::*;
use super::parser::*;
use std::io::*;

const PROMPT: &str = ">> ";

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);

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
        if let Some(prog) = program {
            writeln!(output, "{}", prog.string()).unwrap();
        }
    }
}

fn print_parser_errors(output: &mut dyn Write, errors: &Vec<String>) {
    for msg in errors.iter() {
        writeln!(output, "\t{}", msg).unwrap();
    }
}
```
执行cargo run：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let x =  1 * 2 * 3 * 4 * 5 
let x = ((((1 * 2) * 3) * 4) * 5);
>> x * y / 2 + 3 * 8 - 123
((((x * y) / 2) + (3 * 8)) - 123)
>> true == false
(true == false)
>> 
```
根据传统，加入Logo
```rust,noplaypen
// src/repl.rs

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
```
执行cargo run，并故意输入错误：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let x 12 * 3

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

Woops! We ran into some monkey business here!
 parser errros:
        expected next token to be ASSIGN, got INT instead
```

酷！让我们开始对AST求值吧。