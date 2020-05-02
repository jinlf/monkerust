# 解析return语句

Monkey中的return语句示例如下：
```js
return 5; 
return 10; 
return add(15);
```
其结构是：
```js
return <expression>;
```
定义return语句：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Expression,
}
impl NodeTrait for ReturnStatement {
    fn string(&self) -> String {
        format!("{} {};", self.token.literal, self.return_value.string())
    }
}
```
将RetrunStatement加入Statement：
```rust,noplaypen
// src/ast/ast.rs

#[derive(Debug)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
}
impl NodeTrait for Statement {
    fn string(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.string(),
            Statement::ReturnStatement(return_stmt) => return_stmt.string(),
        }
    }
}
```

先写测试用例：
```rust,noplaypen
// src/parser/parser_test.rs

#[test]
fn test_return_statement() {
    let input = "
return 5;
return 10;
return 993322;
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

            for stmt in statements.iter() {
                if let Statement::ReturnStatement(_) = stmt {
                    assert!(
                        &stmt.string()[..6] == "return",
                        "returnStmt.token.literal not 'return', got={}",
                        stmt.string(),
                    );
                } else {
                    panic!("stmt not ReturnStatement. got={:?}", stmt);
                }
            }
        }
        Err(errors) => panic_with_errors(errors),
    }
}
```
测试必然失败，信息如下：
```
thread 'parser::tests::test_return_statement' panicked at 'program.statements does not contain 3 statements. got=0', src/parser/parser_test.rs:192:13
```

修改parse_statement来支持return语句。
```rust,noplaypen
// src/parser/parser.rs

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.cur_token.r#type {
            TokenType::LET => self.parse_let_statement(),
            TokenType::RETURN => self.parse_return_statement(),
            _ => Err(String::new()),
        }
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        let token = self.cur_token.clone();
        self.next_token();

        // TODO: 我们将跳过分号之前的表达式
        while !self.cur_token_is(&TokenType::SEMICOLON) {
            if self.cur_token_is(&TokenType::EOF) {
                return Err(String::new());
            }
            self.next_token();
        }

        Ok(Statement::ReturnStatement(ReturnStatement {
            token: token,
            return_value: Expression::MockExpression,
        }))
    }
```
这里的return_value仍然用MockExpression。

测试通过！

接下来看看我们怎样解析表达式。