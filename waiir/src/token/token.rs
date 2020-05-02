// src/token/token.rs

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub literal: String,
}

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
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
    MINUS,     // -
    BANG,      // !
    ASTERISK,  // *
    SLASH,     // /
    LT,        // <
    GT,        // >
    TRUE,      // true
    FALSE,     // false
    IF,        // if
    ELSE,      // else
    RETURN,    // return
    EQ,        // ==
    NOTEQ,     // !=
    STRING,    // string
    LBRACKET,  // [
    RBRACKET,  // ]
    COLON,     // :
}

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
