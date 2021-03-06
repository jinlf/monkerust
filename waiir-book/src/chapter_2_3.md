# 词法分析器

词法分析器输入的是源代码，每次调用next_token方法时返回下一个Token。本文为了简化，不在Token中包含文件名和行号等信息。

遵循测试驱动开发（Test-Driven Development，简称TDD）的原则，先写单元测试用例：
```rust,noplaypen
// src/lexer/mod.rs

#[cfg(test)]
mod lexer_test;
```

```rust,noplaypen
// src/lexer/lexer_test.rs

use crate::token::*;

#[test]
fn test_next_token() {
    let input = "=+(){},;";
    let tests = [
        (TokenType::ASSIGN, "="),
        (TokenType::PLUS, "+"),
        (TokenType::LPAREN, "("),
        (TokenType::RPAREN, ")"),
        (TokenType::LBRACE, "{"),
        (TokenType::RBRACE, "}"),
        (TokenType::COMMA, ","),
        (TokenType::SEMICOLON, ";"),
        (TokenType::EOF, ""),
    ];

    let mut l = Lexer::new(String::from(input));
    for (i, tt) in tests.iter().enumerate() {
        let tok = l.next_token();
        assert!(
            tok.r#type == tt.0,
            "test[{}] - tokentype wrong. expected={:?}, got={:?}",
            i,
            tt.0,
            tok.r#type
        );
        assert!(
            tok.literal == tt.1,
            "test[{}] - literal wrong. expected={}, got={}",
            i,
            tt.1,
            tok.literal
        );
    }
}
```
上述代码中的#[test]属性表示下面的函数是个测试函数，在自动化测试时会执行。#[cfg(test)]表示下面的模块是一个测试模块。代码的功能是将一个字符串作为输入，让词法分析器分析并使用断言验证输出的结果与预期结果是否一致。

为了支持Rust的自动化测试，首先需要在src/main.rs文件头部添加：
```rust,noplaypen
// src/main.rs

mod lexer;
mod token;

// [...]
```

这样就可以在命令行下执行Rust的自动化测试命令：
```
$ cargo test
```
不出意外，测试失败了，因为我们还没有编写正式的代码。

首先补充词法分析器的定义和new方法，如下：
```rust,noplaypen
// src/lexer/mod.rs

mod lexer;
pub use lexer::*;
```

```rust,noplaypen
// src/lexer/lexer.rs

pub struct Lexer {
    input: String,
    position: usize,      // 当前字符位置
    read_position: usize, // 当前读取位置（在当前字符位置之后）
    ch: u8,               // 当前字符
}
impl Lexer {
    pub fn new(input: String) -> Lexer {
        let mut l = Lexer {
            input: input,
            position: 0,
            read_position: 0,
            ch: 0,
        };
        l.read_char();
        l
    }
}
```

这里使用read_position是为了向前看若干字符。

定义词法分析器的read_char方法如下：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input.bytes().nth(self.read_position).unwrap();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
```
read_char方法的目的是读取下一个字符，并前进一个字符。如果到输入结尾不能读字符时，就设置ch为0。这个特殊符号用来代表EOF。

简单起见，本文实现的解释器只支持ASCII。您可以自己尝试支持Unicode。

这样，创建词法分析器的同时，读入第一个字符，初始化了ch、position和read_position。

下面来实现词法分析器的next_token方法：
```rust,noplaypen
// src/lexer/lexer.rs

use crate::token::*;

// [...]
    pub fn next_token(&mut self) -> Token {
        let tok: Token;

        match self.ch {
            b'=' => tok = new_token(TokenType::ASSIGN, self.ch),
            b';' => tok = new_token(TokenType::SEMICOLON, self.ch),
            b'(' => tok = new_token(TokenType::LPAREN, self.ch),
            b')' => tok = new_token(TokenType::RPAREN, self.ch),
            b',' => tok = new_token(TokenType::COMMA, self.ch),
            b'+' => tok = new_token(TokenType::PLUS, self.ch),
            b'{' => tok = new_token(TokenType::LBRACE, self.ch),
            b'}' => tok = new_token(TokenType::RBRACE, self.ch),
            0 => {
                tok = Token {
                    r#type: TokenType::EOF,
                    literal: String::new(),
                }
            }
            _ => tok = new_token(TokenType::ILLEGAL, self.ch),
        }
        self.read_char();
        tok
    }
// [...]
pub fn new_token(token_type: TokenType, ch: u8) -> Token {
    let mut literal = String::new();
    literal.push(ch as char);
    Token {
        r#type: token_type,
        literal: literal,
    }
}
```
next_token方法的功能就是根据当前字符，返回下一个Token。

在lexer_test.rs中加入：
```rust,noplaypen
// src/lexer/lexer_test.rs

use crate::lexer::*;
```
再次执行
```
cargo test
```
还是出错：
```
error[E0369]: binary operation `==` cannot be applied to type `token::token::TokenType`
  --> src/lexer/lexer_test.rs:24:24
   |
24 |             tok.r#type == tt.0,
   |             ---------- ^^ ---- token::token::TokenType
   |             |
   |             token::token::TokenType
   |
   = note: an implementation of `std::cmp::PartialEq` might be missing for `token::token::TokenType`
```   
这是因为TokenType没有实现运算符"=="不能直接比较。解决的方法是在TokenType定义上加上PartialEq属性，另外为了打印输出TokenType，还需要加上Debug属性，如下：
```rust,noplaypen
// src/token/token.rs

