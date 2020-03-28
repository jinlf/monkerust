# 语法分析器第一步：解析let语句

定义抽象语法树节点：
```rust,noplaypen
// src/ast.rs

pub trait NodeTrait {
    fn token_literal(&self) -> String;
}

pub enum Node {
    Statement(Statement),
    Expression(Expression),
}
pub enum Statement {}
impl NodeTrait for Statement {
    fn token_literal(&self) -> String {
        String::new() //TODO
    }
}

pub enum Expression {}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        String::new() //TODO
    }
}
```

注意Rust语言的Trait与其它语言的接口不完全是同一个概念，因此向下类型转换就不是很方便，不能用传统的C++、Go这类语言的方式实现相同的功能。

定义Program：
```rust,noplaypen
// src/ast.rs 

pub struct Program {
    pub statements: Vec<Statement>,
}
impl NodeTrait for Program {
    fn token_literal(&self) -> String {
        if self.statements.len() > 0 {
            self.statements[0].token_literal()
        } else {
            String::new()
        }
    }
}
```
在Node枚举中加入Program：
```rust,noplaypen
// src/ast.rs

pub enum Node {
    Program(Program),
    Statement(Statement),
    Expression(Expression),
}
```
定义let语句：
```rust,noplaypen
// src/ast.rs

use super::token::*;

pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}
impl NodeTrait for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

pub struct Identifier {
    pub token: Token,
    pub value: String,
}
impl NodeTrait for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}
```
将LetStatement加入Statement，将Identifier加入Expression，如下：
```rust,noplaypen
// src/ast.rs

pub enum Statement {
    LetStatement(LetStatement),
}
impl NodeTrait for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.token_literal(),
        }
    }
}
pub enum Expression {
    Identifier(Identifier),
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.token_literal(),
        }
    }
}
```

下面我们开始编码语法分析器。
```rust,noplaypen
// src/parser.rs

use super::ast::*;
use super::lexer::*;
use super::token::*;

pub struct Parser<'a> {
    pub l: Lexer<'a>,
    pub cur_token: Token,
    pub peek_token: Token,
}
impl<'a> Parser<'a> {
    pub fn new(l: Lexer<'a>) -> Parser<'a> {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::ILLEGAL, 0),
            peek_token: new_token(TokenType::ILLEGAL, 0),
        };
        p.next_token();
        p.next_token();
        p
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.l.next_token();
    }

    pub fn parse_program(&mut self) -> Option<Program> {
        None
    }
}
```
由于这里需要实现Token的clone方法，需要修改Token和TokenType的属性：
```rust,noplaypen
// src/token.rs

#[derive(Debug, Clone)]
pub struct Token {
    pub tk_type: TokenType,
    pub literal: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {    
```

下面先写测试用例：
```rust,noplaypen
// src/parser_test.rs

use super::ast::*;
use super::lexer::*;
use super::parser::*;

#[test]
fn test_let_statements() {
    let input = "
let x = 5;
let y = 10;
let foobar = 838383;
";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 3,
            "program.statements does not contain 3 statements. got={}",
            statements.len()
        );

        let tests = ["x", "y", "foobar"];
        for (i, tt) in tests.iter().enumerate() {
            test_let_statement(&statements[i], tt);
        }
    } else {
        assert!(false, "parse_program() returned None");
    }
}

fn test_let_statement(s: &Statement, expected_name: &str) {
    assert!(
        s.token_literal() == "let",
        "s.token_literal not 'let'. got={}",
        s.token_literal()
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
            name.token_literal() == expected_name,
            "s.name not '{}'. got={}",
            expected_name,
            name.token_literal()
        );
    } else {
        assert!(false, "s not LetStatement. got={:?}", s);
    }
}
```
测试中输出了Statement，因此需要添加Statement的Debug属性，进而需要添加Expression, LetStatement和Identifier的Debug属性：
```rust,noplaypen
// src/ast.rs

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
为了支持测试，需要修改lib.rs，加入下面两行。
```rust,noplaypen
// src/lib.rs

