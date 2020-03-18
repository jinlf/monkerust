#[derive(Debug, PartialEq)]
pub struct Token {
    pub tk_type: TokenType,
    pub literal: String,
}
impl Clone for Token {
    fn clone(&self) -> Self {
        Token {
            tk_type: self.tk_type,
            literal: self.literal.clone(),
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    ILLEGAL,
    EOF,
    IDENT,
    INT,
    ASSIGN,
    PLUS,
    COMMA,
    SEMICOLON,
    LPAREN,
    RPAREN,
    LBRACE,
    RBRACE,
    FUNCTION,
    LET,

    MINUS,
    BANG,
    ASTERISK,
    SLASH,
    LT,
    GT,

    TRUE,
    FALSE,
    IF,
    ELSE,
    RETURN,

    EQ,
    NOTEQ,

    STR,
    LBRACKET,
    RBRACKET,
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
