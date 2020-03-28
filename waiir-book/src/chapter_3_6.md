# 解析表达式


## Monkey语言表达式


## 自顶向下运算法优先级


## 术语

## 准备AST

```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Expression,
}
impl NodeTrait for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}
```
加入Statement：
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
    ExpressionStatement(ExpressionStatement),
}
impl NodeTrait for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.token_literal(),
            Statement::ReturnStatement(return_stmt) => return_stmt.token_literal(),
            Statement::ExpressionStatement(expr_stmt) => expr_stmt.token_literal(),
        }
    }
}
```

给NodeTrait增加string函数，并通过补写函数解决由此带来的一系列问题。
```rust,noplaypen
// src/ats.rs

pub trait NodeTrait {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}

impl NodeTrait for Statement {
// [...]
    fn string(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.string(),
            Statement::ReturnStatement(return_stmt) => return_stmt.string(),
            Statement::ExpressionStatement(expr_stmt) => expr_stmt.string(),
        }
    }
}

impl NodeTrait for Expression {
// [...]
    fn string(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.string(),
            Expression::MockExpression { v: _ } => String::new(), //TODO remove
        }
    }
}

impl NodeTrait for Program {
// [...]
    fn string(&self) -> String {
        let mut out = String::new();
        for s in self.statements.iter() {
            out.push_str(&s.string());
        }
        out
    }
}

impl NodeTrait for LetStatement {
// [...]
    fn string(&self) -> String {
        format!(
            "{} {} = {};",
            self.token_literal(),
            self.name.string(),
            self.value.string(),
        )
    }
}

impl NodeTrait for Identifier {
// [...]
    fn string(&self) -> String {
        self.value.clone()
    }
}

impl NodeTrait for ReturnStatement {
// [...]
    fn string(&self) -> String {
        format!("{} {};", self.token_literal(), self.return_value.string())
    }
}

impl NodeTrait for ExpressionStatement {
// [...]
    fn string(&self) -> String {
        self.expression.string()
    }
}
```
下面写AST的一个测试用例：
```rust,noplaypen
// src/ast_test.rs

use super::ast::*;
use super::token::*;

#[test]
fn test_string() {
    let program = Program {
        statements: vec![Statement::LetStatement(LetStatement {
            token: Token {
                tk_type: TokenType::LET,
                literal: String::from("let"),
            },
            name: Identifier {
                token: Token {
                    tk_type: TokenType::IDENT,
                    literal: String::from("myVar"),
                },
                value: String::from("myVar"),
            },
            value: Expression::Identifier(Identifier {
                token: Token {
                    tk_type: TokenType::IDENT,
                    literal: String::from("anotherVar"),
                },
                value: String::from("anotherVar"),
            }),
        })],
    };

    assert!(
        program.string() == "let myVar = anotherVar;",
        "program.string() wrong. got={}",
        program.string()
    );
}
```
在lib.rs中加入
```rust,noplaypen
// src/lib.rs

mod ast_test;
```
测试通过！

## 实现递归下降解析器

原著中实现的是Pratt词法分析器，即用语义代码关联到Token类型上。我用Rust实现类似的方式时，发现Rust中可变方法指针的使用不是很方便，考虑原因应该是这种灵活的写法容易带来内存管理方面的不安全，Rust不推荐也不适用这种方式。于是本书中使用最基本的递归下降解析方法。

先写标识符的测试用例：
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_identifier_expression() {
    let input = "foobar;";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 1,
            "program has not enough statements. got={}",
            statements.len()
        );
        if let Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        }) = &statements[0]
        {
            if let Expression::Identifier(Identifier { token, value }) = expression {
                assert!(
                    value == "foobar",
                    "ident.value not {}. got={}",
                    "foobar",
                    value
                );
                assert!(
                    token.literal == "foobar",
                    "ident.token_literal not {}. got={}",
                    "foobar",
                    token.literal
                );
            } else {
                assert!(false, "exp not Identifier. got={:?}", expression);
            }
        } else {
            assert!(
                false,
                "program.statements[0] is not ExpressionStatement. got={:?}",
                &statements[0]
            );
        }
    } else {
        assert!(false, "parse error");
    }
}
```
测试失败：
```
thread 'parser::tests::test_identifier_expression' panicked at 'program has not enough statements. got=0', src/parser_test.rs:257:13
```

