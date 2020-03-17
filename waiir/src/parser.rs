use super::ast::*;
use super::lexer::*;
use super::token::*;

#[derive(PartialOrd, PartialEq)]
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
        println!("parse_program: {:?}", self.cur_token);
        let mut stmts: Vec<Stmt> = Vec::new();
        while self.cur_token.tk_type != TokenType::EOF {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            self.next_token();
        }
        Some(Node::Program(Program { stmts: stmts }))
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        println!("parse_stmt: {:?}", self.cur_token);
        match self.cur_token.tk_type {
            TokenType::LET => self.parse_let_stmt(),
            TokenType::RETURN => self.parse_return_stmt(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        println!("parse_let_stmt: {:?}", self.cur_token);
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

        self.next_token();

        let value = self.parse_expr(Precedence::LOWEST);

        while !self.cur_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Stmt::LetStmt {
            token: token,
            name: name,
            value: value.unwrap(), //TODO
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
        println!("parse_return_stmt: {:?}", self.cur_token);
        let token = self.cur_token.clone();
        self.next_token();
        let value = self.parse_expr(Precedence::LOWEST);
        while !self.cur_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(Stmt::ReturnStmt {
            token: token,
            value: value.unwrap(), //TODO
        })
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        println!("parse_expr_stmt: {:?}", self.cur_token);
        let token = self.cur_token.clone();
        let expr = self.parse_expr(Precedence::LOWEST);
        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }
        if expr.is_none() {
            return None;
        }
        Some(Stmt::ExprStmt {
            token: token,
            expr: expr.unwrap(),
        })
    }

    fn parse_expr(&mut self, precedence: Precedence) -> Option<Expr> {
        println!("parse_expr: {:?}", self.cur_token);
        let mut left_exp: Option<Expr>;
        match self.cur_token.tk_type {
            TokenType::IDENT => left_exp = self.parse_ident(),
            TokenType::INT => left_exp = self.parse_integer_literal(),
            TokenType::BANG | TokenType::MINUS => left_exp = self.parse_prefix_expr(),
            TokenType::TRUE | TokenType::FALSE => left_exp = self.parse_boolean(),
            TokenType::LPAREN => left_exp = self.parse_grouped_expr(),
            TokenType::IF => left_exp = self.parse_if_expr(),
            TokenType::FUNCTION => left_exp = self.parse_func_lite(),
            _ => {
                self.no_prefix_parse_fn_error(self.cur_token.tk_type);
                return None;
            }
        };

        while !self.peek_token_is(TokenType::SEMICOLON) && precedence < self.peek_precedence() {
            match self.peek_token.tk_type {
                TokenType::PLUS
                | TokenType::MINUS
                | TokenType::SLASH
                | TokenType::ASTERISK
                | TokenType::EQ
                | TokenType::NOTEQ
                | TokenType::LT
                | TokenType::GT => {
                    self.next_token();
                    left_exp = self.parse_infix_expr(left_exp.unwrap()); //TODO
                }
                TokenType::LPAREN => {
                    self.next_token();
                    left_exp = self.parse_call_expr(&left_exp.unwrap());
                }
                _ => {
                    return left_exp;
                }
            }
        }
        left_exp
    }

    fn parse_ident(&mut self) -> Option<Expr> {
        println!("parse_ident: {:?}", self.cur_token);
        Some(Expr::Ident(Ident {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }

    fn parse_integer_literal(&mut self) -> Option<Expr> {
        println!("parse_integer_literal: {:?}", self.cur_token);
        let token = self.cur_token.clone();

        match self.cur_token.literal.parse::<i64>() {
            Ok(value) => Some(Expr::IntLiteral(IntLiteral {
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
        println!("parse_prefix_expr: {:?}", self.cur_token);
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

    fn get_precedence(&self, token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::EQ | TokenType::NOTEQ => Precedence::EQUALS,
            TokenType::LT | TokenType::GT => Precedence::LESSGREATER,
            TokenType::PLUS | TokenType::MINUS => Precedence::SUM,
            TokenType::SLASH | TokenType::ASTERISK => Precedence::PRODUCT,
            TokenType::LPAREN => Precedence::CALL,
            _ => Precedence::LOWEST,
        }
    }

    fn peek_precedence(&self) -> Precedence {
        self.get_precedence(self.peek_token.tk_type)
    }
    fn cur_precedence(&self) -> Precedence {
        self.get_precedence(self.cur_token.tk_type)
    }

    fn parse_infix_expr(&mut self, left: Expr) -> Option<Expr> {
        println!("parse_infix_expr: {:?}", self.cur_token);
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expr(precedence);

        if right.is_none() {
            return None;
        }

        Some(Expr::InfixExpr(InfixExpr {
            token: token,
            operator: operator,
            left: Box::new(left),
            right: Box::new(right.unwrap()), //TODO
        }))
    }

    fn parse_boolean(&mut self) -> Option<Expr> {
        println!("parse_boolean: {:?}", self.cur_token);
        Some(Expr::Bool {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenType::TRUE),
        })
    }

    fn parse_grouped_expr(&mut self) -> Option<Expr> {
        println!("parse_grouped_expr: {:?}", self.cur_token);
        self.next_token();
        let exp = self.parse_expr(Precedence::LOWEST);
        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        exp
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
        println!("parse_if_expr: {:?}", self.cur_token);
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expr(Precedence::LOWEST);

        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }

        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let consequence = self.parse_block_stmt();

        let mut alternative: Option<BlockStmt> = None;

        if self.peek_token_is(TokenType::ELSE) {
            self.next_token();

            if !self.expect_peek(TokenType::LBRACE) {
                return None;
            }
            alternative = Some(self.parse_block_stmt());
        }

        Some(Expr::IfExpr {
            token: token,
            condition: Box::new(condition.unwrap()), //TODO
            consequence: consequence,
            alternative: alternative,
        })
    }

    fn parse_block_stmt(&mut self) -> BlockStmt {
        println!("parse_block_stmt: {:?}", self.cur_token);
        let token = self.cur_token.clone();
        let mut stmts: Vec<Stmt> = Vec::new();

        self.next_token();

        while !self.cur_token_is(TokenType::RBRACE) {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            }
            self.next_token();
        }
        BlockStmt {
            token: token,
            stmts: stmts,
        }
    }

    fn parse_func_lite(&mut self) -> Option<Expr> {
        println!("parse_func_lite: {:?}", self.cur_token);
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }
        let parameters = self.parse_func_parameters();
        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }
        let body = self.parse_block_stmt();
        Some(Expr::FuncLite {
            token: token,
            parameters: parameters,
            body: body,
        })
    }

    fn parse_func_parameters(&mut self) -> Vec<Ident> {
        println!("parse_func_parameters: {:?}", self.cur_token);
        let mut idents: Vec<Ident> = Vec::new();
        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return idents;
        }

        self.next_token();

        idents.push(Ident {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        });

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            idents.push(Ident {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            });
        }

        if !self.expect_peek(TokenType::RPAREN) {
            return Vec::new();
        }

        idents
    }

    fn parse_call_expr(&mut self, func: &Expr) -> Option<Expr> {
        let arguments = self.parse_call_arguments();

        Some(Expr::CallExpr {
            token: self.cur_token.clone(),
            func: Box::new(func.clone()),
            arguments: arguments.unwrap(), //TODO
        })
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<Expr>> {
        let mut args: Vec<Expr> = Vec::new();

        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return Some(args);
        }

        self.next_token();
        let arg = self.parse_expr(Precedence::LOWEST);

        args.push(arg.unwrap()); //TODO

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();

            let arg = self.parse_expr(Precedence::LOWEST);

            args.push(arg.unwrap()); //TODO
        }

        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }

        Some(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::*;

    #[test]
    fn test_let_stmts() {
        let tests: [(&str, &str, Box<dyn Any>); 3] = [
            ("let x = 5;", "x", Box::new(5 as i64)),
            ("let y = true;", "y", Box::new(true)),
            ("let foobar = y;", "foobar", Box::new("y")),
        ];
        for tt in tests.iter() {
            let l = Lexer::new(String::from(tt.0));
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&mut p);

            if let Some(Node::Program(Program { stmts })) = program {
                assert!(
                    stmts.len() == 1,
                    "program.stmts does not contain 1 stmts. got={}",
                    stmts.len(),
                );

                test_let_statement(&stmts[0], &*tt.1);

                if let Stmt::LetStmt {
                    token: _,
                    name: _,
                    value,
                } = &stmts[0]
                {
                    test_literal_expr(&value, &*tt.2);
                } else {
                    assert!(false, "stmt[0] is not LetStmt. got={:?}", &stmts[0]);
                }
            } else {
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

        if let Stmt::LetStmt {
            token: _,
            name,
            value: _,
        } = s
        {
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
        } else {
            assert!(false, "s not LetStmt. got={:?}", s);
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

        if let Some(Node::Program(Program { stmts })) = program {
            assert!(
                stmts.len() == 3,
                "program.stmts does not contain 3 stmts. got={}",
                stmts.len()
            );

            for stmt in stmts.iter() {
                if let Stmt::ReturnStmt { token, value: _ } = stmt {
                    assert!(
                        token.literal == "return",
                        "return_stmt.token_literal not 'return', got={}",
                        token.literal
                    );
                } else {
                    println!("stmt not ReturnStmt. got={:?}", stmt);
                }
            }
        } else {
            assert!(false, "parse_program returned None");
        }
    }

    #[test]
    fn test_indent_expr() {
        let input = "foobar;";
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Node::Program(Program { stmts })) = program {
            assert!(
                stmts.len() == 1,
                "program has not enough stmts. got={}",
                stmts.len()
            );

            if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                if let Expr::Ident(ident) = expr {
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
                } else {
                    assert!(
                        false,
                        "program.stmts[0] is not ExprStmt. got={:?}",
                        stmts[0]
                    );
                }
            } else {
                assert!(
                    false,
                    "program.stmts[0] is not ExprStmt. got={:?}",
                    stmts[0]
                );
            }
        } else {
            assert!(false, "parse_program returned None");
        }
    }

    #[test]
    fn test_integer_literal_expr() {
        let input = "5;";

        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Node::Program(Program { stmts })) = program {
            assert!(
                stmts.len() == 1,
                "program has not enough statements. got={}",
                stmts.len()
            );

            if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                if let Expr::IntLiteral(literal) = expr {
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
                } else {
                    assert!(false, "exp not IntLiteral. got={:?}", expr);
                }
            } else {
                assert!(
                    false,
                    "program.stmts[0] is not ExprStmt. got={:?}",
                    stmts[0]
                );
            }
        } else {
            assert!(false, "program parse error");
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

            if let Some(Node::Program(Program { stmts })) = program {
                assert!(
                    stmts.len() == 1,
                    "program.statements does not contain {} statements. got={}",
                    1,
                    stmts.len()
                );

                if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                    if let Expr::PrefixExpr(prefix_expr) = expr {
                        let PrefixExpr {
                            token: _,
                            operator,
                            right,
                        } = prefix_expr;

                        assert!(
                            operator == tt.1,
                            "exp.operator is not '{}'. got={}",
                            tt.1,
                            operator
                        );
                        test_integer_literal(right, tt.2);
                    } else {
                        assert!(false, "stmt is not PrefixExpr. got={:?}", expr);
                    }
                } else {
                    assert!(
                        false,
                        "program.stmts[0] is not ExprStmt. got={:?}",
                        stmts[0]
                    );
                }
            } else {
                assert!(false, "program parse error");
            }
        }
    }

    fn test_integer_literal(il: &Expr, expected_value: i64) {
        if let Expr::IntLiteral(IntLiteral { token, value }) = il {
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
        } else {
            assert!(false, "il not IntLiteral. got={:?}", il);
        }
    }

    #[test]
    fn test_parsing_infix_expr() {
        let tests: [(&str, Box<dyn Any>, &str, Box<dyn Any>); 11] = [
            ("5 + 5;", Box::new(5 as i64), "+", Box::new(5 as i64)),
            ("5 - 5;", Box::new(5 as i64), "-", Box::new(5 as i64)),
            ("5 * 5;", Box::new(5 as i64), "*", Box::new(5 as i64)),
            ("5 / 5;", Box::new(5 as i64), "/", Box::new(5 as i64)),
            ("5 > 5;", Box::new(5 as i64), ">", Box::new(5 as i64)),
            ("5 < 5;", Box::new(5 as i64), "<", Box::new(5 as i64)),
            ("5 == 5;", Box::new(5 as i64), "==", Box::new(5 as i64)),
            ("5 != 5;", Box::new(5 as i64), "!=", Box::new(5 as i64)),
            ("true == true", Box::new(true), "==", Box::new(true)),
            ("true != false", Box::new(true), "!=", Box::new(false)),
            ("false == false", Box::new(false), "==", Box::new(false)),
        ];

        for tt in tests.iter() {
            let l = Lexer::new(String::from(tt.0));
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&mut p);

            if let Some(Node::Program(Program { stmts })) = program {
                assert!(
                    stmts.len() == 1,
                    "program.stmts does not contain {} statements. got={}",
                    1,
                    stmts.len()
                );
                if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                    if let Expr::InfixExpr(infix_expr) = expr {
                        test_literal_expr(&infix_expr.left, &*tt.1);
                        assert!(
                            infix_expr.operator == tt.2,
                            "exp.operator is not '{}', got={}",
                            tt.2,
                            infix_expr.operator
                        );
                        test_literal_expr(&infix_expr.right, &*tt.3);
                    } else {
                        assert!(false, "exp is not InfixExpr. got={:?}", expr);
                    }
                } else {
                    assert!(
                        false,
                        "program.stmts[0] is not ExprStmt. got={:?}",
                        stmts[0]
                    );
                }
            } else {
                assert!(false, "program parse error");
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = [
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("true", "true"),
            ("false", "false"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
            ("a + add(b * c) + d", "((a + add((b * c))) + d)"),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            ),
            (
                "add(a + b + c * d / f  + g)",
                "add((((a + b) + ((c * d) / f)) + g))",
            ),
        ];
        for tt in tests.iter() {
            let l = Lexer::new(String::from(tt.0));
            let mut p = Parser::new(l);
            let program = p.parse_program();
            check_parser_errors(&mut p);

            let actual = program.unwrap().string();
            assert!(actual == tt.1, "expected={}, got={}", tt.1, actual);
        }
    }

    fn test_literal_expr(exp: &Expr, expected: &dyn Any) {
        if let Some(v) = expected.downcast_ref::<i64>() {
            test_integer_literal(&exp, *v);
        } else if let Some(v) = expected.downcast_ref::<&str>() {
            test_ident(exp, &v);
        } else if let Some(v) = expected.downcast_ref::<bool>() {
            test_boolean_literal(exp, *v);
        } else {
            assert!(false, "type of exp not handled. got={:?}", exp);
        }
    }

    fn test_ident(exp: &Expr, value: &str) {
        if let Expr::Ident(ident) = exp {
            assert!(
                ident.value == value,
                "ident.value not {}. got={}",
                value,
                ident.value
            );

            assert!(
                ident.token_literal() == value,
                "ident.token_literal not {}. got={}",
                value,
                ident.token_literal()
            );
        } else {
            assert!(false, "exp not Ident. got={:?}", exp);
        }
    }

    fn test_boolean_literal(exp: &Expr, expected_value: bool) {
        if let Expr::Bool { token, value } = exp {
            assert!(
                *value == expected_value,
                "bo.value not {}. got={}",
                expected_value,
                value
            );
            assert!(
                token.literal == format!("{}", expected_value),
                "bo.token_literal not {}, got={}",
                expected_value,
                token.literal
            );
        } else {
            assert!(false, "exp not Bool. got={:?}", exp);
        }
    }

    #[test]
    fn test_if_expr() {
        let input = "if (x < y) { x }";

        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Node::Program(Program { stmts })) = program {
            assert!(
                stmts.len() == 1,
                "program.body does not contain {} statments. got={}",
                1,
                stmts.len()
            );

            if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                if let Expr::IfExpr {
                    token: _,
                    condition,
                    consequence,
                    alternative,
                } = expr
                {
                    test_infix_expr(condition, &*Box::new("x"), "<", &*Box::new("y"));

                    assert!(
                        consequence.stmts.len() == 1,
                        "consequence is not 1 statements. got={}",
                        consequence.stmts.len(),
                    );

                    if let Stmt::ExprStmt { token: _, expr } = &consequence.stmts[0] {
                        test_ident(expr, "x");
                    } else {
                        assert!(
                            false,
                            "consequence.stmts[0] is not ExprStmt. got={:?}",
                            &consequence.stmts[0]
                        );
                    }
                    assert!(
                        alternative.is_none(),
                        "alterntive was not None. got={:?}",
                        alternative.as_ref().unwrap(),
                    );
                } else {
                    assert!(false, "stmt.expr is not IfExpr. got={:?}", expr);
                }
            } else {
                assert!(
                    false,
                    "program.stmts[0] is not ExprStmt. got={:?}",
                    &stmts[0]
                );
            }
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn test_if_else_expr() {
        let input = "if (x < y) { x } else { y }";

        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Node::Program(Program { stmts })) = program {
            assert!(
                stmts.len() == 1,
                "program.body does not contain {} statments. got={}",
                1,
                stmts.len()
            );

            if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                if let Expr::IfExpr {
                    token: _,
                    condition,
                    consequence,
                    alternative,
                } = expr
                {
                    test_infix_expr(condition, &*Box::new("x"), "<", &*Box::new("y"));

                    assert!(
                        consequence.stmts.len() == 1,
                        "consequence is not 1 statements. got={}",
                        consequence.stmts.len(),
                    );

                    if let Stmt::ExprStmt { token: _, expr } = &consequence.stmts[0] {
                        test_ident(expr, "x");
                    } else {
                        assert!(
                            false,
                            "consequence.stmts[0] is not ExprStmt. got={:?}",
                            &consequence.stmts[0]
                        );
                    }
                    if let Some(a) = alternative {
                        assert!(
                            a.stmts.len() == 1,
                            "alternative is not 1 statements. got={}",
                            a.stmts.len()
                        );

                        if let Stmt::ExprStmt { token: _, expr } = &a.stmts[0] {
                            test_ident(expr, "y");
                        } else {
                            assert!(
                                false,
                                "alternative.stmts[0] is not ExprStmt. got={:?}",
                                &a.stmts[0]
                            );
                        }
                    } else {
                        assert!(false, "alterntive was None");
                    }
                } else {
                    assert!(false, "stmt.expr is not IfExpr. got={:?}", expr);
                }
            } else {
                assert!(
                    false,
                    "program.stmts[0] is not ExprStmt. got={:?}",
                    &stmts[0]
                );
            }
        } else {
            assert!(false, "parse error");
        }
    }

    fn test_infix_expr(
        exp: &Expr,
        expected_left: &dyn Any,
        expected_operator: &str,
        expected_right: &dyn Any,
    ) {
        if let Expr::InfixExpr(infix_expr) = exp {
            let InfixExpr {
                token: _,
                left,
                operator,
                right,
            } = infix_expr;

            test_literal_expr(left, expected_left);

            assert!(
                operator == expected_operator,
                "operator is not '{}', got={}",
                expected_operator,
                operator
            );

            test_literal_expr(right, expected_right);
        } else {
            assert!(false, "exp is not InfixExpr. got={:?}", exp);
        }
    }

    #[test]
    fn test_func_lite_parsing() {
        let input = "fn(x, y) { x + y; }";
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Node::Program(Program { stmts })) = program {
            assert!(
                stmts.len() == 1,
                "program.body does not contain {} statements. got={}",
                1,
                stmts.len()
            );

            if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                if let Expr::FuncLite {
                    token: _,
                    parameters,
                    body,
                } = expr
                {
                    assert!(
                        parameters.len() == 2,
                        "func lite parameters wrong. want 2, got={}",
                        parameters.len()
                    );

                    test_literal_expr(
                        &Expr::Ident(Ident {
                            token: Token {
                                tk_type: TokenType::IDENT,
                                literal: parameters[0].value.clone(),
                            },
                            value: parameters[0].value.clone(),
                        }),
                        &*Box::new("x"),
                    );
                    test_literal_expr(
                        &Expr::Ident(Ident {
                            token: Token {
                                tk_type: TokenType::IDENT,
                                literal: parameters[1].value.clone(),
                            },
                            value: parameters[1].value.clone(),
                        }),
                        &*Box::new("y"),
                    );

                    assert!(
                        body.stmts.len() == 1,
                        "function.body.stmts has not 1 statements. got={}",
                        body.stmts.len()
                    );
                    if let Stmt::ExprStmt { token: _, expr } = &body.stmts[0] {
                        test_infix_expr(expr, &*Box::new("x"), "+", &*Box::new("y"));
                    } else {
                        assert!(false, "body stmt is not ExprStmt, got={:?}", &body.stmts[0]);
                    }
                } else {
                    assert!(false, "stmt.expr is not FuncLite. got={:?}", &expr);
                }
            } else {
                assert!(
                    false,
                    "program.stmts[0] is not ExprStmt. got={:?}",
                    &stmts[0]
                );
            }
        } else {
            assert!(false, "parse error");
        }
    }

    #[test]
    fn test_call_expr_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);";
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Node::Program(Program { stmts })) = program {
            assert!(
                stmts.len() == 1,
                "program.stmts does not contain {} statements. got={}",
                1,
                stmts.len()
            );

            if let Stmt::ExprStmt { token: _, expr } = &stmts[0] {
                if let Expr::CallExpr {
                    token: _,
                    func,
                    arguments,
                } = expr
                {
                    test_ident(&func, "add");

                    assert!(
                        arguments.len() == 3,
                        "wrong length of arguments. got={}",
                        arguments.len()
                    );

                    test_literal_expr(&arguments[0], &*Box::new(1 as i64));
                    test_infix_expr(
                        &arguments[1],
                        &*Box::new(2 as i64),
                        "*",
                        &*Box::new(3 as i64),
                    );
                    test_infix_expr(
                        &arguments[2],
                        &*Box::new(4 as i64),
                        "+",
                        &*Box::new(5 as i64),
                    );
                } else {
                    assert!(false, "stmt.expr is not CallExpr. got={:?}", expr);
                }
            } else {
                assert!(false, "stmt is not ExprStmt. got={:?}", stmts[0])
            }
        } else {
            assert!(false, "parse error",);
        }
    }
}
