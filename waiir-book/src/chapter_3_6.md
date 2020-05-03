# 解析表达式

解析表达式，需要考虑操作符优先级。例如：
```js
5 * 5 + 10
```
对应的AST表示的是：
```js
((5 * 5) + 10)
```
就是说5 * 5在AST中的深度更深，比加法求值更早。解析器需要区分“*”和“+”的优先级。
但是：
```js
5 * (5 + 10)
```
这种带括号的分组表达式的求值顺序却不同，因为括号比“*”的优先级更高。

另一个问题是同一个操作符可以出现在表达式的不同位置，如：
```js
-5 - 10
```
第一个“-”号是前缀操作符，第二个“-”号是中缀操作符。类似的情况还有：
```js
5 * (add(2, 3) + 10)
```
外面的括号是分组操作符，里面的括号是调用表达式的一部分。

## Monkey语言表达式

Monkey编程语言中let和return后面的都是表达式，还有仅包含表达式的表达式语句。

Monkey语言表达式有以下几种：

- 前缀操作符表达式：
```js
-5
!true
!false
```

- 中缀操作符（数值操作符）表达式：
```js
5 + 5
5 - 5
5 / 5
5 * 5
```

- 中缀操作符（比较操作符）表达式：
```js
foo == bar
foo != bar
foo < bar
foo > bar
```

- 分组表达式：
```js
5 * (5 + 5)
((5 + 5) * 5) + 5
```

- 调用表达式：
```js
add(2, 3)
add(add(2, 3), add(5, 10))
max(5, add(5, (5 * 5)))
```

- 标识符表达式：
```js
foo * bar / foobar
add(foo, bar)
```
- 函数字面量

前面提到过函数是一等公民，函数字面量也是表达式：
```js
let add = fn(x, y) { return x + y };
```
可以用函数字面量来替换标识符：
```js
fn(x, y) { return x + y}(5, 5)
(fn(x){ return x}(5) + 10) * 10
```

- if表达式

跟很多语言不同，if也是表达式：
```js
let result = if (10 > 5) { true } else { flase };
result // => true
```

## 自顶向下操作符优先级（普拉特解析）

1973年发表的论文，最近才被广泛使用。

与基于CFG解析不同的是，普拉特解析不把函数关联到语法规则，而是关联到Token。根据前缀和中缀位置不同，每种Token最多关联两个解析函数。

## 术语

**前缀操作符**：操作符在操作数之前，例如：
```js
--5
```
**后缀操作符**：操作符在操作数之后，例如：
```js
foobar++
```
**中缀操作符**：操作符在两个操作数之间，例如：
```js
5 * 8
```
**操作符优先级**：例如
```js
5 + 5 * 10
```
“*”比“+”优先级要高。

## 准备AST

首先考虑表达式语句，例如：
```js
let x = 5;
x + 10;
```
第一行是let语句，第二行就是表达式语句，是语句和表达式之间的桥梁。

定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Expression,
}
impl NodeTrait for ExpressionStatement {
    fn string(&self) -> String {
        self.expression.string()
    }
}
```
加入Statement：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
    ExpressionStatement(ExpressionStatement),
}
impl NodeTrait for Statement {
    fn string(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.string(),
            Statement::ReturnStatement(return_stmt) => return_stmt.string(),
            Statement::ExpressionStatement(expr_stmt) => expr_stmt.string(),
        }
    }
}
```

下面写AST的一个测试用例：
```rust,noplaypen
// src/ast/ast_test.rs

use crate::ast::*;
use crate::token::*;

#[test]
fn test_string() {
    let program = Program {
        statements: vec![Statement::LetStatement(LetStatement {
            token: Token {
                r#type: TokenType::LET,
                literal: String::from("let"),
            },
            name: Identifier {
                token: Token {
                    r#type: TokenType::IDENT,
                    literal: String::from("myVar"),
                },
                value: String::from("myVar"),
            },
            value: Expression::Identifier(Identifier {
                token: Token {
                    r#type: TokenType::IDENT,
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
在src/ast/mod.rs中加入
```rust,noplaypen
// src/ast/mod.rs

#[cfg(test)]
mod ast_test;
```
测试通过！

## 实现普拉特解析器

实现函数指针：
```rust,noplaypen
// src/parser/parser.rs

use std::collections::HashMap;

type PrefixParseFn = fn(&mut Parser) -> Result<Expression, String>;
type InfixParseFn = fn(&mut Parser, Expression) -> Result<Expression, String>;

