# return语句

return语句的行为是停止对一系列语句的求值，返回值作为最后的求值结果。

例如：
```js
5 * 5 * 5; 
return 10; 
9 * 9 * 9;
```
上述三条语句的求值结果为10。表达式9 * 9 * 9不会被求值。

定义如下：
```rust,noplaypen
// src/object.rs

#[derive(Debug, PartialEq, Eq)]
pub struct ReturnValue {
    pub value: Box<Object>,
}
impl ObjectTrait for ReturnValue {
    fn get_type(&self) -> String {
        String::from("RETURN_VALUE")
    }
    fn inspect(&self) -> String {
        self.value.inspect()
    }
}

pub enum Object {
// [...]
    ReturnValue(ReturnValue),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
// [...]
            Object::ReturnValue(rv) => rv.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
// [...]
            Object::ReturnValue(rv) => rv.inspect(),
        }
    }
}
```
增加测试用例：
```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_return_statements() {
    let tests = [
        ("return 10;", 10),
        ("return 10; 9;", 10),
        ("return 2 * 5; 9;", 10),
        ("9; return 2 * 5; 9;", 10),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        test_integer_object(evaluated, tt.1);
    }
}
```

增加一条求值return语句的分支：
```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Statement(Statement::ReturnStatement(ReturnStatement {
            token: _,
            return_value,
        })) => {
            let val = eval(Node::Expression(return_value));
            if val.is_some() {
                Some(Object::ReturnValue(ReturnValue {
                    value: Box::new(val.unwrap()),
                }))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn eval_statements(stmts: Vec<Statement>) -> Option<Object> {
    let mut result: Option<Object> = None;
    for statement in stmts.iter() {
        result = eval(Node::Statement(statement.clone()));
        if let Some(Object::ReturnValue(ReturnValue { value })) = result {
            return Some(*value);
        }
    }
    result
}
```
需要注意的是，遇到return语句时，返回的是语句中表达式的值，而不是return本身。

增加一个测试用例：
```rust,noplaypen
// src/evaluator_test.rs

fn test_return_statements() {
    let tests = [
// [...]
        (
            "if (10 > 1) {
                if (10 > 1) {
                    return 10;
                }
                return 1;
            }",
            10,
        ),
    ];
// [...]
```
测试结果：
```
thread 'evaluator::tests::test_return_statements' panicked at 'object has wrong value. got=1, want=10', src/evaluator_test.rs:224:13
```
失败的原因是，嵌套块语句中包含retrun求值时，应该在最外层终止并返回。前面实现的eval_statements仅适用于Program这种最外层语句的情况。

所以我们将刚刚修改过的eval_statements改名为eval_program，并修改参数类型，如下：
```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
        Node::Program(_) => eval_program(node),
// [...]        
    }
}

fn eval_program(node: Node) -> Option<Object> {
    let mut result: Option<Object> = None;
    if let Node::Program(Program { statements }) = node {
        for statement in statements.iter() {
            result = eval(Node::Statement(statement.clone()));
            if let Some(Object::ReturnValue(ReturnValue { value })) = result {
                return Some(*value);
            }
        }
    }
    result
}
```

我们增加一个新函数eval_block_statement来支持对允许嵌套的块语句求值，如下：
```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]        
        Node::Statement(Statement::BlockStatement(block)) => eval_block_statement(block),
// [...]
    }
}

fn eval_block_statement(block: BlockStatement) -> Option<Object> {
    let mut result: Option<Object> = None;
    for statement in block.statements.iter() {
        result = eval(Node::Statement(statement.clone()));

        if let Some(Object::ReturnValue(_)) = result {
            return result;
        }
    }
    result
}
```
与Program不同，这里return求值的结果对象是ReturnValue，而不是内部的Expression。

测试通过！