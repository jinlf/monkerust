# 定义Token

定义Token：
```rust,noplaypen
// src/token.rs

pub struct Token {
    pub tk_type: TokenType,
    pub literal: String,
}
```


定义TokenType
```rust,noplaypen
// src/token.rs

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