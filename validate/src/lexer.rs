// src/lexer.rs

use super::token::*;

pub struct Lexer<'a> {
    input: &'a str,
    position: usize,      // current position in input (points to current char)
    read_position: usize, // current reading position in input (after current char)
    ch: u8,               // current char under examination
}
impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer<'a> {
        let mut l = Lexer {
            input: input,
            position: 0,
            read_position: 0,
            ch: 0,
        };
        l.read_char();
        l
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input.bytes().nth(self.read_position).unwrap();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        let tok: Token;

        self.skip_whitespace();

        match self.ch {
            b':' => tok = new_token(TokenType::COLON, self.ch),
            b'=' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    tok = Token {
                        tk_type: TokenType::EQ,
                        literal: String::from("=="),
                    }
                } else {
                    tok = new_token(TokenType::ASSIGN, self.ch)
                }
            }
            b';' => tok = new_token(TokenType::SEMICOLON, self.ch),
            b'(' => tok = new_token(TokenType::LPAREN, self.ch),
            b')' => tok = new_token(TokenType::RPAREN, self.ch),
            b',' => tok = new_token(TokenType::COMMA, self.ch),
            b'+' => tok = new_token(TokenType::PLUS, self.ch),
            b'{' => tok = new_token(TokenType::LBRACE, self.ch),
            b'}' => tok = new_token(TokenType::RBRACE, self.ch),
            b'-' => tok = new_token(TokenType::MINUS, self.ch),
            b'!' => {
                if self.peek_char() == b'=' {
                    self.read_char();
                    tok = Token {
                        tk_type: TokenType::NOTEQ,
                        literal: String::from("!="),
                    }
                } else {
                    tok = new_token(TokenType::BANG, self.ch)
                }
            }
            b'/' => tok = new_token(TokenType::SLASH, self.ch),
            b'*' => tok = new_token(TokenType::ASTERISK, self.ch),
            b'<' => tok = new_token(TokenType::LT, self.ch),
            b'>' => tok = new_token(TokenType::GT, self.ch),
            b'"' => {
                tok = Token {
                    tk_type: TokenType::STRING,
                    literal: self.read_string(),
                }
            }
            b'[' => tok = new_token(TokenType::LBRACKET, self.ch),
            b']' => tok = new_token(TokenType::RBRACKET, self.ch),
            0 => {
                tok = Token {
                    tk_type: TokenType::EOF,
                    literal: String::new(),
                }
            }
            _ => {
                if is_letter(self.ch) {
                    let literal = self.read_identifier();
                    tok = Token {
                        tk_type: lookup_ident(&literal),
                        literal: literal,
                    };
                    return tok;
                } else if self.ch.is_ascii_digit() {
                    tok = Token {
                        tk_type: TokenType::INT,
                        literal: self.read_number(),
                    };
                    return tok;
                }
                tok = new_token(TokenType::ILLEGAL, self.ch);
            }
        }
        self.read_char();
        tok
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }
        String::from(&self.input[position..self.position])
    }


    fn skip_whitespace(&mut self) {
        loop {
            match self.ch {
                b' ' | b'\t' | b'\n' | b'\r' => self.read_char(),
                _ => return,
            }
        }
    }

    fn read_number(&mut self) -> String {
        let position = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }
        String::from(&self.input[position..self.position])
    }

    fn peek_char(&mut self) -> u8 {
        if self.read_position >= self.input.len() {
            return 0;
        } else {
            return self.input.bytes().nth(self.read_position).unwrap();
        }
    }

    fn read_string(&mut self) -> String {
        let position = self.position + 1;
        loop {
            self.read_char();
            if self.ch == b'"' {
                break;
            }
        }
        String::from(&self.input[position..self.position])
    }
}

pub fn new_token(token_type: TokenType, ch: u8) -> Token {
    let mut literal = String::new();
    literal.push(ch as char);
    Token {
        tk_type: token_type,
        literal: literal,
    }
}

fn is_letter(ch: u8) -> bool {
    ch.is_ascii_alphabetic() || ch == b'_'
}