修改parse_statement方法：
```rust,noplaypen
// src/parse.rs

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.tk_type {
// [...]
            _ => {
                if let Some(stmt) = self.parse_expression_statement() {
                    return Some(Statement::ExpressionStatement(stmt));
                }
                None
            }
        }
    }
```
增加parse_expression_statement方法
```rust,noplaypen
// src/parse.rs

    fn parse_expression_statement(&mut self) -> Option<ExpressionStatement> {
        let token = self.cur_token.clone();
        let expression = self.parse_expression(Precedence::LOWEST);
        if expression.is_none() {
            return None;
        }
        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }
        Some(ExpressionStatement {
            token: token,
            expression: expression.unwrap(),
        })
    }
```
这里需要考虑运算符的优先级了，定义如下：
```rust,noplaypen
// src/parser.rs

pub enum Precedence {
    LOWEST,
    EQUALS,      // ==
    LESSGREATER, // > or <
    SUM,         // +
    PRODUCT,     // *
    PREFIX,      // -x or !x
    CALL,        // myFunction(X)
}
```
增加如下方法的实现：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let left_exp: Option<Expression>;
        match self.cur_token.tk_type {
            TokenType::IDENT => left_exp = self.parse_identifier(),
            _ => return None,
        }
        left_exp
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        Some(Expression::Identifier(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }
```
测试通过！

## 整型字面量

测试用例如下：
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_integer_literal_expression() {
    let input = "5;";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 1,
            "program has not enough statements. got={}",
            statements.len()
        );

        if let Statement::ExpressionStatement(ExpressionStatement { token, expression }) =
            &statements[0]
        {
            if let Expression::IntegerLiteral(IntegerLiteral { token, value }) = expression {
                assert!(*value == 5, "literal.value not {}. got={}", 5, value);
                assert!(
                    token.literal == "5",
                    "literal.token_literal not {}. got={}",
                    "5",
                    token.literal
                );
            } else {
                assert!(false, "exp not IntegerLiteral. got={:?}", expression);
            }
        } else {
            assert!(
                false,
                "program.statements[0] is not ExpressionStatement. got={:?}",
                &statements[0]
            );
        }
    } else {
        assert!(false, "parse error");
    }
}
```
IntegerLiteral的定义在ast.rs中：
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}
impl NodeTrait for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug)]
pub enum Expression {
// [...]
    IntegerLiteral(IntegerLiteral),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::IntegerLiteral(integer_literal) => integer_literal.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::IntegerLiteral(integer_literal) => integer_literal.string(),
        }
    }
}
```
新增加IntegerLiteral的解析代码：
```rust,noplaypen
// src/parser.rs

    fn parse_integer_literal(&mut self) -> Option<Expression> {
        if let Ok(value) = self.cur_token.literal.parse::<i64>() {
            Some(Expression::IntegerLiteral(IntegerLiteral {
                token: self.cur_token.clone(),
                value: value,
            }))
        } else {
            self.errors.push(format!(
                "could not parse {} as integer",
                self.cur_token.literal
            ));
            None
        }
    }
```
测试结果失败！
```
thread 'parser::tests::test_integer_literal_expression' panicked at 'program has not enough statements. got=0', src/parser_test.rs:367:13
```

在parse_expression方法中增加对IntegerLiteral的支持：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let left_exp: Option<Expression>;
        match self.cur_token.tk_type {
            TokenType::IDENT => left_exp = self.parse_identifier(),
            TokenType::INT => left_exp = self.parse_integer_literal(),
            _ => return None,
        }
        left_exp
    }
```
测试通过！

## 前缀操作符