// [...]
pub struct Parser {
    pub l: Lexer,
    pub cur_token: Token,
    pub peek_token: Token,
    pub prefix_parse_fns: HashMap<TokenType, PrefixParseFn>,
    pub infix_parse_fns: HashMap<TokenType, InfixParseFn>,
}

impl Parser {
    pub fn new(l: Lexer) -> Parser {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::EOF, 0),
            peek_token: new_token(TokenType::EOF, 0),
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };
// [...]        
    }
// [...]   
    fn register_prefix(&mut self, token_type: TokenType, func: PrefixParseFn) {
        self.prefix_parse_fns.insert(token_type, func);
    }
    fn register_infix(&mut self, token_type: TokenType, func: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, func);
    } 
}
```

为了支持HashMap，需要为TokenType加上Hash和Eq trait：
```rust,noplaypen
// src/token/token.rs

#[derive(PartialEq, Debug, Clone, Hash, Eq)]
pub enum TokenType {
// [...]    
```

先考虑Monkey语言中最简单的表达式：标识符。表达式语句中的标识符是这样的：
```js
foobar;
```
在其它上下文中的标识符是这样的：
```js
add(foobar, barfoo);
foobar + barfoo;
if (foobar) {
    // [...]
}
```

从测试用例开始：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_identifier_expression() {
    let input = "foobar;";

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                        "ident.token.literal not {}. got={}",
                        "foobar",
                        token.literal
                    );
                } else {
                    panic!("exp not Identifier. got={:?}", expression);
                }
            } else {
                panic!(
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```
上述代码的思想是解析一个表达式，从根上开始验证AST，是否是Program，是否是一条语句，是否是一条表达式语句，表达式是否是标识符，标识符内容是否正确。

当然，测试失败：
```
thread 'parser::tests::test_identifier_expression' panicked at 'program has not enough statements. got=0', src/parser/parser_test.rs:257:13
```

修改parse_statement方法：
```rust,noplaypen
// src/parse.rs

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.cur_token.r#type {
// [...]
            _ => Ok(self.parse_expression_statement()?),
        }
    }
```
增加parse_expression_statement方法
```rust,noplaypen
// src/parser/parse.rs

    fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();
        let expression = self.parse_expression(Precedence::LOWEST)?;
        if self.peek_token_is(&TokenType::SEMICOLON) {
            self.next_token();
        }
        Ok(Statement::ExpressionStatement(ExpressionStatement {
            token: token,
            expression: expression,
        }))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, String> {
        if let Some(prefix) = self.prefix_parse_fns.get(&self.cur_token.r#type) {
            let left_exp = prefix(self)?;
            return Ok(left_exp);
        }
        Err(self.no_prefix_parse_fn_error(&self.cur_token.r#type))
    }

    fn no_prefix_parse_fn_error(&self, t: &TokenType) -> String {
        format!("no prefix parse function for {:?} found", t)
    }
```
这里需要考虑运算符的优先级了，定义如下：
```rust,noplaypen
// src/parser/parser.rs

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
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::EOF, 0),
            peek_token: new_token(TokenType::EOF, 0),
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };
        p.register_prefix(TokenType::IDENT, |parser| parser.parse_identifier());
// [...]
    }
// [...]            
    fn parse_identifier(&mut self) -> Result<Expression, String> {
        Ok(Expression::Identifier(Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        }))
    }
```

测试通过！

## 整数字面量

Monkey中的整数字面量如下：
```js
5;
```
在其它上下文中的整数字面量如下：
```js
let x = 5;
add(5, 10);
5 + 5 + 5;
```

测试用例如下：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_integer_literal_expression() {
    let input = "5;";

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
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
                if let Expression::IntegerLiteral(IntegerLiteral { token, value }) = expression {
                    assert!(*value == 5, "literal.value not {}. got={}", 5, value);
                    assert!(
                        token.literal == "5",
                        "literal.token_literal not {}. got={}",
                        "5",
                        token.literal
                    );
                } else {
                    panic!("exp not IntegerLiteral. got={:?}", expression);
                }
            } else {
                panic!(
                    "program.statements[0] is not ExpressionStatement. got={:?}",
                    &statements[0]
                );
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```
与测试标识符的原理一致，测试整数字面量也需要这么多代码。

整数字面量的定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}
impl NodeTrait for IntegerLiteral {
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
    fn string(&self) -> String {
        match self {
// [...]
            Expression::IntegerLiteral(integer_literal) => integer_literal.string(),
        }
    }
}
```
这里用Rust的i64类型保存整数字面量的值。

新增解析代码：
```rust,noplaypen
// src/parser/parser.rs

    fn parse_integer_literal(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        if let Ok(value) = self.cur_token.literal.parse::<i64>() {
            Ok(Expression::IntegerLiteral(IntegerLiteral {
                token: token,
                value: value,
            }))
        } else {
            Err(format!(
                "could not parse {} as integer",
                self.cur_token.literal
            ))
        }
    }
```
测试结果失败！
```
thread 'parser::parser_test::test_integer_literal_expression' panicked at 'parser error: no prefix function for INT
no prefix function for SEMICOLON', src/parser/parser_test.rs:39:5
```

在parse_expression方法中增加对整数字面量的支持：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::IDENT, |parser| parser.parse_identifier());
        p.register_prefix(TokenType::INT, |parser| parser.parse_integer_literal());
// [...]
```
测试通过！

## 前缀操作符

Monkey中的前缀操作符是“!”和“-”，使用如下：
```js
-5;
!foobar;
5 + - 10;
```
结构表示：
```js
<prefix operator><expression>;
```

任何表达式都可以接在前缀操作符后面：
```js
!isGreaterThanZero(2);
5 + -add(5, 5);
```


测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

enum ExpectedType {
    Ival(i64),
    Sval(String),
    Bval(bool),
}
impl From<&str> for ExpectedType {
    fn from(v: &str) -> Self {
        ExpectedType::Sval(String::from(v))
    }
}

#[test]
fn test_parsing_prefix_expression() {
    let tests = vec![
        ("!5;", "!", ExpectedType::Ival(5)),
        ("-15;", "-", ExpectedType::Ival(15)),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(String::from(tt.0));
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

                        if let ExpectedType::Ival(i) = tt.2 {
                            test_integer_literal(right, i);
                        } else {
                            panic!("error");
                        }
                    } else {
                        panic!("stmt is not PrefixExpression. got={:?}", expression);
                    }
                } else {
                    panic!(
                        "program.statements[0] is not ExpressionStatement. got={:?}",
                        &statements[0]
                    );
                }
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}
```
为了提高测试代码的重用性，定义测试整数字面量的函数：
```rust,noplaypen
// src/parser/parser_test.rs

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
        panic!("il not IntegerLiteral. got={:?}", il);
    }
}
```
前缀表达式的定义如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Box<Expression>,
}
impl NodeTrait for PrefixExpression {
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
    fn string(&self) -> String {
        match self {
// [...]
            Expression::PrefixExpression(prefix_expr) => prefix_expr.string(),
        }
    }
}
```
这里使用Box\<Expression\>表示right的类型，是因为这是一个递归定义，必须借用Box来避免Rust编译器报错。

测试错误：
```
thread 'parser::parser_test::test_parsing_prefix_expression' panicked at 'parser error: no prefix function for BANG', src/parser/parser_test.rs:39:5
```

提示很明显，需要为“!”和“-”增加处理代码：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::INT, |parser| parser.parse_integer_literal());
        p.register_prefix(TokenType::BANG, |parser| parser.parse_prefix_expression());
        p.register_prefix(TokenType::MINUS, |parser| parser.parse_prefix_expression());
// [...]

    fn parse_prefix_expression(&mut self) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::PREFIX)?;

        Ok(Expression::PrefixExpression(PrefixExpression {
            token: token,
            operator: operator,
            right: Box::new(right),
        }))
    }
```
测试成功！

