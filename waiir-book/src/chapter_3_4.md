# 解析器第一步：解析let语句

在Monkey语言中，变量绑定语句形式如下：
```js
let x = 5;
let y = 10;
let foobar = add(5, 5);
let barfoo = 5 * 5 / 10 + 18 - add(5, 5) + multiply(124); 
let anotherName = barfoo;
```
这种语句称为“let语句”，它把变量值绑定到一个名字上。

正确工作的解析器将构造一个符合原始语句信息的抽象语法树。

定义抽象语法树节点：
```rust,noplaypen
// src/ast/mod.rs

mod ast;
pub use ast::*;
```

```rust,noplaypen
// src/ast/ast.rs

pub trait NodeTrait {
    fn string(&self) -> String;
}

pub enum Node {
    Statement(Statement),
    Expression(Expression),
}

pub enum Statement {}
impl NodeTrait for Statement {
    fn string(&self) -> String {
        String::new()   //TODO
    }
}

pub enum Expression {}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        String::new()   //TODO
    }
}
```
Rust语言具有强大的enum和match匹配能力，本文用enum来定义Node、Statement和Expression，以便用match语句进行类型匹配。

上述代码中的NodeTrait规定了所有Node的共同特征，具有string方法，该方法返回当前Node节点转成的字符串值。Monkey语言AST中节点Node可以是Statement，也可以是Expression。下面再加上一种：Program节点。

定义Program：
```rust,noplaypen
// src/ast/ast.rs 

pub struct Program {
    pub statements: Vec<Statement>,
}
impl NodeTrait for Program {
    fn string(&self) -> String {
        let mut out = String::new();
        for s in self.statements.iter() {
            out.push_str(&s.string());
        }
        out
    }
}
```
在Node枚举中加入Program：
```rust,noplaypen
// src/ast/ast.rs

pub enum Node {
    Program(Program),
    Statement(Statement),
    Expression(Expression),
}
```
Program节点是所有AST的根节点，包含一系列Statement节点。

下面定义let语句：
```rust,noplaypen
// src/ast/ast.rs

use crate::token::*;
// [...]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}
impl NodeTrait for LetStatement {
    fn string(&self) -> String {
        format!(
            "{} {} = {};",
            self.token.literal,
            self.name.string(),
            self.value.string(),
        )
    }
}

pub struct Identifier {
    pub token: Token,
    pub value: String,
}
impl NodeTrait for Identifier {
    fn string(&self) -> String {
        self.value.clone()
    }
}
```
LetStatement节点包含name和value两个成员，其中name表示绑定的标识符，value表示产生值的表达式。Identifier节点包含的value即标识符的名字。

将LetStatement加入Statement，将Identifier加入Expression，如下：
```rust,noplaypen
// src/ast/ast.rs

pub enum Statement {
    LetStatement(LetStatement),
}
impl NodeTrait for Statement {
    fn string(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.string(),
        }
    }
}
pub enum Expression {
    Identifier(Identifier),
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.string(),
        }
    }
}
```
其实Rust支持将匿名结构体作为枚举成员的类型，例如：
```rust,noplaypen
pub enum Statement {
    LetStatement {  
        token: Token,
        name: Identifier,
        value: Expression,
    },
}
```
这种方式在match匹配时会省去一些代码输入工作，但考虑到做成匿名结构体后无法直接按类型访问，在无法确定匿名方式是否合适的情况下，本文统一使用结构体和枚举定义分离的方式。

定义完上述三种节点，则Monkey源代码：
```js
let x = 5;
```
将形成以下AST：

![AST](image/f3-1.png "一棵AST")


下面我们开始编码解析器：
```rust,noplaypen
// src/parser/mod.rs

mod parser;
pub use parser::*;
```

```rust,noplaypen
// src/parser/parser.rs

use crate::ast::*;
use crate::lexer::*;
use crate::token::*;

pub struct Parser {
    pub l: Lexer,
    pub cur_token: Token,
    pub peek_token: Token,
}
impl Parser {
    pub fn new(l: Lexer) -> Parser {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::EOF, 0),
            peek_token: new_token(TokenType::EOF, 0),
        };
        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.cur_token = std::mem::replace(&mut self.peek_token, self.l.next_token());
    }

    pub fn parse_program(&mut self) -> Result<Program, Vec<String>> {
        Ok(Program {
            statements: Vec::new(),
        })
    }
}
```
解析器有三个成员：词法分析器，当前Token，下一个Token。next_token方法的功能就是从词法分析器中读取Token并更新当前Token和下一个Token。

