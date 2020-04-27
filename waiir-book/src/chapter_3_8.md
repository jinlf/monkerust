# 扩展解析器

类似于test_integer_literal，我们再编写一个辅助测试的函数：
```rust,noplaypen
// src/parser/parser_test.rs

fn test_identifier(exp: &Expression, expected_value: &str) {
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
        panic!("exp not Identifier. got={:?}", exp);
    }
}
```

下面用这个函数来做测试：
```rust,noplaypen
// src/parser/parser_test.rs

fn test_literal_expression(exp: &Expression, expected: &ExpectedType) {
    match expected {
        ExpectedType::Ival(v) => test_integer_literal(exp, *v),
        ExpectedType::Sval(v) => test_identifier(exp, v),
        ExpectedType::Bval(v) => panic!("unsupported"),
    }
}

fn test_infix_expression(
    exp: &Expression,
    expected_left: &ExpectedType,
    expected_operator: &str,
    expected_right: &ExpectedType,
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
        panic!("exp is not InfixExpression. got={:?}", exp);
    }
}
```

## 布尔值字面量

布尔值字面量表示如下：
```js
true;
false;
let foobar = true; 
let barfoo = false;
```
定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct BooleanLiteral {
    pub token: Token,
    pub value: bool,
}
impl NodeTrait for BooleanLiteral {
    fn string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug)]
pub enum Expression {
// [...]
    BooleanLiteral(BooleanLiteral),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
// [...]
            Expression::BooleanLiteral(bo) => bo.string(),
        }
    }
}
```
用Rust的bool类型保存Monkey语言的布尔值。

在parse_expression中增加对布尔值字面量的支持：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::IDENT, |parser| parser.parse_identifier());
        p.register_prefix(TokenType::INT, |parser| parser.parse_integer_literal());
        p.register_prefix(TokenType::BANG, |parser| parser.parse_prefix_expression());
        p.register_prefix(TokenType::MINUS, |parser| parser.parse_prefix_expression());
        p.register_prefix(TokenType::TRUE, |parser| parser.parse_boolean_literal());
        p.register_prefix(TokenType::FALSE, |parser| parser.parse_boolean_literal());
// [...]          
```

```rust,noplaypen
// src/parser/parser.rs

    fn parse_boolean_literal(&self) -> Result<Expression, String> {
        Ok(Expression::BooleanLiteral(BooleanLiteral {
            token: self.cur_token.clone(),
            value: self.cur_token_is(&TokenType::TRUE),
        }))
    }
```
测试通过！


增加一些测试用例测试布尔值字面量：
```rust,noplaypen
// src/parser/parser_test.rs

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

修改test_literal_expression，增加对布尔值字面量的支持：
```rust,noplaypen
// src/parser/parser_test.rs

fn test_literal_expression(exp: &Expression, expected: &ExpectedType) {
// [...]
        ExpectedType::Bval(v) => test_boolean_literal(exp, *v),
// [...]            
}

fn test_boolean_literal(exp: &Expression, expected_value: bool) {
    if let Expression::BooleanLiteral(BooleanLiteral { token, value }) = exp {
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
        assert!(false, "exp not BooleanLiteral. got={:?}", exp);
    }
}
```
为了能够使用test_literal_expression方法，需要将test_parsing_infix_expressions重构：
```rust,noplaypen
// src/parser/parser_test.rs

