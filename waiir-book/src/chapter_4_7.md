# return语句

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
修改如下：
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
Program和BlockStatement中处理return是不一样的。

测试通过！