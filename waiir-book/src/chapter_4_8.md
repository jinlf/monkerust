# 错误处理

这里不是指Monkey语言支持错误处理，而是对解释器遇到内部错误时的处理。例如：非法操作符、不支持的操作或其它执行期间遇到的错误。

错误的处理方式跟Return语句的处理方式类似。遇到错误终止求值，返回错误对象。

错误对象的定义如下：
```rust
// src/object/object.rs

#[derive(Debug, PartialEq, Eq)]
pub struct ErrorObj {
    pub message: String,
}
impl ObjectTrait for ErrorObj {
    fn get_type(&self) -> &str {
        "ERROR"
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
为了防止跟Rust自带的Error类型名称冲突，这里使用ErrorObj。

这里实现的错误对象仅仅封装了错误消息。您可以试着加上文件名和行号这类信息。


测试用例：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

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
        if let Object::ErrorObj(ErrorObj { message }) = evaluated {
            assert!(
                message == tt.1,
                "wrong error message. expected={}, got={}",
                tt.1,
                message
            );
        } else {
            panic!("no error object returned. got={:?}", evaluated);
        }
    }
}
```
测试失败结果如下：
```
thread 'evaluator::tests::test_error_handling' panicked at 'no error object returned. got=Null(Null)', src/evaluator/evaluator_test.rs:409:17
```
先处理前缀表达式中使用了不支持的操作符错误：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_prefix_expression(operator: &str, right: Object) -> Result<Object, String> {
    match operator {
// [...]
        _ => Err(format!(
            "unknown operator: {}{}",
            operator,
            right.get_type(),
        )),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Result<Object, String> {
    if let Object::Integer(Integer { value }) = right {
        Ok(Object::Integer(Integer { value: -value }))
    } else {
        Err(format!("unknown operator: -{}", right.get_type()))
    }
}
```

中缀表达式除了非法操作符错误，还有左右子表达式类型不一致的错误：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_infix_expression(
    operator: &str,
    left: Object,
    right: Object,
) -> Result<Object, String> {
    if left.get_type() != right.get_type() {
        return Err(format!(
            "type mismatch: {} {} {}",
            left.get_type(),
            operator,
            right.get_type(),
        ));
    }
// [...]
    match operator {
// [...]
        _ => Err(format!(
            "unknown operator: {} {} {}",
            left.get_type(),
            operator,
            right.get_type(),
        )),
    }
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Result<Object, String> {
    match operator {
// [...]
        _ => Err(format!("unknown operator: INTEGER {} INTEGER", operator)),
    }
}
```

测试仍然失败：
```
thread 'evaluator::evaluator_test::test_error_handling' panicked at 'no error object returned. got=Integer(Integer { value: 5 })', src/evaluator/evaluator_test.rs:210:13
```

需要在Program和块语句中处理求值出错的情况：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_program(node: Node) -> Result<Object, String> {
    let mut result: Object = Object::Null(NULL);
    if let Node::Program(Program { statements }) = node {
        for statement in statements.into_iter() {
            result = eval(Node::Statement(statement))?;
            if let Object::ReturnValue(ReturnValue { value }) = result {
                return Ok(*value);
            }
        }
    }
    Ok(result)
}

fn eval_block_statement(block: BlockStatement) -> Result<Object, String> {
    let mut result: Object = Object::Null(NULL);
    for statement in block.statements.into_iter() {
        result = eval(Node::Statement(statement))?;
        if let Object::ReturnValue(_) = result {
            return Ok(result);
        }
    }
    Ok(result)
}
```

```rust,noplaypen
// src/evaluator/evaluator.rs

pub fn evaluate(program: Program, env: Rc<RefCell<Environment>>) -> Object {
    match eval(Node::Program(program), Rc::clone(&env)) {
        Ok(v) => v,
        Err(err) => Object::ErrorObj(ErrorObj { message: err }),
    }
}
```

测试通过！

错误处理已经完成！