# 定义Token

先从如下Monkey代码入手：
```js
let five = 5; 
let ten = 10;
let add = fn(x, y) { 
    x + y;
};
let result = add(five, ten);
```

这里包含整数5和10，变量名x、y、add和result，关键字let和fn，以及其它一些字符：(、)、{、}、=、,、;、+。

用Rust语言定义Token数据结构，以及表示Token类型的TokenType：
```rust,noplaypen
// src/token.rs

pub struct Token {
    pub tk_type: TokenType,
    pub literal: String,
}

pub enum TokenType {
    ILLEGAL,   // unknown character
    EOF,       // end of file
    IDENT,     // identifier
    INT,       // integer
    ASSIGN,    // =
    PLUS,      // +
    COMMA,     // ,
    SEMICOLON, // ;
    LPAREN,    // (
    RPAREN,    // )
    LBRACE,    // {
    RBRACE,    // }
    FUNCTION,  // function
    LET,       // let
}
```
这里有两个Token类型比较特殊：
- ILLEGAL：表示未知的Token，分析到ILLEGAL表示遇到错误；
- EOF：表示文件结束符，分析到EOF时就可以停止了。

下面开始写词法分析器。