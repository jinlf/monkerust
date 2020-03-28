# 扩展解析器

```rust,noplaypen
// src/parser_test.rs

fn test_identifier(exp: &Expression, expected_value: String) {
    if let Expression::Identifier(Identifier { token, value }) = exp {
        assert!(
            *value == expected_value,
            "ident.value not {}. got={}",
            expected_value,
            value
        );

        assert!(
            token.literal == expected_value,
            "ident.token_literal not {}. got={}",
            expected_value,
            token.literal
        );
    } else {
        assert!(false, "exp not Identifier. got={:?}", exp);
    }
}
```

```rust,noplaypen
// src/parser_test.rs

fn test_literal_expression(exp: &Expression, expected: &dyn std::any::Any) {
    if let Some(v) = expected.downcast_ref::<i64>() {
        test_integer_literal(exp, *v);
    } else if let Some(v) = expected.downcast_ref::<&str>() {
        test_identifier(exp, String::from(*v));
    } else {
        assert!(false, "type of exp not handled. got={:?}", exp);
    }
}

fn test_infix_expression(
    exp: &Expression,
    expected_left: &dyn std::any::Any,
    expected_operator: String,
    expected_right: &dyn std::any::Any,
) {
    if let Expression::InfixExpression(InfixExpression {
        token: _,
        left,
        operator,
        right,
    }) = exp
    {
        test_literal_expression(left, expected_left);
        assert!(
            *operator == expected_operator,
            "exp.operator is not '{}'. got={:?}",
            expected_operator,
            operator
        );
        test_literal_expression(right, expected_right);
    } else {
        assert!(false, "exp is not InfixExpression. got={:?}", exp);
    }
}
```

## Boolean字面量

```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
}
impl NodeTrait for Boolean {
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
    Boolean(Boolean),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::Boolean(bo) => bo.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::Boolean(bo) => bo.string(),
        }
    }
}
```

在parse_expression中增加
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
            TokenType::TRUE | TokenType::FALSE => left_exp = self.parse_boolean(),
// [...]
        }        
```

```rust,noplaypen
// src/parser.rs

    fn parse_boolean(&self) -> Option<Expression> {
        Some(Expression::Boolean(Boolean {
            token: self.cur_token.clone(),
            value: self.cur_token_is(TokenType::TRUE),
        }))
    }
```
测试通过！

在test_operator_precedence_parsing中增加
```rust,noplaypen
// src/parser_test.rs