fn test_parsing_infix_expressions() {
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
                        test_literal_expression(left, &tt.1);

                        assert!(
                            operator == tt.2,
                            "exp.operator is not '{}. got={}",
                            tt.2,
                            operator
                        );

                        test_literal_expression(right, &tt.3);
                    } else {
// [...]
```


同样重构test_parsing_prefix_expression 
```rust,noplaypen
// src/parser/parser_test.rs

fn test_parsing_prefix_expression() {
    let tests = vec![
        ("!5;", "!", ExpectedType::from(5)),
        ("-15;", "-", ExpectedType::from(15)),
        ("!true", "!", ExpectedType::from(true)),
        ("!false", "!", ExpectedType::from(false)),
    ];

    for tt in tests.iter() {
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
                    test_literal_expression(right, &tt.2);
                } else {
// [...]
```
测试通过！

## 分组表达式

Monkey语言中的分组表达式如下：
```js
(5 + 5) * 2;
```

测试用例
```rust,noplaypen
// src/parser/parser_test.rs

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
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::TRUE, |parser| parser.parse_boolean_literal());
        p.register_prefix(TokenType::FALSE, |parser| parser.parse_boolean_literal());
        p.register_prefix(TokenType::LPAREN, |parser| {
            parser.parse_grouped_expression()
        });
```
其中
```rust,noplaypen
// src/parser/parser.rs

    fn parse_grouped_expression(&mut self) -> Result<Expression, String> {
        self.next_token();
        let exp = self.parse_expression(Precedence::LOWEST)?;
        self.expect_peek(&TokenType::RPAREN)?;
        Ok(exp)
    }
```
测试通过！

## if表达式

在Monkey语言中if和else例子如下：
```js
if (x > y) { 
    return x;
} else { 
    return y;
}
```
else是可选的：
```js
if (x > y) { 
    return x;
}
```
Monkey中if是表达式，可以这样用：
```js
let foobar = if (x > y) { x } else { y };
```
结构如下：
```js
if (<condition>)<consequence> else <alternative>
```
其中consequence和alternative都是块语句，即被中括号包围起来的一系列语句。

定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}
impl NodeTrait for IfExpression {
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
    fn string(&self) -> String {
        match self {
// [...]
            Expression::IfExpression(if_expr) => if_expr.string(),
        }
    }
}
```
由于alternative是可选的，这里用Option来包装。

用到的BlockStatement定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}
impl NodeTrait for BlockStatement {
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
// src/parser/parser_test.rs

#[test]
fn test_if_expression() {
    let input = "if (x < y) { x }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                        &ExpectedType::from("x"),
                        "<",
                        &ExpectedType::from("y"),
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
                        test_identifier(expression, "x");

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
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```
将input换成
```rust,noplaypen
if (x < y) { x } else { y }
```
可以编写另一个测试用例test_if_else_expression，与test_if_expression不同的部分如下：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_if_else_expression() {
    let input = "if (x < y) { x } else { y }";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                        &ExpectedType::from("x"),
                        "<",
                        &ExpectedType::from("y"),
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
                        test_identifier(expression, "x");

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
                                test_identifier(expression, "y");
                            } else {
                                assert!(
                                    false,
                                    "statements[0] is not ExpressionStatement. got={:?}",
                                    &a.statements[0]
                                );
                            }
                        } else {
                            panic!("exp alternative.statements was None");
                        }
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
        }
        Err(errors) => panic_with_errors(errors),
    }
}                 
```

测试结果如下：
```
thread 'parser::tests::test_if_expression' panicked at 'parser has 3 errors
parser error: "no prefix parse function for IF found"
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
', src/parser/parser_test.rs:372:9
...
thread 'parser::tests::test_if_else_expression' panicked at 'parser has 6 errors
parser error: "no prefix parse function for IF found"
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
parser error: "no prefix parse function for ELSE found"
parser error: "no prefix parse function for LBRACE found"
parser error: "no prefix parse function for RBRACE found"
', src/parser/parser_test.rs:372:9
```
修改parse_expression如下：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::LPAREN, |parser| {
            parser.parse_grouped_expression()
        });
        p.register_prefix(TokenType::IF, |parser| parser.parse_if_expression());      
// [...]
```
其中：
```rust,noplaypen
// src/parser/parser.rs

    fn parse_if_expression(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        self.expect_peek(&TokenType::LPAREN)?;
        self.next_token();
        let condition = self.parse_expression(Precedence::LOWEST)?;
        self.expect_peek(&TokenType::RPAREN)?;
        self.expect_peek(&TokenType::LBRACE)?;

        let consequence = self.parse_block_statement()?;

        let mut alternative: Option<BlockStatement> = None;

        if self.peek_token_is(&TokenType::ELSE) {
            self.next_token();

            self.expect_peek(&TokenType::LBRACE)?;
            alternative = Some(self.parse_block_statement()?);
        }

        Ok(Expression::IfExpression(IfExpression {
            token: token,
            condition: Box::new(condition),
            consequence: consequence,
            alternative: alternative,
        }))
    }

    fn parse_block_statement(&mut self) -> Result<BlockStatement, String> {
        let token = self.cur_token.clone();
        let mut statements: Vec<Statement> = Vec::new();
        self.next_token();

        while !self.cur_token_is(&TokenType::RBRACE) {
            if self.cur_token_is(&TokenType::EOF) {
                return Err(String::from("EOF"));
            }
            let stmt = self.parse_statement()?;
            statements.push(stmt);
            self.next_token();
        }
        Ok(BlockStatement {
            token: token,
            statements: statements,
        })
    }
```
测试通过！

## 函数字面量

Monkey语言中函数字面量如下：
```js
fn(x, y) { 
    return x + y;
}
```
其结构如下：
```js
fn <parameters><block statement>
```
其中parameters的结构是：
```js
(<parameter 1>, <parameter 2>, <parameter 3>, ...)
```
参数可以为空：
```js
fn() {
    return foobar + barfoo;
}
```
Monkey中的函数字面量也是表达式：
```js
fn() {
    return fn(x, y) { return x > y; };
}
```
这里一个函数是另一个函数返回语句中的表达式。

还可以用函数做实参：
```js
myFunc(x, y, fn(x, y) { return x > y; });
```

定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}
impl NodeTrait for FunctionLiteral {
    fn string(&self) -> String {
        format!(
            "{} ({}) {}",
            self.token.literal,
            self.parameters
                .iter()
                .map(|x| x.string())
                .collect::<Vec<String>>()
                .join(", "),
            self.body.string()
        )
    }
}

pub enum Expression {
// [...]
    FunctionLiteral(FunctionLiteral),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
// [...]
            Expression::FunctionLiteral(function_literal) => function_literal.string(),
        }
    }
}
```
上述定义中参数是标识符列表，函数体是一个块语句。

测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_function_literal_parsing() {
    let input = "fn(x, y) { x + y; }";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                        &ExpectedType::from("x"),
                    );
                    test_literal_expression(
                        &Expression::Identifier(parameters[1].clone()),
                        &ExpectedType::from("y"),
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
                            &ExpectedType::from("x"),
                            "+",
                            &ExpectedType::from("y"),
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
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```
由于这里需要对Identifier类型的parameters项进行clone，下面就增加Clone属性。
```rust,noplaypen
// src/ast/ast.rs

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
', src/parser/parser_test.rs:439:9
```

需要修改parse_expression方法支持函数字面量
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::IF, |parser| parser.parse_if_expression());
        p.register_prefix(TokenType::FUNCTION, |parser| {
            parser.parse_function_literal()
        });
// [...]

    fn parse_function_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        self.expect_peek(&TokenType::LPAREN)?;

        let parameters = self.parse_function_parameters()?;

        self.expect_peek(&TokenType::LBRACE)?;

        let body = self.parse_block_statement()?;

        Ok(Expression::FunctionLiteral(FunctionLiteral {
            token: token,
            parameters: parameters,
            body: body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<Identifier>, String> {
        if self.peek_token_is(&TokenType::RPAREN) {
            self.next_token();
            return Ok(Vec::new());
        }

        self.next_token();

        let mut identfiers: Vec<Identifier> = Vec::new();
        identfiers.push(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        });

        while self.peek_token_is(&TokenType::COMMA) {
            self.next_token();
            self.next_token();

            identfiers.push(Identifier {
                token: self.cur_token.clone(),
                value: self.cur_token.literal.clone(),
            })
        }

        self.expect_peek(&TokenType::RPAREN)?;
        Ok(identfiers)
    }
```
增加测试用例
```rust,noplaypen
// src/parser/parser_test.rs

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
        match p.parse_program() {
            Ok(Program { statements }) => {
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
                                &ExpectedType::from(*ident),
                            );
                        }
                    } else {
                        panic!("parse error");
                    }
                } else {
                    panic!("parse error");
                }
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}
```
测试通过！

## 调用表达式

调用表达式的结构如下：
```js
<expression>(<expression 1>, <expression 2>, ...)
```
其中实参，即括号内的表达式可以为空。

调用表达式示例如下：
```js
add(2, 3)
```
或
```js
add(2 + 2, 3 * 3 * 3)
```
函数名称也可以换成函数字面量：
```js
fn(x, y) { x + y; }(2, 3)
```
调用表达式的实参表达式也可以是函数字面量：
```js
callsFunction(2, 3, fn(x, y) { x + y; });
```

定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}
impl NodeTrait for CallExpression {
    fn string(&self) -> String {
        format!(
            "{}({})",
            self.function.string(),
            self.arguments
                .iter()
                .map(|x| x.string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub enum Expression {
// [...]
    CallExpression(CallExpression),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
// [...]
            Expression::CallExpression(call_expr) => call_expr.string(),
        }
    }
}
```
测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_call_expression_parsing() {
    let input = "add(1, 2 * 3, 4 + 5);";

    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                    test_identifier(function, "add");

                    assert!(
                        arguments.len() == 3,
                        "wrong length of arguments. got={}",
                        arguments.len()
                    );

                    test_literal_expression(&arguments[0], &ExpectedType::from(1));
                    test_infix_expression(
                        &arguments[1],
                        &ExpectedType::from(2),
                        "*",
                        &ExpectedType::from(3),
                    );
                    test_infix_expression(
                        &arguments[2],
                        &ExpectedType::from(4),
                        "+",
                        &ExpectedType::from(5),
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
        }
        Err(errors) => panic_with_errors(errors),
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
', src/parser/parser_test.rs:497:9
```
在中缀操作符解析时加上左括号的处理：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_infix(TokenType::GT, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::LPAREN, |parser, exp| {
            parser.parse_call_expression(exp)
        });
// [...]

    fn parse_call_expression(&mut self, function: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let arguements = self.parse_expression_list(TokenType::RPAREN)?;

        Ok(Expression::CallExpression(CallExpression {
            token: token,
            function: Box::new(function),
            arguments: arguements,
        }))
    }

    fn parse_call_arguments(&mut self) -> Result<Vec<Expression>, String> {
        let mut args: Vec<Expression> = Vec::new();
        if self.peek_token_is(&TokenType::RPAREN) {
            self.next_token();
            return Ok(args);
        }

        self.next_token();
        let arg = self.parse_expression(Precedence::LOWEST)?;
        args.push(arg);

        while self.peek_token_is(&TokenType::COMMA) {
            self.next_token();
            self.next_token();
            let arg = self.parse_expression(Precedence::LOWEST)?;
            args.push(arg);
        }

        self.expect_peek(&TokenType::RPAREN)?;

        Ok(args)
    }

    fn parse_expression_list(&mut self, end: TokenType) -> Result<Vec<Expression>, String> {
        let mut list: Vec<Expression> = Vec::new();

        if self.peek_token_is(&end) {
            self.next_token();
            return Ok(list);
        }

        self.next_token();
        let mut expr = self.parse_expression(Precedence::LOWEST)?;
        list.push(expr);

        while self.peek_token_is(TokenType::COMMA) {
            self.next_token();
            self.next_token();
            expr = self.parse_expression(Precedence::LOWEST)?;
            list.push(expr);
        }

        self.expect_peek(&end)?;
        Ok(list)
    }
```
测试结果仍然出错：
```
thread 'parser::tests::test_call_expression_parsing' panicked at 'parser has 5 errors
parser error: "expected next token to be RPAREN, got COMMA instead"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for COMMA found"
parser error: "no prefix parse function for RPAREN found"
parser error: "no prefix parse function for SEMICOLON found"
', src/parser/parser_test.rs:546:9
```
出现此错误的原因是中缀处理时没查到对应的优先级，修改get_precedence函数：
```rust,noplaypen
// src/parser/parser.rs

fn get_precedence(t: &TokenType) -> Precedence {
    match t {
// [...]
        TokenType::LPAREN => Precedence::CALL,
        _ => Precedence::LOWEST,
    }
}
```
为了验证CALL优先级是最高优先级，扩展测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

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

我们开始编写解析器的时候，由于没有表达式解析能力，在处理let和return语句时加入了MockExpression，以及跳过表达式解析的代码，现在是时候解决这些问题了。

删除ast.rs中的TODO
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    PrefixExpression(PrefixExpression),
    InfixExpression(InfixExpression),
    BooleanLiteral(BooleanLiteral),
    IfExpression(IfExpression),
    FunctionLiteral(FunctionLiteral),
    CallExpression(CallExpression),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.string(),
            Expression::IntegerLiteral(integer_literal) => integer_literal.string(),
            Expression::PrefixExpression(prefix_expr) => prefix_expr.string(),
            Expression::InfixExpression(infix_expr) => infix_expr.string(),
            Expression::BooleanLiteral(bo) => bo.string(),
            Expression::IfExpression(if_expr) => if_expr.string(),
            Expression::FunctionLiteral(function_literal) => function_literal.string(),
            Expression::CallExpression(call_expr) => call_expr.string(),
        }
    }
}
```
修改parse_let_statement和parse_return_statement。
```rust,noplaypen
// src/parser/parser.rs

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();

        self.expect_peek(&TokenType::IDENT)?;

        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        self.expect_peek(&TokenType::ASSIGN)?;

        self.next_token();

        let value = self.parse_expression(Precedence::LOWEST)?;

        if self.peek_token_is(&TokenType::SEMICOLON) {
            self.next_token();
        }

        Ok(Statement::LetStatement(LetStatement {
            token: token,
            name: name,
            value: value,
        }))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();
        self.next_token();

        let return_value = self.parse_expression(Precedence::LOWEST)?;

        if self.peek_token_is(&TokenType::SEMICOLON) {
            self.next_token();
        }

        Ok(Statement::ReturnStatement(ReturnStatement {
            token: token,
            return_value: return_value,
        }))
    }
```
测试通过！

TODO都删除了，下面试一试测试驱动的解析。