迄今为止，我们只使用了LOWEST一种优先级，下面我们会用到其它优先级。

## 中缀操作符

中缀操作符如下：
```js
5 + 5; 
5 - 5; 
5 * 5; 
5 / 5; 
5 > 5; 
5 < 5;
5 == 5; 
5 != 5;
```
结构如下：
```js
<expression><infix operator><expression>
```

先写测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_parsing_infix_expressions() {
    let tests = vec![
        ("5 + 5;", ExpectedType::Ival(5), "+", ExpectedType::Ival(5)),
        ("5 - 5;", ExpectedType::Ival(5), "-", ExpectedType::Ival(5)),
        ("5 * 5;", ExpectedType::Ival(5), "*", ExpectedType::Ival(5)),
        ("5 / 5;", ExpectedType::Ival(5), "/", ExpectedType::Ival(5)),
        ("5 > 5;", ExpectedType::Ival(5), ">", ExpectedType::Ival(5)),
        ("5 < 5;", ExpectedType::Ival(5), "<", ExpectedType::Ival(5)),
        (
            "5 == 5;",
            ExpectedType::Ival(5),
            "==",
            ExpectedType::Ival(5),
        ),
        (
            "5 != 5;",
            ExpectedType::Ival(5),
            "!=",
            ExpectedType::Ival(5),
        ),
    ];

    for tt in tests.iter() {
        let l = Lexer::new(String::from(tt.0));
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
                    if let Expression::InfixExpression(InfixExpression {
                        token: _,
                        left,
                        operator,
                        right,
                    }) = expression
                    {
                        if let ExpectedType::Ival(i) = tt.1 {
                            test_integer_literal(left, i);
                        } else {
                            panic!("error");
                        }
                        assert!(
                            operator == tt.2,
                            "exp.operator is not '{}. got={}",
                            tt.2,
                            operator
                        );

                        if let ExpectedType::Ival(i) = tt.3 {
                            test_integer_literal(right, i);
                        } else {
                            panic!("error");
                        }                    
                    } else {
                        panic!("exp is not InfixExpression. got={:?}", expression);
                    }
                } else {
                    panic!(
                        "program.statements[0] is not ExpressionStatement. got={:?}",
                        &statements[0]
                    );
                }
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}
```
跟前缀表达式不同，这里需要验证left和right两个分支。

定义中缀表达式：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub operator: String,
    pub right: Box<Expression>,
}
impl NodeTrait for InfixExpression {
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
', src/parser/parser_test.rs:298:9
```
提示我们没有处理“+”号的代码。


