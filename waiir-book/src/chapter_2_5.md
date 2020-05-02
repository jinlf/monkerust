# 编写REPL

REPL表示“Read Eval Print Loop”，有时称做控制台，有时也称做交互模式，Python、Ruby和JavaScript等许多语言开发环境中都包含REPL。REPL读取输入，发送给解释器来求值，打印解释器的结果，再次启动。

我们当前还不知道怎样求值，但我们可以把Monkey源代码的Token打印出来，这样实现的REPL如下：
```rust,noplaypen
// src/repl/mod.rs

mod repl;
pub use repl::*;
```

```rust,noplaypen
// src/repl/repl.rs

use crate::lexer::*;
use crate::token::*;
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
        let mut l = Lexer::new(line);
        let mut tok = l.next_token();
        while tok.r#type != TokenType::EOF {
            writeln!(output, "{:?}", tok).unwrap();
            tok = l.next_token();
        }
    }
}
```

在main.rs中添加如下行：
```rust,noplaypen
// src/main.rs

mod repl;

// [...]
fn main() {
    println!("Hello, This is the Monkey programming language!");
    println!("Feel free to type in commands");
    repl::start(&mut std::io::stdin(), &mut std::io::stdout());
}
```

为了能够输出Token，还需要为Token添加Debug属性：
```rust,noplaypen
// src/token/token.rs

#[derive(Debug)]
pub struct Token {
    pub r#type: TokenType,
    pub literal: String,
}
```
这样，能够支持词法分析的REPL就做好了。您可以在命令行下输入cargo run来启动REPL：
```
$ cargo run
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let add = fn(x, y) { x + y; };
Token { r#type: LET, literal: "let" }
Token { r#type: IDENT, literal: "add" }
Token { r#type: ASSIGN, literal: "=" }
Token { r#type: FUNCTION, literal: "fn" }
Token { r#type: LPAREN, literal: "(" }
Token { r#type: IDENT, literal: "x" }
Token { r#type: COMMA, literal: "," }
Token { r#type: IDENT, literal: "y" }
Token { r#type: RPAREN, literal: ")" }
Token { r#type: LBRACE, literal: "{" }
Token { r#type: IDENT, literal: "x" }
Token { r#type: PLUS, literal: "+" }
Token { r#type: IDENT, literal: "y" }
Token { r#type: SEMICOLON, literal: ";" }
Token { r#type: RBRACE, literal: "}" }
Token { r#type: SEMICOLON, literal: ";" }
>> 
```

完美！接下来开发解析器吧。