fn test_operator_precedence_parsing() {
    let tests = [
// [...]
        ("true", "true"),
        ("false", "false"),
        ("3 > 5 == false", "((3 > 5) == false)"),
        ("3 < 5 == true", "((3 < 5) == true)"),
    ];
// [...]
```

修改test_literal_expression
```rust,noplaypen
// src/parser_test.rs

fn test_literal_expression(exp: &Expression, expected: &dyn std::any::Any) {
// [...]
    } else if let Some(v) = expected.downcast_ref::<bool>() {
        test_boolean_literal(exp, *v);
    } else {
// [...]            
}

fn test_boolean_literal(exp: &Expression, expected_value: bool) {
    if let Expression::Boolean(Boolean { token, value }) = exp {
        assert!(
            *value == expected_value,
            "bo.value not {}. got={}",
            expected_value,
            value
        );
        assert!(
            token.literal == format!("{}", expected_value),
            "bo.token_literal not {}. got={}",
            expected_value,
            token.literal
        );
    } else {
        assert!(false, "exp not Boolean. got={:?}", exp);
    }
}
```
修改test_parsing_infix_expressions为
```rust,noplaypen
// src/parser_test.rs

fn test_parsing_infix_expressions() {
    let tests: [(&str, Box<dyn std::any::Any>, &str, Box<dyn std::any::Any>); 11] = [
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
// [...]
        if let Some(Program { statements }) = program {
// [...]
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
                    test_literal_expression(left, &*tt.1);

                    assert!(
                        operator == tt.2,
                        "exp.operator is not '{}. got={}",
                        tt.2,
                        operator
                    );

                    test_literal_expression(right, &*tt.3);
                } else {
// [...]
```
修改test_parsing_infix_expression 
```rust,noplaypen
// src/parser_test.rs

fn test_parsing_prefix_expression() {
    let tests: [(&str, &str, Box<dyn std::any::Any>); 4] = [
        ("!5;", "!", Box::new(5 as i64)),
        ("-15;", "-", Box::new(15 as i64)),
        ("!true", "!", Box::new(true)),
        ("!false", "!", Box::new(false)),
    ];

    for tt in tests.iter() {
// [...]
        if let Some(Program { statements }) = program {
// [...]
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::PrefixExpression(PrefixExpression {
                    token: _,
                    operator,
                    right,
                }) = expression
                {
// [...]
                    test_literal_expression(right, &*tt.2);
                } else {
// [...]
```
测试通过！

## 组合表达式

测试用例
```rust,noplaypen
// src/parser_test.rs

fn test_operator_precedence_parsing() {
    let tests = [
// [...]
        ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
        ("(5 + 5) * 2", "((5 + 5) * 2)"),
        ("2 / (5 + 5)", "(2 / (5 + 5))"),
        ("-(5 + 5)", "(-(5 + 5))"),
        ("!(true == true)", "(!(true == true))"),
    ];
// [...]
```
测试失败结果：
```
thread 'parser::tests::test_operator_precedence_parsing' panicked at 'parser has 3 errors
parser error: "no prefix parse function for LPAREN found"
parser error: "no prefix parse function for RPAREN found"
parser error: "no prefix parse function for PLUS found"
```
修改parse_expression如下：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
            TokenType::LPAREN => left_exp = self.parse_grouped_expression(),
            _ => {
// [...]
```
其中
```rust,noplaypen
// src/parser.rs

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let exp = self.parse_expression(Precedence::LOWEST);
        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        exp
    }
```
测试通过！

## if表达式

定义：
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}
impl NodeTrait for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "if{} {}",
            self.condition.string(),
            self.consequence.string()
        ));
        if let Some(a) = &self.alternative {
            out.push_str(&format!("else {}", a.string()));
        }
        out
    }
}

pub enum Expression {
// [...]
    IfExpression(IfExpression),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::IfExpression(if_expr) => if_expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::IfExpression(if_expr) => if_expr.string(),
        }
    }
}
```
而BlockStatement的定义如下：
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}
impl NodeTrait for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut out = String::new();

        for s in self.statements.iter() {
            out.push_str(&s.string());
        }
        out
    }
}

pub enum Statement {
// [...]
    BlockStatement(BlockStatement),
}
impl NodeTrait for Statement {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Statement::BlockStatement(block_stmt) => block_stmt.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Statement::BlockStatement(block_stmt) => block_stmt.string(),
        }
    }
}
```
下面写测试用例：
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_if_expression() {
    let input = "if (x < y) { x }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 1,
            "program.body does not contain {} statements. got={}",
            1,
            statements.len()
        );

        if let Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        }) = &statements[0]
        {
            if let Expression::IfExpression(IfExpression {
                token: _,
                condition,
                consequence,
                alternative,
            }) = expression
            {
                test_infix_expression(
                    condition,
                    &*Box::new("x"),
                    String::from("<"),
                    &*Box::new("y"),
                );

                assert!(
                    consequence.statements.len() == 1,
                    "consequence is not 1 statements. got={}",
                    consequence.statements.len()
                );

                if let Statement::ExpressionStatement(ExpressionStatement {
                    token: _,
                    expression,
                }) = &consequence.statements[0]
                {
                    test_identifier(expression, String::from("x"));

                    assert!(
                        alternative.is_none(),
                        "exp alternative.statements was not None. got={:?}",
                        alternative,
                    );
                } else {
                    assert!(
                        false,
                        "statements[0] is not ExpressionStatement. got={:?}",
                        &consequence.statements[0]
                    );
                }
            } else {
                assert!(
                    false,
                    "stmt.expression is not IfExpression. got={:?}",
                    expression
                );
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
将input换成
```rust,noplaypen
if (x < y) { x } else { y }
```
可以做另一个测试用例test_if_else_expression。
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_if_else_expression() {
    let input = "if (x < y) { x } else { y }";
// [...]
                    test_identifier(expression, String::from("x"));

                    if let Some(a) = alternative {
                        assert!(
                            a.statements.len() == 1,
                            "alternative is not 1 statements. got={}",
                            a.statements.len()
                        );
                        if let Statement::ExpressionStatement(ExpressionStatement {
                            token: _,
                            expression,
                        }) = &a.statements[0]
                        {
                            test_identifier(expression, String::from("y"));
                        } else {
                            assert!(
                                false,
                                "statements[0] is not ExpressionStatement. got={:?}",
                                &a.statements[0]
                            );
                        }
                    } else {
                        assert!(false, "exp alternative.statements was None");
                    }
                } else {
// [...]                    

```

测试结果如下：
```
thread 'parser::tests::test_if_expression' panicked at 'parser has 3 errors
parser error: "no prefix parse function for IF found"
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
', src/parser_test.rs:372:9
...
thread 'parser::tests::test_if_else_expression' panicked at 'parser has 6 errors
parser error: "no prefix parse function for IF found"
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
parser error: "no prefix parse function for ELSE found"
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
', src/parser_test.rs:372:9
```
修改parse_expression如下：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
            TokenType::IF => left_exp = self.parse_if_expression(),
            _ => {
// [...]                
```
其中：
```rust,noplaypen
// src/parser.rs

    fn parse_if_expression(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }
        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST);
        if condition.is_none() {
            return None;
        }
        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let consequence = self.parse_block_statement();
        if consequence.is_none() {
            return None;
        }

        let mut alternative: Option<BlockStatement> = None;

        if self.peek_token_is(TokenType::ELSE) {
            self.next_token();

            if !self.expect_peek(TokenType::LBRACE) {
                return None;
            }
            alternative = self.parse_block_statement();
            if alternative.is_none() {
                return None;
            }
        }

        Some(Expression::IfExpression(IfExpression {
            token: token,
            condition: Box::new(condition.unwrap()),
            consequence: consequence.unwrap(),
            alternative: alternative,
        }))
    }

    fn parse_block_statement(&mut self) -> Option<BlockStatement> {
        let token = self.cur_token.clone();
        let mut statements: Vec<Statement> = Vec::new();
        self.next_token();

        while !self.cur_token_is(TokenType::RBRACE) {
            if self.cur_token_is(TokenType::EOF) {
                return None;
            }
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                return None;
            }
            self.next_token();
        }
        Some(BlockStatement {
            token: token,
            statements: statements,
        })
    }
```
测试通过！

## 函数字面量

```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}
impl NodeTrait for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut params: Vec<String> = Vec::new();
        for p in self.parameters.iter() {
            params.push(p.string());
        }
        format!(
            "{} ({}) {}",
            self.token_literal(),
            params.join(", "),
            self.body.string()
        )
    }
}

pub enum Expression {
// [...]
    FunctionLiteral(FunctionLiteral),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::FunctionLiteral(function_literal) => function_literal.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::FunctionLiteral(function_literal) => function_literal.string(),
        }
    }
}
```
测试用例
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_function_literal_parsing() {
    let input = "fn(x, y) { x + y; }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 1,
            "program.body does not contain {} statements. got={}",
            1,
            statements.len()
        );

        if let Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        }) = &statements[0]
        {
            if let Expression::FunctionLiteral(FunctionLiteral {
                token: _,
                parameters,
                body,
            }) = expression
            {
                assert!(
                    parameters.len() == 2,
                    "function literal parameters wrong. want 2, got={}",
                    parameters.len()
                );

                test_literal_expression(
                    &Expression::Identifier(parameters[0].clone()),
                    &*Box::new("x"),
                );
                test_literal_expression(
                    &Expression::Identifier(parameters[1].clone()),
                    &*Box::new("y"),
                );

                assert!(
                    body.statements.len() == 1,
                    "function.body.statements has not 1 statements. got={}",
                    body.statements.len()
                );

                if let Statement::ExpressionStatement(ExpressionStatement {
                    token: _,
                    expression,
                }) = &body.statements[0]
                {
                    test_infix_expression(
                        expression,
                        &*Box::new("x"),
                        String::from("+"),
                        &*Box::new("y"),
                    );
                } else {
                    assert!(
                        false,
                        "function body stmt is not ExpressionStatement. got={:?}",
                        &body.statements[0]
                    );
                }
            } else {
                assert!(
                    false,
                    "stmt.expression is not FunctionLiteral. got={:?}",
                    expression
                );
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
由于这里需要对Identifier类型的parameters项进行clone，下面就增加Clone属性。
```rust,noplaypen
// src/ast.rs

#[derive(Debug, Clone)]
pub struct Identifier {
// [...]
```

测试错误信息如下：
```
thread 'parser::tests::test_function_literal_parsing' panicked at 'parser has 6 errors
parser error: "no prefix parse function for FUNCTION found"
parser error: "expected next token to be RPAREN, got COMMA instead"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for RPAREN found"
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
', src/parser_test.rs:439:9
```

需要修改parse_expression方法支持函数字面量
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left_exp: Option<Expression>;
        let tk_type = self.cur_token.tk_type.clone();
        match tk_type {
// [...]
            TokenType::FUNCTION => left_exp = self.parse_function_literal(),
            _ => {
// [...]

    fn parse_function_literal(&mut self) -> Option<Expression> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::LPAREN) {
            return None;
        }

        let parameters = self.parse_function_parameters();
        if parameters.is_none() {
            return None;
        }

        if !self.expect_peek(TokenType::LBRACE) {
            return None;
        }

        let body = self.parse_block_statement();
        if body.is_none() {
            return None;
        }

        Some(Expression::FunctionLiteral(FunctionLiteral {
            token: token,
            parameters: parameters.unwrap(),
            body: body.unwrap(),
        }))
    }

    fn parse_function_parameters(&mut self) -> Option<Vec<Identifier>> {
        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return Some(Vec::new());
        }

        self.next_token();

        let mut identfiers: Vec<Identifier> = Vec::new();
        identfiers.push(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        });

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();

            identfiers.push(Identifier {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            })
        }

        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }
        Some(identfiers)
    }
