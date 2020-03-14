use super::token::*;

pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: u8,
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

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0
        } else {
            self.ch = self.input.as_bytes()[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        let tok: Token;

        self.skip_whitespace();

        match self.ch {
            b'=' => {
                if self.peek_char() == b'=' {
                    let ch = self.ch;
                    self.read_char();
                    tok = Token {
                        tk_type: TokenType::EQ,
                        literal: format!("{}{}", ch as char, self.ch as char),
                    };
                } else {
                    tok = new_token(TokenType::ASSIGN, self.ch);
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
                    let ch = self.ch;
                    self.read_char();
                    tok = Token {
                        tk_type: TokenType::NOTEQ,
                        literal: format!("{}{}", ch as char, self.ch as char),
                    };
                } else {
                    tok = new_token(TokenType::BANG, self.ch);
                }
            }
            b'*' => tok = new_token(TokenType::ASTERISK, self.ch),
            b'/' => tok = new_token(TokenType::SLASH, self.ch),
            b'<' => tok = new_token(TokenType::LT, self.ch),
            b'>' => tok = new_token(TokenType::GT, self.ch),
            0 => {
                tok = Token {
                    tk_type: TokenType::EOF,
                    literal: String::new(),
                }
            }
            _ => {
                if (self.ch as char).is_ascii_alphabetic() {
                    let literal = self.read_identifier();
                    tok = Token {
                        literal: literal.clone(),
                        tk_type: lookup_ident(&literal),
                    };
                    return tok;
                } else if (self.ch as char).is_ascii_digit() {
                    tok = Token {
                        tk_type: TokenType::INT,
                        literal: self.read_number(),
                    };
                    return tok;
                } else {
                    tok = new_token(TokenType::ILLEGAL, self.ch);
                }
            }
        }
        self.read_char();
        tok
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while (self.ch as char).is_ascii_alphabetic() {
            self.read_char();
        }
        String::from(&self.input[position..self.position])
    }

    fn skip_whitespace(&mut self) {
        while vec![b' ', b'\t', b'\n', b'\r'].contains(&self.ch) {
            self.read_char();
        }
    }

    fn read_number(&mut self) -> String {
        let position = self.position;
        while (self.ch as char).is_ascii_digit() {
            self.read_char()
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
}

pub fn new_token(token_type: TokenType, ch: u8) -> Token {
    let mut literal = String::new();
    literal.push(ch as char);
    Token {
        tk_type: token_type,
        literal: literal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_next_token1() {
        let input = "=+(){},;";
        let tests: [(TokenType, &str); 9] = [
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
                tok.tk_type == tt.0,
                "tests[{}] - tokentype wrong. expected={:?}, got={:?}",
                i,
                tt.0,
                tok.tk_type
            );
            assert!(
                tok.literal == tt.1,
                "tests[{}] - literal wrong. expected={}, got={}",
                i,
                tt.1,
                tok.literal
            );
        }
    }

    #[test]
    fn test_next_token2() {
        let input = r"
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
        let tests: [(TokenType, &str); 74] = [
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

        let mut l = Lexer::new(String::from(input));
        for (i, tt) in tests.iter().enumerate() {
            let tok = l.next_token();
            assert!(
                tok.tk_type == tt.0,
                "tests[{}] - tokentype wrong. expected={:?}, got={:?}",
                i,
                tt.0,
                tok.tk_type
            );
            assert!(
                tok.literal == tt.1,
                "tests[{}] - literal wrong. expected={}, got={}",
                i,
                tt.1,
                tok.literal
            );
        }
    }
}
