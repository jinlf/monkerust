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
// src/ast.rs

#[derive(Debug)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Expression,
}
impl NodeTrait for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}
```
将RetrunStatement加入Statement：
```rust,noplaypen
// src/ast.rs

#[derive(Debug)]
pub enum Statement {
    LetStatement(LetStatement),
    ReturnStatement(ReturnStatement),
}
impl NodeTrait for Statement {
    fn token_literal(&self) -> String {
        match self {
            Statement::LetStatement(let_stmt) => let_stmt.token_literal(),
            Statement::ReturnStatement(return_stmt) => return_stmt.token_literal(),
        }
    }
}
```

先写测试用例：
```rust,noplaypen
// src/parser_test.rs

#[test]
fn test_return_statement() {
    let input = "
return 5;
return 10;
return 993322;
";
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    check_parser_errors(&mut p);

    if let Some(Program { statements }) = program {
        assert!(
            statements.len() == 3,
            "program.statements does not contain 3 statements. got={}",
            statements.len()
        );

        for stmt in statements.iter() {
            if let Statement::ReturnStatement(_) = stmt {
                assert!(
                    stmt.token_literal() == "return",
                    "returnStmt.token_literal not 'return', got={}",
                    stmt.token_literal()
                );
            } else {
                assert!(false, "stmt not ReturnStatement. got={:?}", stmt);
            }
        }
    } else {
        assert!(false, "parse error");
    }
}
```
测试必然失败，信息如下：
```
thread 'parser::tests::test_return_statement' panicked at 'program.statements does not contain 3 statements. got=0', src/parser_test.rs:192:13
```

修改parse_statement来支持return语句。
```rust,noplaypen
// src/parser.rs

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token.tk_type {
            TokenType::LET => {
                if let Some(stmt) = self.parse_let_statement() {
                    return Some(Statement::LetStatement(stmt));
                }
                None
            }
            TokenType::RETURN => {
                if let Some(stmt) = self.parse_return_statement() {
                    return Some(Statement::ReturnStatement(stmt));
                }
                None
            }
            _ => None,
        }
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement> {
        let token = self.cur_token.clone();
        self.next_token();

        // TODO: 我们将跳过分号之前的表达式
        while !self.cur_token_is(TokenType::SEMICOLON) {
            if self.cur_token_is(TokenType::EOF) {
                return None;
            }
            self.next_token();
        }
        Some(ReturnStatement {
            token: token,
            return_value: Expression::MockExpression { v: 0 },
        })
    }
```
这里的return_value仍然用MockExpression。

测试通过！

接下来看看我们怎样解析表达式。