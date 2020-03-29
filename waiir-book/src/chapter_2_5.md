# 编写REPL

REPL表示“Read Eval Print Loop”，有时称做控制台，有时也称做交互模式，Python、Ruby和JavaScript等许多语言开发环境中都包含REPL。REPL读取输入，发送给解释器来求值，打印解释器的结果，再次启动。

我们当前还不知道怎样求值，但我们可以把Monkey源代码的Token打印出来，这样实现的REPL如下：
```rust,noplaypen
// src/repl.rs

use super::lexer::*;
use super::token::*;
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
        let mut l = Lexer::new(&line);
        let mut tok = l.next_token();
        while tok.tk_type != TokenType::EOF {
            writeln!(output, "{:?}", tok).unwrap();
            tok = l.next_token();
        }
    }
}
```

在lib.rs中添加如下行：
```rust,noplaypen
// src/lib.rs

pub mod repl;
```

将main函数修改如下：
```rust,noplaypen
// src/main.rs

fn main() {
    println!("Hello, This is the Monkey programming language!");
    println!("Feel free to type in commands");
    repl::start(&mut std::io::stdin(), &mut std::io::stdout());
}
```

为了能够输出Token，还需要为Token添加Debug属性：
```rust,noplaypen
// src/token.rs

#[derive(Debug)]
pub struct Token {
    pub tk_type: TokenType,
    pub literal: String,
}
```
这样，能够支持词法分析的REPL就做好了。您可以在命令行下输入cargo run来启动REPL：
```
$ cargo run
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let add = fn(x, y) { x + y; };
Token { tk_type: LET, literal: "let" }
Token { tk_type: IDENT, literal: "add" }
Token { tk_type: ASSIGN, literal: "=" }
Token { tk_type: FUNCTION, literal: "fn" }
Token { tk_type: LPAREN, literal: "(" }
Token { tk_type: IDENT, literal: "x" }
Token { tk_type: COMMA, literal: "," }
Token { tk_type: IDENT, literal: "y" }
Token { tk_type: RPAREN, literal: ")" }
Token { tk_type: LBRACE, literal: "{" }
Token { tk_type: IDENT, literal: "x" }
Token { tk_type: PLUS, literal: "+" }
Token { tk_type: IDENT, literal: "y" }
Token { tk_type: SEMICOLON, literal: ";" }
Token { tk_type: RBRACE, literal: "}" }
Token { tk_type: SEMICOLON, literal: ";" }
>> 
```

完美！接下来开发解析器吧。