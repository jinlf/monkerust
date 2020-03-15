use super::ast::*;
use super::lexer::*;
use super::token::*;
use std::any::*;
use std::ops::*;

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
            value: Expr::MockExpr {}, // TODO
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
            return_value: Expr::MockExpr {},
        })
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
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
        let mut left_exp: Option<Expr>;
        match self.cur_token.tk_type {
            TokenType::IDENT => left_exp = self.parse_ident(),
            TokenType::INT => left_exp = self.parse_integer_literal(),
            TokenType::BANG | TokenType::MINUS => left_exp = self.parse_prefix_expr(),
            TokenType::TRUE | TokenType::FALSE => left_exp = self.parse_boolean(),
            TokenType::IF => left_exp = self.parse_if_expr(),
            TokenType::LPAREN => left_exp = self.parse_grouped_expr(),
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
                _ => {
                    return left_exp;
                }
            }
        }
        left_exp
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

    fn get_precedence(&self, token_type: TokenType) -> Precedence {
        match token_type {
            TokenType::EQ | TokenType::NOTEQ => Precedence::EQUALS,
            TokenType::LT | TokenType::GT => Precedence::LESSGREATER,
            TokenType::PLUS | TokenType::MINUS => Precedence::SUM,
            TokenType::SLASH | TokenType::ASTERISK => Precedence::PRODUCT,
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
        Some(Expr::Boolean(Boolean {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenType::TRUE),
        }))
    }

    fn parse_grouped_expr(&mut self) -> Option<Expr> {
        self.next_token();
        let exp = self.parse_expr(Precedence::LOWEST);
        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        exp
    }

    fn parse_if_expr(&mut self) -> Option<Expr> {
        let token = self.cur_token.clone();

        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expr(Precedence::LOWEST);

        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let consequence = self.parse_block_stmt();

        let alternative = None; //TODO

        Some(Expr::IfExpr {
            token: token,
            condition: Box::new(condition.unwrap()), //TODO
            consequence: consequence,
            alternative: alternative,
        })
    }

    fn parse_block_stmt(&mut self) -> BlockStmt {
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
                        Expr::Ident(ident) => {
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
                        Expr::IntegerLiteral(literal) => {
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
                            Expr::PrefixExpr(prefix_expr) => match prefix_expr {
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

            match program.unwrap() {
                Node::Program { stmts } => {
                    assert!(
                        stmts.len() == 1,
                        "program.stmts does not contain {} statements. got={}",
                        1,
                        stmts.len()
                    );
                    match &stmts[0] {
                        Stmt::ExprStmt { token, expr } => match expr {
                            Expr::InfixExpr(infix_expr) => {
                                test_literal_expr(&infix_expr.left, &*tt.1);
                                assert!(
                                    infix_expr.operator == tt.2,
                                    "exp.operator is not '{}', got={}",
                                    tt.2,
                                    infix_expr.operator
                                );
                                test_literal_expr(&infix_expr.right, &*tt.3);
                            }
                            _ => {
                                assert!(false, "exp is not InfixExpr. got={:?}", expr);
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
        } else if let Some(v) = expected.downcast_ref::<String>() {
            test_ident(exp, &v);
        } else if let Some(v) = expected.downcast_ref::<bool>() {
            test_boolean_literal(exp, *v);
        } else {
            assert!(false, "type of exp not handled. got={:?}", exp);
        }
    }

    fn test_ident(exp: &Expr, value: &str) {
        match exp {
            Expr::Ident(ident) => {
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
            }
            _ => {
                assert!(false, "exp not Ident. got={:?}", exp);
            }
        }
    }

    fn test_boolean_literal(exp: &Expr, value: bool) {
        match exp {
            Expr::Boolean(bo) => {
                assert!(
                    bo.value == value,
                    "bo.value not {}. got={}",
                    value,
                    bo.value
                );
                assert!(
                    bo.token_literal() == format!("{}", value),
                    "bo.token_literal not {}, got={}",
                    value,
                    bo.token_literal()
                );
            }
            _ => {
                assert!(false, "exp not Boolean. got={:?}", exp);
            }
        }
    }

    #[test]
    fn test_if_expr() {
        let input = "if (x < y) { x }";

        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        match program.unwrap() {
            Node::Program { stmts } => {
                assert!(
                    stmts.len() == 1,
                    "program.body does not contain {} statments. got={}",
                    1,
                    stmts.len()
                );

                match &stmts[0] {
                    Stmt::ExprStmt { token, expr } => match expr {
                        Expr::IfExpr {
                            token,
                            condition,
                            consequence,
                            alternative,
                        } => {
                            test_infix_expr(condition, &*Box::new("x"), "<", &*Box::new("y"));

                            assert!(
                                consequence.stmts.len() == 1,
                                "consequence is not 1 statements. got={}",
                                consequence.stmts.len(),
                            );

                            match &consequence.stmts[0] {
                                Stmt::ExprStmt { token, expr } => {
                                    test_ident(expr, "x");
                                }
                                _ => {
                                    assert!(
                                        false,
                                        "consequence.stmts[0] is not ExprStmt. got={:?}",
                                        &consequence.stmts[0]
                                    );
                                }
                            }

                            assert!(
                                alternative.is_none(),
                                "alterntive was not None. got={:?}",
                                alternative.as_ref().unwrap(),
                            );
                        }
                        _ => {
                            assert!(false, "stmt.expr is not IfExpr. got={:?}", expr);
                        }
                    },
                    _ => {
                        assert!(
                            false,
                            "program.stmts[0] is not ExprStmt. got={:?}",
                            &stmts[0]
                        );
                    }
                }
            }
            _ => {
                assert!(false, "parse error");
            }
        }
    }

    #[test]
    fn test_if_else_expr() {
        let input = "if (x < y) { x } else { y }";

        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        match program.unwrap() {
            Node::Program { stmts } => {
                assert!(
                    stmts.len() == 1,
                    "program.body does not contain {} statments. got={}",
                    1,
                    stmts.len()
                );

                match &stmts[0] {
                    Stmt::ExprStmt { token, expr } => match expr {
                        Expr::IfExpr {
                            token,
                            condition,
                            consequence,
                            alternative,
                        } => {
                            test_infix_expr(condition, &*Box::new("x"), "<", &*Box::new("y"));

                            assert!(
                                consequence.stmts.len() == 1,
                                "consequence is not 1 statements. got={}",
                                consequence.stmts.len(),
                            );

                            match &consequence.stmts[0] {
                                Stmt::ExprStmt { token, expr } => {
                                    test_ident(expr, "x");
                                }
                                _ => {
                                    assert!(
                                        false,
                                        "consequence.stmts[0] is not ExprStmt. got={:?}",
                                        &consequence.stmts[0]
                                    );
                                }
                            }
                            match alternative {
                                Some(a) => {
                                    assert!(
                                        a.stmts.len() == 1,
                                        "alternative is not 1 statements. got={}",
                                        a.stmts.len()
                                    );

                                    match &a.stmts[0] {
                                        Stmt::ExprStmt { token, expr } => {
                                            test_ident(expr, "y");
                                        }
                                        _ => {
                                            assert!(
                                                false,
                                                "alternative.stmts[0] is not ExprStmt. got={:?}",
                                                &a.stmts[0]
                                            );
                                        }
                                    }
                                }
                                _ => {
                                    assert!(false, "alterntive was None");
                                }
                            }
                        }
                        _ => {
                            assert!(false, "stmt.expr is not IfExpr. got={:?}", expr);
                        }
                    },
                    _ => {
                        assert!(
                            false,
                            "program.stmts[0] is not ExprStmt. got={:?}",
                            &stmts[0]
                        );
                    }
                }
            }
            _ => {
                assert!(false, "parse error");
            }
        }
    }

    fn test_infix_expr(
        exp: &Expr,
        expected_left: &dyn Any,
        expected_operator: &str,
        expected_right: &dyn Any,
    ) {
        match exp {
            Expr::InfixExpr(infix_expr) => match infix_expr {
                InfixExpr {
                    token,
                    left,
                    operator,
                    right,
                } => {
                    test_literal_expr(left, expected_left);

                    assert!(
                        operator == expected_operator,
                        "operator is not '{}', got={}",
                        expected_operator,
                        operator
                    );

                    test_literal_expr(right, expected_right);
                }
                _ => {
                    assert!(false, "error");
                }
            },
            _ => {
                assert!(false, "exp is not InfixExpr. got={:?}", exp);
            }
        }
    }

    #[test]
    fn test_func_lite_parsing() {
        let input = "fn(x, y) { x + y; }";
        let l = Lexer::new(String::from(input));
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        match program.unwrap() {
            Node::Program { stmts } => {
                assert!(
                    stmts.len() == 1,
                    "program.body does not contain {} statements. got={}",
                    1,
                    stmts.len()
                );

                match &stmts[0] {
                    Stmt::ExprStmt { token, expr } => match expr {
                        Expr::FuncLite {
                            token,
                            parameters,
                            body,
                        } => {
                            assert!(
                                parameters.len() == 2,
                                "func lite parameters wrong. want 2, got={}",
                                parameters.len()
                            );

                            test_literal_expr(
                                &Expr::Ident(Ident {
                                    token: Token {
                                        tk_type: TokenType::IDENT,
                                        literal: parameters[0].value,
                                    },
                                    value: parameters[0].value,
                                }),
                                &*Box::new("x"),
                            );
                            test_literal_expr(
                                &Expr::Ident(Ident {
                                    token: Token {
                                        tk_type: TokenType::IDENT,
                                        literal: parameters[1].value,
                                    },
                                    value: parameters[1].value,
                                }),
                                &*Box::new("y"));
                            };

                            assert!(
                                body.stmts.len() == 1,
                                "function.body.stmts has not 1 statements. got={}",
                                body.stmts.len()
                            );
                            match &body.stmts[0] {
                                Stmt::ExprStmt { token, expr } => {
                                    test_infix_expr(expr, &*Box::new("x"), "+", &*Box::new("y"));
                                }
                                _ => {
                                    assert!(
                                        false,
                                        "body stmt is not ExprStmt, got={:?}",
                                        &body.stmts[0]
                                    );
                                }
                            }
                        }
                        _ => {
                            assert!(false, "stmt.expr is not FuncLite. got={:?}", &expr);
                        }
                    },
                    _ => {
                        assert!(
                            false,
                            "program.stmts[0] is not ExprStmt. got={:?}",
                            &stmts[0]
                        );
                    }
                }
            }
            _ => {
                assert!(false, "parse error");
            }
        }
    }
}