```
增加测试用例
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_function_parameter_parsing() {
    let tests = [
        ("fn() {};", Vec::new()),
        ("fn(x) {};", vec!["x"]),
        ("fn(x, y, z) {};", vec!["x", "y", "z"]),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(tt.0);
        let mut p = Parser::new(l);
        let program = p.parse_program();
        check_parser_errors(&mut p);

        if let Some(Program { statements }) = program {
            if let Statement::ExpressionStatement(ExpressionStatement {
                token: _,
                expression,
            }) = &statements[0]
            {
                if let Expression::FunctionLiteral(FunctionLiteral {
                    token: _,
                    parameters,
                    body: _,
                }) = expression
                {
                    assert!(
                        parameters.len() == tt.1.len(),
                        "length parameters wrong. want {}, got={}",
                        tt.1.len(),
                        parameters.len()
                    );
                    for (i, ident) in tt.1.iter().enumerate() {
                        test_literal_expression(
                            &Expression::Identifier(parameters[i].clone()),
                            ident,
                        );
                    }
                } else {
                    assert!(false, "parse error");
                }
            } else {
                assert!(false, "parse error");
            }
        } else {
            assert!(false, "parse error");
        }
    }
}
```
测试通过！

## 调用表达式

定义
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}
impl NodeTrait for CallExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
    fn string(&self) -> String {
        let mut args: Vec<String> = Vec::new();
        for a in self.arguments.iter() {
            args.push(a.string());
        }

        format!("{}({})", self.function.string(), args.join(", "))
    }
}