测试用例：
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_parsing_prefix_expression() {
    let tests = [("!5;", "!", 5), ("-15;", "-", 15)];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);
        if let Some(Program { statements }) = program {
            assert!(
                statements.len() == 1,
                "program.statements does not contain {} statements. got={}",
                1,
                statements.len()
            );

            if let Statement::ExpressionStatement(ExpressionStatement { token, expression }) =
                &statements[0]
            {
                if let Expression::PrefixExpression(PrefixExpression {
                    token: _,
                    operator,
                    right,
                }) = expression
                {
                    assert!(
                        operator == tt.1,
                        "exp.operator is not '{}'. got={}",
                        tt.1,
                        operator
                    );

                    test_integer_literal(right, tt.2);
                } else {
                    assert!(false, "stmt is not PrefixExpression. got={:?}", expression);
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        } else {
            assert!(false, "parse error");
        }
    }
}
```

```rust,noplaypen
// src/parser_test.rs

fn test_integer_literal(il: &Expression, expected_value: i64) {
    if let Expression::IntegerLiteral(IntegerLiteral { token, value }) = il {
        assert!(
            *value == expected_value,
            "integ.value not {}. got={}",
            expected_value,
            value
        );

        assert!(
            token.literal == format!("{}", expected_value),
            "integ.token_literal not {}. got={}",
            value,
            token.literal
        );
    } else {
        assert!(false, "il not IntegerLiteral. got={:?}", il);
    }
}
```
PrefixExpression的定义如下：
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Box<Expression>,
}
impl NodeTrait for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!("({}{})", self.operator, self.right.string())
    }
}
// [...]
pub enum Expression {
// [...]
    PrefixExpression(PrefixExpression),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::PrefixExpression(prefix_expr) => prefix_expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::PrefixExpression(prefix_expr) => prefix_expr.string(),
        }
    }
}
```
测试错误：
```
thread 'parser::tests::test_parsing_prefix_expression' panicked at 'stmt is not PrefixExpression. got=IntegerLiteral(IntegerLiteral { token: Token { tk_type: INT, literal: "5" }, value: 5 })', src/parser_test.rs:434:25
```
增加如下方法：
```rust,noplaypen
// src/parser.rs

    fn no_prefix_parse_fn_error(&mut self, t: TokenType) {
        self.errors
            .push(format!("no prefix parse function for {:?} found", t));
    }
```
修改parse_expression方法，使用上面的方法：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
            TokenType::IDENT => left_exp = self.parse_identifier(),
            TokenType::INT => left_exp = self.parse_integer_literal(),
            _ => {
                self.no_prefix_parse_fn_error(tk_type);
                return None;
            }
        }
        left_exp
    }
```
再次执行测试，错误信息变成：
```
thread 'parser::tests::test_parsing_prefix_expression' panicked at 'parser has 1 errors
parser error: "no prefix parse function for BANG found"
', src/parser_test.rs:281:9
```
为“！”和“-”增加处理代码：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
// [...]
        match tk_type {
// [...]
            TokenType::BANG | TokenType::MINUS => left_exp = self.parse_prefix_expression(),
            _ => {
// [...]
        }
        left_exp
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX);
        if right.is_none() {
            return None;
        }

        Some(Expression::PrefixExpression(PrefixExpression {
            token: token,
            operator: operator,
            right: Box::new(right.unwrap()),
        }))
    }
```
测试成功！

## 中缀操作符

