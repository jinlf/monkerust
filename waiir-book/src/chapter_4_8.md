# 错误处理

```rust,noplaypen
// src/object.rs

#[derive(Debug, PartialEq, Eq)]
pub struct ErrorObj {
    pub message: String,
}
impl ObjectTrait for ErrorObj {
    fn get_type(&self) -> String {
        String::from("ERROR")
    }
    fn inspect(&self) -> String {
        format!("ERROR: {}", self.message)
    }
}

pub enum Object {
// [...]
    ErrorObj(ErrorObj),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
// [...]
            Object::ErrorObj(e) => e.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
// [...]
            Object::ErrorObj(e) => e.inspect(),
        }
    }
}
```

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_error_handling() {
    let tests = [
        ("5 + true;", "type mismatch: INTEGER + BOOLEAN"),
        ("5 + true; 5;", "type mismatch: INTEGER + BOOLEAN"),
        ("-true", "unknown operator: -BOOLEAN"),
        ("true + false;", "unknown operator: BOOLEAN + BOOLEAN"),
        ("5; true + false; 5", "unknown operator: BOOLEAN + BOOLEAN"),
        (
            "if (10 > 1) { true + false; }",
            "unknown operator: BOOLEAN + BOOLEAN",
        ),
        (
            "
if (10 > 1) { 
if (10 > 1) {
    return true + false;
}
return 1;
}",
            "unknown operator: BOOLEAN + BOOLEAN",
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Some(Object::ErrorObj(ErrorObj { message })) = evaluated {
            assert!(
                message == tt.1,
                "wrong error message. expected={}, got={}",
                tt.1,
                message
            );
        } else {
            assert!(false, "no error object returned. got={:?}", evaluated);
        }
    }
}
```
测试结果
```
thread 'evaluator::tests::test_error_handling' panicked at 'no error object returned. got=Some(Null(Null))', src/evaluator_test.rs:409:17
```

```rust,noplaypen
// src/evaluator.rs

pub fn new_error(msg: String) -> Option<Object> {
    Some(Object::ErrorObj(ErrorObj { message: msg }))
}
```

```rust,noplaypen
// src/evaluator.rs

pub fn get_type(obj: &Option<Object>) -> String {
    if obj.is_some() {
        obj.as_ref().unwrap().get_type()
    } else {
        String::from("None")
    }
}

fn eval_prefix_expression(operator: &str, right: Option<Object>) -> Option<Object> {
    match operator {
// [...]
        _ => new_error(format!(
            "unknown operator: {}{}",
            operator,
            get_type(&right)
        )),
    }
}

fn eval_minus_prefix_operator_expression(right: Option<Object>) -> Option<Object> {
    if let Some(Object::Integer(Integer { value })) = right {
        Some(Object::Integer(Integer { value: -value }))
    } else {
        new_error(format!("unknown operator: -{}", get_type(&right)))
    }
}

fn eval_infix_expression(
    operator: &str,
    left: Option<Object>,
    right: Option<Object>,
) -> Option<Object> {
    if get_type(&left) != get_type(&right) {
        return new_error(format!(
            "type mismatch: {} {} {}",
            get_type(&left),
            operator,
            get_type(&right)
        ));
    }
// [...]
    if let Some(left_obj) = left {
        if let Some(right_obj) = right {
            return match operator {
// [...]
                _ => new_error(format!(
                    "unknown operator: {} {} {}",
                    left_obj.get_type(),
                    operator,
                    right_obj.get_type(),
                )),
            };
        }
    }
    Some(Object::Null(NULL))
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Option<Object> {
    match operator {
// [...]
        _ => new_error(format!("unknown operator: INTEGER {} INTEGER", operator)),
    }
}
```
测试结果
```
thread 'evaluator::tests::test_error_handling' panicked at 'no error object returned. got=Some(Integer(Integer { value: 5 }))', src/evaluator_test.rs:438:17
```

```rust,noplaypen
// src/evaluator.rs

fn eval_program(node: Node) -> Option<Object> {
    let mut result: Option<Object> = None;
    if let Node::Program(Program { statements }) = node {
        for statement in statements.iter() {
            result = eval(Node::Statement(statement.clone()));
            if let Some(Object::ReturnValue(ReturnValue { value })) = result {
                return Some(*value);
            } else if let Some(Object::ErrorObj(_)) = result {
                return result;
            }
        }
    }
    result
}

fn eval_block_statement(block: BlockStatement) -> Option<Object> {
    let mut result: Option<Object> = None;
    for statement in block.statements.iter() {
        result = eval(Node::Statement(statement.clone()));

        if let Some(Object::ReturnValue(_)) = result {
            return result;
        } else if let Some(Object::ErrorObj(_)) = result {
            return result;
        }
    }
    result
}
```

```rust,noplaypen
// src/evaluator.rs

fn is_error(obj: &Option<Object>) -> bool {
    if let Some(Object::ErrorObj(_)) = obj {
        true
    } else {
        false
    }
}

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::PrefixExpression(PrefixExpression {
            token: _,
            operator,
            right,
        })) => {
            let right_obj = eval(Node::Expression(*right));
            if is_error(&right_obj) {
                return right_obj;
            }
            eval_prefix_expression(&operator, right_obj)
        }
        Node::Expression(Expression::InfixExpression(InfixExpression {
            token: _,
            left,
            operator,
            right,
        })) => {
            let left_obj = eval(Node::Expression(*left));
            if is_error(&left_obj) {
                return left_obj;
            }
            let right_obj = eval(Node::Expression(*right));
            if is_error(&right_obj) {
                return right_obj;
            }
            eval_infix_expression(&operator, left_obj, right_obj)
        }
// [...]
        Node::Statement(Statement::ReturnStatement(ReturnStatement {
            token: _,
            return_value,
        })) => {
            let val = eval(Node::Expression(return_value));
            if is_error(&val) {
                return val;
            }
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

fn eval_if_expression(ie: IfExpression) -> Option<Object> {
    let condition = eval(Node::Expression(*ie.condition));
    if is_error(&condition) {
        return condition;
    }
// [...]
}
```
测试通过！