#[derive(PartialEq, Debug)]
pub enum TokenType {
// [...]
```
执行cargo test，成功：
```
running 1 test
test lexer::lexer_test::test_next_token ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
测试通过表明现在词法分析器已经能够支持测试用例中的各种Token了。

修改测试用例，测试本章前面提到的Monkey代码：
```rust,noplaypen
// src/lexer/lexer_test.rs

fn test_next_token() {
    let input = "
let five = 5;        
let ten = 10;

let add = fn(x, y) { 
    x + y;
};

let result = add(five, ten);
";
    let tests = [
        (TokenType::LET, "let"),
        (TokenType::IDENT, "five"),
        (TokenType::ASSIGN, "="),
        (TokenType::INT, "5"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::LET, "let"),
        (TokenType::IDENT, "ten"),
        (TokenType::ASSIGN, "="),
        (TokenType::INT, "10"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::LET, "let"),
        (TokenType::IDENT, "add"),
        (TokenType::ASSIGN, "="),
        (TokenType::FUNCTION, "fn"),
        (TokenType::LPAREN, "("),
        (TokenType::IDENT, "x"),
        (TokenType::COMMA, ","),
        (TokenType::IDENT, "y"),
        (TokenType::RPAREN, ")"),
        (TokenType::LBRACE, "{"),
        (TokenType::IDENT, "x"),
        (TokenType::PLUS, "+"),
        (TokenType::IDENT, "y"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::RBRACE, "}"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::LET, "let"),
        (TokenType::IDENT, "result"),
        (TokenType::ASSIGN, "="),
        (TokenType::IDENT, "add"),
        (TokenType::LPAREN, "("),
        (TokenType::IDENT, "five"),
        (TokenType::COMMA, ","),
        (TokenType::IDENT, "ten"),
        (TokenType::RPAREN, ")"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::EOF, ""),
    ];
// [...]
```
测试必然会失败，因为词法分析器还没有支持标识符、关键字和数字。

支持标识符的代码如下：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
        let tok: Token;
        match self.ch {
// [...]            
            _ => {
                if is_letter(self.ch) {
                    tok = Token {
                        r#type: TokenType::IDENT,
                        literal: String::from(self.read_identifier()),
                    };
                    return tok;
                }
                tok = new_token(TokenType::ILLEGAL, self.ch);
            }
        }
// [...] 
    }

    fn read_identifier(&mut self) -> &str {
        let position = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }
        &self.input[position..self.position]
    }      
// [...]
fn is_letter(ch: u8) -> bool {
    ch.is_ascii_alphabetic() || ch == b'_'
}
```
不符合标识符字符的情况会返回ILLEGAL。

is_letter函数检查是否是字符和下划线，这是标识符中支持的字符。

这里Token的literal不再是单一字符，所以没有使用new_token函数来创建Token，而是根据read_identifier方法返回的字符串直接创建Token。

需要支持关键字，在token.rs中实现一个关键字查找函数：
```rust,noplaypen
// src/token/token.rs

pub fn lookup_ident(ident: &str) -> TokenType {
    match ident {
        "fn" => TokenType::FUNCTION,
        "let" => TokenType::LET,
        _ => TokenType::IDENT,
    }
}
```
目前只支持测试用例中的let和fn关键字。

这样，处理标识符的代码就应该改成：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
        let tok: Token;
        match self.ch {
// [...]            
            _ => {
                if is_letter(self.ch) {
                    let literal = self.read_identifier();
                    tok = Token {
                        r#type: lookup_ident(&literal),
                        literal: String::from(literal),
                    };
                    return tok;
                }
                tok = new_token(TokenType::ILLEGAL, self.ch);
            }
        }
// [...] 
    }
```
这里调用了read_identifier方法，不需要在返回之前再次调用read_char方法来更新Lexer，所以用return语句直接返回tok即可。

执行cargo test，仍然报错，如下：
```
---- lexer::lexer_test::test_next_token stdout ----
thread 'lexer::lexer_test::test_next_token' panicked at 'test[0] - tokentype wrong. expected=LET, got=ILLEGAL', src/lexer/lexer_test.rs:60:9
```
原因是我们的词法分析器没有跳过空格、回车等特殊分隔字符，需要处理一下：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
        let tok: Token;

        self.skip_whitespace();

        match self.ch {
// [...]            
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.ch {
                b' ' | b'\t' | b'\n' | b'\r' => self.read_char(),
                _ => return,
            }
        }
    }
```

下面再加上整数的词法分析：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
        let tok: Token;

        self.skip_whitespace();
        
        match self.ch {
// [...]            
            _ => {
                if is_letter(self.ch) {
                    let literal = self.read_identifier();
                    tok = Token {
                        r#type: lookup_ident(&literal),
                        literal: String::from(literal),
                    };
                    return tok;
                } else if self.ch.is_ascii_digit() {
                    tok = Token {
                        r#type: TokenType::INT,
                        literal: String::from(self.read_number()),
                    };
                    return tok;
                }
                tok = new_token(TokenType::ILLEGAL, self.ch);
            }
        }
// [...]            
    }

    fn read_number(&mut self) -> &str {
        let position = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        &self.input[position..self.position]
    }
```
执行cargo test，成功！

由此，前文提到的Monkey代码段已经能够被我们的词法分析器分析了。

注意：简单起见，本文实现的解释器不支持浮点数，以及十六进制、八进制等表示方式。感兴趣的读者您可以自行实现。