在创建解析器时调用了两次next_token方法，是为了初始化当前Token和下一个Token。这里您不用担心input是否够用的问题，词法分析器在input结尾处会一直返回EOF，而不会报错。

在继续进一步工作前，先用伪代码解释一下解析器的工作原理：
```js
function parseProgram() { 
    program = newProgramASTNode()

    advanceTokens()

    for (currentToken() != EOF_TOKEN) {
        statement = null

        if (currentToken() == LET_TOKEN) { 
            statement = parseLetStatement()
        } else if (currentToken() == RETURN_TOKEN) { 
            statement = parseReturnStatement()
        } else if (currentToken() == IF_TOKEN) { 
            statement = parseIfStatement()
        }

        if (statement != null) { 
            program.Statements.push(statement)
        }
    
        advanceTokens() 
    }
    return program 
}

function parseLetStatement() { 
    advanceTokens()

    identifier = parseIdentifier()

    advanceTokens()

    if currentToken() != EQUAL_TOKEN { 
        parseError("no equal sign!") 
        return null
    }

    advanceTokens()

    value = parseExpression()
    variableStatement = newVariableStatementASTNode() 
    variableStatement.identifier = identifier 
    variableStatement.value = value
    return variableStatement
}

function parseIdentifier() { 
    identifier = newIdentifierASTNode() 
    identifier.token = currentToken() 
    return identifier
}

function parseExpression() {
    if (currentToken() == INTEGER_TOKEN) {
        if (nextToken() == PLUS_TOKEN) {    
            return parseOperatorExpression()
        } else if (nextToken() == SEMICOLON_TOKEN) { 
            return parseIntegerLiteral()
        }
    } else if (currentToken() == LEFT_PAREN) {
        return parseGroupedExpression() 
    }
// [...]
}

function parseOperatorExpression() {
    operatorExpression = newOperatorExpression()
    operatorExpression.left = parseIntegerLiteral() 
    operatorExpression.operator = currentToken() 
    operatorExpression.right = parseExpression()
    return operatorExpression() 
}
// [...]
```
上述伪代码的基本思想是递归下降解析。入口是parseProgram，用来构造AST的根节点，然后调用它的子节点解析函数，构造各个Statement。这些解析函数再调用它们子节点的解析函数，递归下去，解析完成时返回的是整个AST。

下面我们继续开发，先写测试用例：
```rust,noplaypen
// src/parser/mod.rs

#[cfg(test)]
mod parser_test;
```

```rust,noplaypen
// src/parser/parser_test.rs

use crate::ast::*;
use crate::lexer::*;
use crate::parser::*;

#[test]
fn test_let_statements() {
    let input = "
let x = 5;
let y = 10;
let foobar = 838383;
";
    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(Program { statements }) => {
            assert!(
                statements.len() == 3,
                "program.statements does not contain 3 statements. got={}",
                statements.len()
            );
            let tests = ["x", "y", "foobar"];
            for (i, tt) in tests.iter().enumerate() {
                test_let_statement(&statements[i], tt);
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}

fn panic_with_errors(errors: Vec<String>) {
    let mut messages = Vec::new();
    for msg in errors.into_iter() {
        messages.push(msg);
    }
    panic!("parser error: {}", messages.join("\n"));
}

fn test_let_statement(s: &Statement, expected_name: &str) {
    assert!(
        &s.string()[0..3] == "let",
        "s.token.literal not 'let'. got={}",
        s.string()
    );
    if let Statement::LetStatement(LetStatement {
        token: _,
        name,
        value: _,
    }) = s {
        assert!(
            name.value == expected_name,
            "letStmt.name.value not '{}', got={}",
            expected_name,
            name.value
        );
        assert!(
            name.token.literal == expected_name,
            "s.name not '{}'. got={}",
            expected_name,
            name.token.literal
        );
    } else {
        panic!("s not LetStatement. got={:?}", s);
    }
}
```
测试用例的思想是递归下降验证解析出来的AST，跟预期的AST是否一致。

上述代码还使用了Rust语言中的if let语句，这种语句可以匹配展开复杂的数据结构，酷！

测试中需要打印输出Statement，因此需要添加Statement的Debug属性，进而需要添加Expression, LetStatement和Identifier的Debug属性：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub enum Statement {
    LetStatement(LetStatement),
}