这里用到了优先级，需要实现一个Token的优先级查表：
```rust,noplaypen
// src/parser/parser.rs

    fn peek_precedence(&self) -> Precedence {
        get_precedence(&self.peek_token.r#type)
    }
    fn cur_precedence(&self) -> Precedence {
        get_precedence(&self.cur_token.r#type)
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
// src/parser/parser.rs

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression, String> {
        let token = self.cur_token.clone();
        let operator = self.cur_token.literal.clone();
        let precedence = self.cur_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Ok(Expression::InfixExpression(InfixExpression {
            token: token,
            left: Box::new(left),
            operator: operator,
            right: Box::new(right),
        }))
    }
```
与前缀表达式解析不同的是，中缀表达式的左子表达式已经在方法外面解析完成，通过参数传递进来，本方法中解析右子表达式，然后组合成中缀表达式节点返回。

需要修改parse_expression支持中缀表达式：
```rust,noplaypen
// src/parser/parser.rs

impl Parser {
    pub fn new(l: Lexer) -> Parser {
// [...]
        p.register_prefix(TokenType::MINUS, |parser| parser.parse_prefix_expression());

        p.register_infix(TokenType::PLUS, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::MINUS, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::SLASH, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::ASTERISK, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::EQ, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::NOTEQ, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::LT, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
        p.register_infix(TokenType::GT, |parser, exp| {
            parser.parse_infix_expression(exp)
        });
// [...]
```
此部分代码是普拉特解析器的核心，稍后我们会做解释。

```rust,noplaypen
// src/parser/parser.rs

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, String> {
        if let Some(prefix) = self.prefix_parse_fns.get(&self.cur_token.r#type) {
            let mut left_exp = prefix(self)?;
            while !self.peek_token_is(&TokenType::SEMICOLON) && precedence < self.peek_precedence()
            {
                let infix_fn: InfixParseFn;
                if let Some(infix) = self.infix_parse_fns.get(&self.peek_token.r#type) {
                    infix_fn = *infix;
                } else {
                    return Ok(left_exp);
                }
                self.next_token();
                left_exp = infix_fn(self, left_exp)?;
            }
            Ok(left_exp)
        } else {
            Err(self.no_prefix_parse_fn_error(&self.cur_token.r#type))
        }
    }
```
由于这里有precedence的比较，需要修改Precedence的定义，加上PartialOrd和PartialEq属性，由此产生的优先级排序正好满足我们的需求。
```rust,noplaypen
// src/parser/parser.rs

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

写一个测试操作符优先级的用例：
```rust,noplaypen
// src/parser/parser_test.rs

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
        let l = Lexer::new(String::from(tt.0));
        let mut p = Parser::new(l);
        match p.parse_program() {
            Ok(program) => {
                let actual = program.string();
                assert!(actual == tt.1, "expected={:?}, got={:?}", tt.1, actual);
            }
            Err(errors) => panic_with_errors(errors),
        }
    }
}
```
测试通过！

那普拉特解析器是如何工作的呢？