pub enum Expression {
// [...]
    CallExpression(CallExpression),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
// [...]
            Expression::CallExpression(call_expr) => call_expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
// [...]
            Expression::CallExpression(call_expr) => call_expr.string(),
        }
    }
}
```
测试用例
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_call_expression_parsing() {
    let input = "add(1, 2 * 3, 4 + 5);";

    let l = Lexer::new(input);
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
            if let Expression::CallExpression(CallExpression {
                token: _,
                function,
                arguments,
            }) = expression
            {
                test_identifier(function, String::from("add"));

                assert!(
                    arguments.len() == 3,
                    "wrong length of arguments. got={}",
                    arguments.len()
                );

                test_literal_expression(&arguments[0], &*Box::new(1 as i64));
                test_infix_expression(
                    &arguments[1],
                    &*Box::new(2 as i64),
                    String::from("*"),
                    &*Box::new(3 as i64),
                );
                test_infix_expression(
                    &arguments[2],
                    &*Box::new(4 as i64),
                    String::from("+"),
                    &*Box::new(5 as i64),
                );
            } else {
                assert!(
                    false,
                    "stmt.expression is not CallExpression. got={:?}",
                    expression
                );
            }
        } else {
            assert!(
                false,
                "stmt is not ExpressionStatement. got={:?}",
                &statements[0]
            );
        }
    } else {
        assert!(false, "parse error");
    }
}
```
测试结果如下：
```
thread 'parser::tests::test_call_expression_parsing' panicked at 'parser has 5 errors
parser error: "expected next token to be RPAREN, got COMMA instead"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for RPAREN found"
parser error: "no prefix parse function for SEMICOLON found"
', src/parser_test.rs:497:9
```
修改：
```rust,noplaypen
// src/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
// [...]
        while !self.peek_token_is(TokenType::SEMICOLON) && precedence < self.peek_precedence() {
            let tk_type = self.peek_token.tk_type.clone();
            match tk_type {
// [...]
                TokenType::LPAREN => {
                    self.next_token();
                    left_exp = self.parse_call_expression(left_exp.unwrap())
                }
                _ => return left_exp,
            }
        }
        left_exp
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let token = self.cur_token.clone();
        let arguements = self.parse_call_arguments();
        if arguements.is_none() {
            return None;
        }

        Some(Expression::CallExpression(CallExpression {
            token: token,
            function: Box::new(function),
            arguments: arguements.unwrap(),
        }))
    }

    fn parse_call_arguments(&mut self) -> Option<Vec<Expression>> {
        let mut args: Vec<Expression> = Vec::new();
        if self.peek_token_is(TokenType::RPAREN) {
            self.next_token();
            return Some(args);
        }

        self.next_token();
        let arg = self.parse_expression(Precedence::LOWEST);
        if arg.is_none() {
            return None;
        }
        args.push(arg.unwrap());

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            let arg = self.parse_expression(Precedence::LOWEST);
            if arg.is_none() {
                return None;
            }
            args.push(arg.unwrap());
        }

        if !self.expect_peek(TokenType::RPAREN) {
            return None;
        }

        Some(args)
    }
```
测试结果
```
thread 'parser::tests::test_call_expression_parsing' panicked at 'parser has 5 errors
parser error: "expected next token to be RPAREN, got COMMA instead"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for RPAREN found"
parser error: "no prefix parse function for SEMICOLON found"
', src/parser_test.rs:546:9
```
修改get_precedence
```rust,noplaypen
// src/parser.rs

fn get_precedence(t: &TokenType) -> Precedence {
    match t {
// [...]
        TokenType::LPAREN => Precedence::CALL,
        _ => Precedence::LOWEST,
    }
}
```
扩展测试用例
```rust,noplaypen
// src/parser_test.rs

fn test_operator_precedence_parsing() {
    let tests = [
// [...]
        ("a + add(b * c) + d", "((a + add((b * c))) + d)"),
        (
            "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
            "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
        ),
        (
            "add(a + b + c * d / f + g)",
            "add((((a + b) + ((c * d) / f)) + g))",
        ),
    ];
```
测试通过！

