# 扩展Token集合及词法分析器

继续扩展test_next_token测试用例：
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
!-/*5;
5 < 10 > 5;
";
// [...]
```
后面添加的用于验证的Token序列：
```rust,noplaypen
// src/lexer/lexer.rs

// [...]
        (TokenType::BANG, "!"),
        (TokenType::MINUS, "-"),
        (TokenType::SLASH, "/"),
        (TokenType::ASTERISK, "*"),
        (TokenType::INT, "5"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::INT, "5"),
        (TokenType::LT, "<"),
        (TokenType::INT, "10"),
        (TokenType::GT, ">"),
        (TokenType::INT, "5"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::EOF, ""),
    ];        
// [...]
```

扩展TokenType，如下：
```rust,noplaypen
// src/token/token.rs

pub enum TokenType {
// [...]
    MINUS,     // -
    BANG,      // !
    ASTERISK,  // *
    SLASH,     // /
    LT,        // <
    GT,        // >
}    
```
测试失败的信息如下：
```
thread 'lexer::lexer_test::test_next_token' panicked at 'test[36] - tokentype wrong. expected=BANG, got=ILLEGAL', src/lexer/lexer_test.rs:74:9
```
为了支持这些新增加的TokenType，需要扩展我们的词法分析器：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
// [...]
        match self.ch {
// [...]
            b'-' => tok = new_token(TokenType::MINUS, self.ch),
            b'!' => tok = new_token(TokenType::BANG, self.ch),
            b'/' => tok = new_token(TokenType::SLASH, self.ch),
            b'*' => tok = new_token(TokenType::ASTERISK, self.ch),
            b'<' => tok = new_token(TokenType::LT, self.ch),
            b'>' => tok = new_token(TokenType::GT, self.ch),
            0 => {
// [...]                
    }
```
测试通过！

再增加一些新的测试用例。
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
!-/*5;
5 < 10 > 5;

if (5 < 10) { 
    return true;
} else {
    return false;
}";
// [...]
}
```
这里用到了if、else、return、true和false。
新增加测试用例：
```rusr,noplaypen
// src/lexer/lexer.rs

// [...]
        (TokenType::IF, "if"),
        (TokenType::LPAREN, "("),
        (TokenType::INT, "5"),
        (TokenType::LT, "<"),
        (TokenType::INT, "10"),
        (TokenType::RPAREN, ")"),
        (TokenType::LBRACE, "{"),
        (TokenType::RETURN, "return"),
        (TokenType::TRUE, "true"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::RBRACE, "}"),
        (TokenType::ELSE, "else"),
        (TokenType::LBRACE, "{"),
        (TokenType::RETURN, "return"),
        (TokenType::FALSE, "false"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::RBRACE, "}"),
        (TokenType::EOF, ""),
    ];        
// [...]        
```

然后扩展TokenType如下：
```rust,noplaypen
// src/token/token.rs

pub enum TokenType {
// [...]
    TRUE,      // true
    FALSE,     // false
    IF,        // if
    ELSE,      // else
    RETURN,    // return
}
```
新加的这几个TokenType都是关键字，需要扩展lookup_ident函数：
```rust,noplaypen
// src/token/token.rs

pub fn lookup_ident(ident: &str) -> TokenType {
    match ident {
        "fn" => TokenType::FUNCTION,
        "let" => TokenType::LET,
        "true" => TokenType::TRUE,
        "false" => TokenType::FALSE,
        "if" => TokenType::IF,
        "else" => TokenType::ELSE,
        "return" => TokenType::RETURN,
        _ => TokenType::IDENT,
    }
}
```
测试通过！

下面试一下“==”和“!=”操作符：
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
!-/*5;
5 < 10 > 5;

if (5 < 10) { 
    return true;
} else {
    return false;
}

10 == 10;
10 != 9;
";
// [...]
}
```
添加验证Token序列：
```rust,noplaypen
// src/lexer/lexer.rs

// [...]
        (TokenType::INT, "10"),
        (TokenType::EQ, "=="),
        (TokenType::INT, "10"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::INT, "10"),
        (TokenType::NOTEQ, "!="),
        (TokenType::INT, "9"),
        (TokenType::SEMICOLON, ";"),
        (TokenType::EOF, ""),
    ];        
// [...]     
```

由于只看当前字符已经不能确定是“=”还是“==”，是“!”还是“!=”，需要扩展词法分析器，向前多看一个字符。首先需要增加一个peek_char方法。
```rust,noplaypen
// src/lexer/lexer.rs

    fn peek_char(&mut self) -> u8 {
        if self.read_position >= self.input.len() {
            return 0;
        } else {
            return self.input.bytes().nth(self.read_position).unwrap();
        }
    }
```
这个peek_char方法与read_char方法很类似，只是并不改变position和read_position的值。

增加“==”和“!=”Token类型。
```rust,noplaypen
// src/token/token.rs

pub enum TokenType {
// [...]
    EQ,        // ==
    NOTEQ,     // !=
}
```
测试失败的信息如下：
```
thread 'lexer::lexer_test::test_next_token' panicked at 'test[66] - tokentype wrong. expected=EQ, got=ASSIGN', src/lexer/lexer_test.rs:108:9
```

修改next_token方法，如下：
```rust,noplaypen
// src/lexer/lexer.rs

    pub fn next_token(&mut self) -> Token {
// [...]
        match self.ch {
            b'=' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    tok = Token {
                        r#type: TokenType::EQ,
                        literal: String::from("=="),
                    }
                } else {
                    tok = new_token(TokenType::ASSIGN, self.ch)
                }
            }
// [...]     
            b'!' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    tok = Token {
                        r#type: TokenType::NOTEQ,
                        literal: String::from("!="),
                    }
                } else {
                    tok = new_token(TokenType::BANG, self.ch)
                }
            }
// [...]           
        }        
    }
```
这里当当前字符是“=”时，我们通过调用peek_char方法看看下一个字符是否是“=”，如果是，就将这两个字符连起来构成“==”Token，否则返回“=”Token。“!”与“!=”的处理也是如此。

测试通过。

词法分析器就完成了！在编写解析器之前，我们先编写一个REPL。