#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
}

#[derive(Debug)]
pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}
```
为了支持测试，需要修改main.rs，加入下面几行。
```rust,noplaypen
// src/main.rs

mod ast;
mod parser;
```
测试失败的信息如下：
```
thread 'parser::parser_test::test_let_statements' panicked at 'program.statements does not contain 3 statements. got=0', src/parser/parser_test.rs:18:13
```
必然的，因为parse_program还没实现呢。

实现如下：
```rust,noplaypen
// src/parser/parser.rs

    pub fn parse_program(&mut self) -> Result<Program, Vec<String>> {
        let mut statements: Vec<Statement> = Vec::new();
        let mut errors = Vec::new();
        while self.cur_token.r#type != TokenType::EOF {
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    if err != "" {
                        errors.push(err)
                    }
                }
            }
            self.next_token();
        }
        if errors.len() != 0 {
            return Err(errors);
        }

        Ok(Program {
            statements: statements,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.cur_token.r#type {
            TokenType::LET => self.parse_let_statement(),
            _ => Err(String::new()),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();

        self.expect_peek(&TokenType::IDENT)?;

        let name = Identifier {
            token: self.cur_token.clone(),
            value: self.cur_token.literal.clone(),
        };

        self.expect_peek(&TokenType::ASSIGN)?;

        self.next_token();

        // TODO: 我们将跳过分号之前的表达式
        while !self.cur_token_is(&TokenType::SEMICOLON) {
            if self.cur_token_is(&TokenType::EOF) {
                return Err(String::new());
            }
            self.next_token();
        }

        Ok(Statement::LetStatement(LetStatement {
            token: token,
            name: name,
            value: Expression::MockExpression,
        }))
    }

    fn cur_token_is(&self, t: &TokenType) -> bool {
        &self.cur_token.r#type == t
    }
    fn peek_token_is(&self, t: &TokenType) -> bool {
        &self.peek_token.r#type == t
    }
    fn expect_peek(&mut self, t: &TokenType) -> Result<(), String> {
        if self.peek_token_is(t) {
            self.next_token();
            Ok(())
        } else {
            Err(String::new())
        }
    }
```
由于LetStatement的value是Expression，目前为止，我们还没有能解析Expression的能力，只能跳过，我这里做了一个占位用的MockExpression定义，未来需要移除，代码如下：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
    MockExpression, //TODO remove
}
impl NodeTrait for Expression {
    fn string(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.string(),
            Expression::MockExpression => String::new(), //TODO remove
        }
    }
}
```

由于这里需要调用Token的clone方法，修改Token和TokenType的属性如下：
```rust,noplaypen
// src/token/token.rs

#[derive(Debug, Clone)]
pub struct Token {
    pub r#type: TokenType,
    pub literal: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {   
// [...] 
```

测试通过！

大家可以比较一下用Rust实现的Program、Statement和LetStatement的解析过程，跟伪代码逻辑是一致的，只是暂时缺少解析其它类型语句的分支，我们会在后续的开发过程中逐渐补充。

为了更好地调试，我们在继续工作之前先为解析器加入一些错误处理的能力：
```rust,noplaypen
// src/parser/parser.rs

    fn peek_error(&mut self, t: &TokenType) -> String {
        format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.r#type
        )
    }
}
```
修改expect_peek，加上收集错误的代码调用：
```rust,noplaypen
// src/parser/parser.rs

    fn expect_peek(&mut self, t: &TokenType) -> Result<(), String> {
        if self.peek_token_is(t) {
            self.next_token();
            Ok(())
        } else {
            Err(self.peek_error(t))
        }
    }
```
注意，这里执行了一次t.clone()，因为根据Rust的安全限制，不能访问已经移走的对象，因此第一次访问t时候我使用的是克隆出来的对象。

为了测试一下解析错误的情况，可以临时修改测试用例：
```rust,noplaypen
// src/parser/parser.rs

    fn test_let_statements() {
        let input = "
let x 5;
let = 10;
let 838383;
";
```
测试得到的结果如下：
```
thread 'parser::parser_test::test_let_statements' panicked at 'parser error: expected next token to be ASSIGN, got INT instead
expected next token to be IDENT, got ASSIGN instead
expected next token to be IDENT, got INT instead', src/parser/parser_test.rs:38:5
```
我们刚刚加的错误处理起效了。

