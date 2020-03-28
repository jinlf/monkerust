# 条件

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_if_else_expression() {
    let tests: [(&str, Option<Object>); 7] = [
        (
            "if (true) { 10 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
        ("if (false) { 10 }", None),
        (
            "if (1) { 10 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
        (
            "if (1 < 2) { 10 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
        ("if (1 > 2) { 10 }", None),
        (
            "if (1 > 2) { 10 } else { 20 }",
            Some(Object::Integer(Integer { value: 20 })),
        ),
        (
            "if (1 < 2) { 10 } else { 20 }",
            Some(Object::Integer(Integer { value: 10 })),
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Some(Object::Integer(Integer { value })) = tt.1 {
            test_integer_object(evaluated, value);
        } else {
            test_null_object(evaluated);
        }
    }
}

fn test_null_object(obj: Option<Object>) {
    assert!(obj == Some(Object::Null(NULL)), "object is not NULL, got={:?}", obj);
}
```

测试结果

```
thread 'evaluator::tests::test_if_else_expression' panicked at 'object is not Integer. got=None', src/evaluator_test.rs:188:13
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Statement(Statement::BlockStatement(BlockStatement {
            token: _,
            statements,
        })) => eval_statements(statements),
        Node::Expression(Expression::IfExpression(if_expr)) => eval_if_expression(if_expr),
        _ => None,
    }
}

fn eval_if_expression(ie: IfExpression) -> Option<Object> {
    let condition = eval(Node::Expression(*ie.condition));
    if is_truthy(&condition) {
        return eval(Node::Statement(Statement::BlockStatement(ie.consequence)));
    } else if ie.alternative.is_some() {
        return eval(Node::Statement(Statement::BlockStatement(
            ie.alternative.unwrap(),
        )));
    } else {
        Some(Object::Null(NULL))
    }
}

fn is_truthy(obj: &Option<Object>) -> bool {
    match obj {
        Some(Object::Null(NULL)) => false,
        Some(Object::BooleanObj(TRUE)) => true,
        Some(Object::BooleanObj(FALSE)) => false,
        _ => true,
    }
}
```

测试通过！

用 cargo run 执行

```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> if (5 * 5 + 10 > 34) { 99 } else { 100 }
99
>> if ((1000 / 2) + 250 * 2 == 1000) { 9999 }
9999
>>
```