pub mod ast;
pub mod parser;

mod parser_test;
```
测试失败：
```
thread 'parser::tests::test_let_statements' panicked at 'parse_program() returned None', src/parser_test.rs:59:13
```
因为parse_program还没实现呢。

实现如下：
```rust,noplaypen
// src/parser.rs

    pub fn parse_program(&mut self) -> Option<Program> {
        let mut statements: Vec<Statement> = Vec::new();

        while self.cur_token.tk_type != TokenType::EOF {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }

        Some(Program {
            statements: statements,
        })
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.tk_type {
            TokenType::LET => {
                if let Some(stmt) = self.parse_let_statement() {
                    return Some(Statement::LetStatement(stmt));
                }
                None
            }
            _ => None,
        }
    }

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

        // TODO: We're skipping the expressions until we 
        // encounter a semicolon
        while !self.cur_token_is(TokenType::SEMICOLON) {
            if self.cur_token_is(TokenType::EOF) {
                return None;
            }
            self.next_token();
        }

        Some(LetStatement {
            token: token,
            name: name,
            value: Expression::MockExpression { v: 0 },
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
            false
        }
    }
```
由于LetStatement的value是Expression，而到目前为止，我们还没有能解析Expression的能力，所以我这里做了一个占位用的MockExpression定义，未来需要移除，代码如下：
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub enum Expression {
    Identifier(Identifier),
    MockExpression { v: i32 }, //TODO remove
}
impl NodeTrait for Expression {
    fn token_literal(&self) -> String {
        match self {
            Expression::Identifier(ident) => ident.token_literal(),
            Expression::MockExpression { v: _ } => String::new(), //TODO remove
        }
    }
}
```
测试通过！

下面加一些错误处理的能力。
```rust,noplaypen
// src/parser.rs

pub struct Parser<'a> {
    pub l: Lexer<'a>,
    pub cur_token: Token,
    pub peek_token: Token,
    pub errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(l: Lexer<'a>) -> Parser<'a> {
        let mut p = Parser {
            l: l,
            cur_token: new_token(TokenType::ILLEGAL, 0),
            peek_token: new_token(TokenType::ILLEGAL, 0),
            errors: Vec::new(),
        };
// [...]
    }

    fn peek_error(&mut self, t: TokenType) {
        let msg = format!(
            "expected next token to be {:?}, got {:?} instead",
            t, self.peek_token.tk_type
        );
        self.errors.push(msg);
    }
}
```
这样，测试用例就可以改为：
```rust,noplaypen
// src/parser_test.rs

fn test_let_statements() {
// [...]
    let program = p.parse_program();
    check_parser_errors(&mut p);

// [...]
}

fn check_parser_errors(p: &mut Parser) {
    if p.errors.len() == 0 {
        return;
    }

    let mut msgs = String::from(format!("parser has {} errors\n", p.errors.len()));
    for msg in p.errors.iter() {
        msgs.push_str(&format!("parser error: {:?}\n", msg));
    }
    assert!(false, msgs);
}
```

expect_peek也需要修改一下：
```rust,noplaypen
// src/parser.rs

    fn expect_peek(&mut self, t: TokenType) -> bool {
        if self.peek_token_is(t.clone()) {
            self.next_token();
            true
        } else {
            self.peek_error(t);
            false
        }
    }
```

为了测试一下解析错误的情况，可以临时修改测试用例：
```rust,noplaypen
// src/parser.rs

    fn test_let_statements() {
        let input = "
let x 5;
let = 10;
let 838383;
";
```
测试得到的结果如下：
```
thread 'parser::tests::test_let_statements' panicked at 'parser has 3 errors
parser error: "expected next token to be ASSIGN, got INT instead"
parser error: "expected next token to be IDENT, got ASSIGN instead"
parser error: "expected next token to be IDENT, got INT instead"
```
我们刚刚加的错误处理起效了。