```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_parsing_infix_expressions() {
    let tests = [
        ("5 + 5;", 5, "+", 5),
        ("5 - 5;", 5, "-", 5),
        ("5 * 5;", 5, "*", 5),
        ("5 / 5;", 5, "/", 5),
        ("5 > 5;", 5, ">", 5),
        ("5 < 5;", 5, "<", 5),
        ("5 == 5;", 5, "==", 5),
        ("5 != 5;", 5, "!=", 5),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Program { statements }) = program {
            assert!(
                statements.len() == 1,
                "program.statements does not contain {} statements. got={}",
                1,
                statements.len()
            );
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::InfixExpression(InfixExpression {
                    token: _,
                    left,
                    operator,
                    right,
                }) = expression
                {
                    test_integer_literal(left, tt.1);

                    assert!(
                        operator == tt.2,
                        "exp.operator is not '{}. got={}",
                        tt.2,
                        operator
                    );

                    test_integer_literal(right, tt.3);
                } else {
                    assert!(false, "exp is not InfixExpression. got={:?}", expression);
                }
            } else {
                assert!(
                    false,
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        } else {
            assert!(false, "parse error");
        }
    }
}
```
定义InfixExpression
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub operator: String,
    pub right: Box<Expression>,
}
impl NodeTrait for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.string(),
            self.operator,
            self.right.string()
        )
    }
}

#[derive(Debug)]
pub enum Expression {
// [...]
    InfixExpression(InfixExpression),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::InfixExpression(infix_expr) => infix_expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::InfixExpression(infix_expr) => infix_expr.string(),
        }
    }
}
```
测试失败信息：
```
thread 'parser::tests::test_parsing_infix_expressions' panicked at 'parser has 1 errors
parser error: "no prefix parse function for PLUS found"
', src/parser_test.rs:298:9
```

这里用到了优先级，需要实现一个Token的优先级查表：
```rust,noplaypen
// src/parser.rs

    fn peek_precedence(&self) -> Precedence {
        get_precedence(&self.peek_token.tk_type)
    }
    fn cur_precedence(&self) -> Precedence {
        get_precedence(&self.cur_token.tk_type)
    }
}

fn get_precedence(t: &TokenType) -> Precedence {
    match t {
        TokenType::EQ | TokenType::NOTEQ => Precedence::EQUALS,
        TokenType::LT | TokenType::GT => Precedence::LESSGREATER,
        TokenType::PLUS | TokenType::MINUS => Precedence::SUM,
        TokenType::SLASH | TokenType::ASTERISK => Precedence::PRODUCT,
        _ => Precedence::LOWEST,
    }
}
```

下面是parse_infix_expression代码
```rust,noplaypen
// src/parser.rs

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence);
        if right.is_none() {
            return None;
        }
        Some(Expression::InfixExpression(InfixExpression {
            token: token,
            left: Box::new(left),
            operator: operator,
            right: Box::new(right.unwrap()),
        }))
    }
```
需要修改parse_expression支持InfixExpression：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
        }
        if left_exp.is_none() {
            return None;
        }

        while !self.peek_token_is(TokenType::SEMICOLON) && precedence < self.peek_precedence() {
            let tk_type = self.peek_token.tk_type.clone();
            match tk_type {
                TokenType::PLUS
                | TokenType::MINUS
                | TokenType::SLASH
                | TokenType::ASTERISK
                | TokenType::EQ
                | TokenType::NOTEQ
                | TokenType::LT
                | TokenType::GT => {
                    self.next_token();
                    left_exp = self.parse_infix_expression(left_exp.unwrap());
                }
                _ => return left_exp,
            }
        }
        left_exp
    }
```
这里需要把left_exp修改成可变的。

另外，由于这里有precedence的比较，需要修改Precedence的定义，加上PartialOrd和PartialEq属性。
```rust,noplaypen
// src/parser.rs

#[derive(PartialOrd, PartialEq)]
pub enum Precedence {
    LOWEST,
    EQUALS,      // ==
    LESSGREATER, // > or <
    SUM,         // +
    PRODUCT,     // *
    PREFIX,      // -x or !x
    CALL,        // myFunction(X)
}
```
测试通过！

写一个测试操作符优先级的用例
```rust,noplaypen
// src/parser_test.rs

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
        ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
        (
            "3 + 4 * 5 == 3 * 1 +  4 * 5",
            "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
        ),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        let actual = program.unwrap().string();
        assert!(actual == tt.1, "expected={:?}, got={:?}", tt.1, actual);
    }
}
```
测试通过！