## 删除TODO

删除ast.rs中的TODO
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    PrefixExpression(PrefixExpression),
    InfixExpression(InfixExpression),
    Boolean(Boolean),
    IfExpression(IfExpression),
    FunctionLiteral(FunctionLiteral),
    CallExpression(CallExpression),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.token_literal(),
            Expression::IntegerLiteral(integer_literal) => integer_literal.token_literal(),
            Expression::PrefixExpression(prefix_expr) => prefix_expr.token_literal(),
            Expression::InfixExpression(infix_expr) => infix_expr.token_literal(),
            Expression::Boolean(bo) => bo.token_literal(),
            Expression::IfExpression(if_expr) => if_expr.token_literal(),
            Expression::FunctionLiteral(function_literal) => function_literal.token_literal(),
            Expression::CallExpression(call_expr) => call_expr.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.string(),
            Expression::IntegerLiteral(integer_literal) => integer_literal.string(),
            Expression::PrefixExpression(prefix_expr) => prefix_expr.string(),
            Expression::InfixExpression(infix_expr) => infix_expr.string(),
            Expression::Boolean(bo) => bo.string(),
            Expression::IfExpression(if_expr) => if_expr.string(),
            Expression::FunctionLiteral(function_literal) => function_literal.string(),
            Expression::CallExpression(call_expr) => call_expr.string(),
        }
    }
}
```
修改parse_let_statement和parse_return_statement。
```rust,noplaypen
// src/parser.rs

    fn parse_let_statement(&mut self) -> Option<LetStatement> {
        let token = self.cur_token.clone();
        if !self.expect_peek(TokenType::IDENT) {
            return None;
        }

        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        if !self.expect_peek(TokenType::ASSIGN) {
            return None;
        }

        self.next_token();
        let value = self.parse_expression(Precedence::LOWEST);
        if value.is_none() {
            return None;
        }

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(LetStatement {
            token: token,
            name: name,
            value: value.unwrap(),
        })
    }

        fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        let token = self.cur_token.clone();
        self.next_token();

        let return_value = self.parse_expression(Precedence::LOWEST);
        if return_value.is_none() {
            return None;
        }

        if self.peek_token_is(TokenType::SEMICOLON) {
            self.next_token();
        }

        Some(ReturnStatement {
            token: token,
            return_value: return_value.unwrap(),
        })
    }
```
测试通过！
