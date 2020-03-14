use super::ast::*;
use super::lexer::*;
use super::token::*;

pub enum Precedence {
    LOWEST,
    EQUALS,
    LESSGREATER,
    SUM,
    PRODUCT,
    PREFIX,
    CALL,
}

pub struct Parser {
    l: Lexer,
    cur_token: Token,
    peek_token: Token,
    errors: Vec<String>,
}
impl Parser {
    pub fn new(l: Lexer) -> Parser {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::ILLEGAL, 0),
            peek_token: new_token(TokenType::ILLEGAL, 0),
            errors: Vec::new(),
        };
        p.next_token();
        p.next_token();
        p
    }

    fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    pub fn parse_program(&mut self) -> Option<Node> {
        let mut stmts: Vec<Stmt> = Vec::new();
        while self.cur_token.tk_type != TokenType::EOF {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            self.next_token();
        }
        Some(Node::Program { stmts: stmts })
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.cur_token.tk_type {
            TokenType::LET => self.parse_let_stmt(),
            TokenType::RETURN => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::IDENT) {
            return None;
        }

        let name = Ident {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::ASSIGN) {
            return None;
        }

        while !self.cur_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Stmt::LetStmt {
            token: token,
            name: name,
            value: None,
        })
    }

    fn cur_token_is(&self, t: TokenType) -> bool {
        self.cur_token.tk_type == t
    }

    fn peek_token_is(&self, t: TokenType) -> bool {
        self.peek_token.tk_type == t
    }

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(t) {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }

    pub fn get_errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead.",
            t, self.peek_token.tk_type
        );
        self.errors.push(msg);
    }

    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        let token = self.cur_token.clone();
        self.next_token();
        while !self.cur_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }
        Some(Stmt::ReturnStmt {
            token: token,
            return_value: None,
        })
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        let token = self.cur_token.clone();
        let expr = self.parse_expr(Precedence::LOWEST);
        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }
        Some(Stmt::ExprStmt {
            token: token,
            expr: expr,
        })
    }

    fn parse_expr(&mut self, prec: Precedence) -> Option<Expr> {
        match self.cur_token.tk_type {
            TokenType::IDENT => self.parse_ident(),
            TokenType::INT => self.parse_integer_literal(),
            TokenType::BANG | TokenType::MINUS => self.parse_prefix_expr(),
            _ => {
                self.no_prefix_parse_fn_error(self.cur_token.tk_type);
                None
            }
        }
    }

    fn parse_ident(&mut self) -> Option<Expr> {
        Some(Expr::Ident(Ident {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&mut self) -> Option<Expr> {
        let token = self.cur_token.clone();

        match self.cur_token.literal.parse::<i64>() {
            Ok(value) => Some(Expr::IntegerLiteral(IntegerLiteral {
                token: token,
                value: value,
            })),
            _ => {
                let msg = format!("could not parse {} as integer", self.cur_token.literal);
                self.errors.push(msg);
                None
            }
        }
    }

    fn no_prefix_parse_fn_error(&mut self, t: TokenType) {
        let msg = format!("no prefix parse function for {:?} found", t);
        self.errors.push(msg);
    }

    fn parse_prefix_expr(&mut self) -> Option<Expr> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();

        self.next_token();

        let right = self.parse_expr(Precedence::PREFIX);

        Some(Expr::PrefixExpr(PrefixExpr {
            token: token,
            operator: operator,
            right: Box::new(right.unwrap()), //TODO
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_stmts() {
        let input = r"
let x = 5;
let y = 10;
let foobar = 838383;
";
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        match program {
            Some(Node::Program { stmts }) => {
                assert!(
                    stmts.len() == 3,
                    "program.stmts does not contain 3 stmts. got={}",
                    stmts.len(),
                );

                let tests = ["x", "y", "foobar"];

                for (i, tt) in tests.iter().enumerate() {
                    let stmt = &stmts[i];
                    test_let_statement(stmt, tt)
                }
            }
            _ => {
                assert!(false, "parse_program returned None");
            }
        }
    }

    fn test_let_statement(s: &Stmt, expected_name: &str) {
        assert!(
            s.token_literal() == "let",
            "s.token_literal not 'let'. got={}",
            s.token_literal()
        );

        match s {
            Stmt::LetStmt { token, name, value } => {
                assert!(
                    name.value == expected_name,
                    "letStmt.name.value not '{}'. got={}",
                    expected_name,
                    name.value
                );

                assert!(
                    name.token_literal() == expected_name,
                    "name not '{}', got={:?}",
                    expected_name,
                    name
                );
            }
            _ => assert!(false, "s not LetStatement. got={:?}", s),
        }
    }

    fn check_parser_errors(p: &mut Parser) {
        let errors = p.get_errors();
        if errors.len() == 0 {
            return;
        }

        let mut err = format!("parser has {} errors\n", errors.len());
        for msg in errors.iter() {
            err.push_str(&format!("parser error: {}\n", msg));
        }
        assert!(false, err);
    }

    #[test]
    fn test_return_stmts() {
        let input = r"
return 5;
return 10;
return 993322;
        ";
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);

        let program = p.parse_program();
        check_parser_errors(&mut p);

        match program {
            Some(Node::Program { stmts }) => {
                assert!(
                    stmts.len() == 3,
                    "program.stmts does not contain 3 stmts. got={}",
                    stmts.len()
                );

                for stmt in stmts.iter() {
                    match stmt {
                        Stmt::ReturnStmt {
                            token,
                            return_value,
                        } => {
                            assert!(
                                token.literal == "return",
                                "return_stmt.token_literal not 'return', got={}",
                                token.literal
                            );
                        }
                        _ => {
                            println!("stmt not ReturnStmt. got={:?}", stmt);
                        }
                    }
                }
            }
            _ => {
                assert!(false, "parse_program returned None");
            }
        }
    }

    #[test]
    fn test_indent_expr() {
        let input = "foobar;";
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);
        match program {
            Some(Node::Program { stmts }) => {
                assert!(
                    stmts.len() == 1,
                    "program has not enough stmts. got={}",
                    stmts.len()
                );

                match &stmts[0] {
                    Stmt::ExprStmt { token, expr } => match expr {
                        Some(Expr::Ident(ident)) => {
                            assert!(
                                ident.value == "foobar",
                                "ident.value not {}. got={}",
                                "foobar",
                                ident.value
                            );
                            assert!(
                                ident.token_literal() == "foobar",
                                "ident.token_literal not {}. got={}",
                                "foobar",
                                ident.token_literal()
                            );
                        }
                        _ => {}
                    },
                    _ => {
                        assert!(
                            false,
                            "program.stmts[0] is not ExprStmt. got={:?}",
                            stmts[0]
                        );
                    }
                }
            }
            _ => {
                assert!(false, "parse_program returned None");
            }
        }
    }

    #[test]
    fn test_integer_literal_expr() {
        let input = "5;";

        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        match program {
            Some(Node::Program { stmts }) => {
                assert!(
                    stmts.len() == 1,
                    "program has not enough statements. got={}",
                    stmts.len()
                );

                match &stmts[0] {
                    Stmt::ExprStmt { token, expr } => match expr {
                        Some(Expr::IntegerLiteral(literal)) => {
                            assert!(
                                literal.value == 5,
                                "literal.value not {}, got={}",
                                5,
                                literal.value
                            );
                            assert!(
                                literal.token_literal() == "5",
                                "literal.token_literal not {}. got={}",
                                "5",
                                literal.token_literal()
                            );
                        }
                        _ => {
                            assert!(false, "exp not IntegerLiteral. got={:?}", expr);
                        }
                    },
                    _ => {
                        assert!(
                            false,
                            "program.stmts[0] is not ExprStmt. got={:?}",
                            stmts[0]
                        );
                    }
                }
            }
            _ => {
                assert!(false, "program parse error");
            }
        }
    }

    #[test]
    fn test_parsing_prefix_expr() {
        let tests = [("!5;", "!", 5), ("-15;", "-", 15)];

        for tt in tests.iter() {
            let l = Lexer::new(String::from(tt.0));
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&mut p);

            match program {
                Some(Node::Program { stmts }) => {
                    assert!(
                        stmts.len() == 1,
                        "program.statements does not contain {} statements. got={}",
                        1,
                        stmts.len()
                    );

                    match &stmts[0] {
                        Stmt::ExprStmt { token, expr } => match expr {
                            Some(Expr::PrefixExpr(prefix_expr)) => match prefix_expr {
                                PrefixExpr {
                                    token,
                                    operator,
                                    right,
                                } => {
                                    assert!(
                                        operator == tt.1,
                                        "exp.operator is not '{}'. got={}",
                                        tt.1,
                                        operator
                                    );
                                    test_integer_literal(right, tt.2);
                                }
                                _ => {
                                    assert!(
                                        false,
                                        "stmt is not PrefixExpress. got={:?}",
                                        prefix_expr
                                    );
                                }
                            },
                            _ => {
                                assert!(false, "stmt is not PrefixExpr. got={:?}", expr);
                            }
                        },
                        _ => assert!(
                            false,
                            "program.stmts[0] is not ExprStmt. got={:?}",
                            stmts[0]
                        ),
                    }
                }
                _ => {
                    assert!(false, "program parse error");
                }
            }

            fn test_integer_literal(il: &Expr, expected_value: i64) {
                match il {
                    Expr::IntegerLiteral(integer_literal) => match integer_literal {
                        IntegerLiteral { token, value } => {
                            assert!(
                                *value == expected_value,
                                "value not {}. get={}",
                                expected_value,
                                value
                            );
                            assert!(
                                token.literal == expected_value.to_string(),
                                "token.literal not {}, got={}",
                                expected_value,
                                token.literal
                            );
                        }
                        _ => {
                            assert!(false, "il not IntegerLiteral. got={:?}", il);
                        }
                    },
                    _ => {
                        assert!(false, "error");
                    }
                }
            }
        }
    }
}
