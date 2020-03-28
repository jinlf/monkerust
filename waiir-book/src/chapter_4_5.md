# 表达式求值

## 整数字面量

测试用例

```rust,noplaypen
// src/evaluator_test.rs

use super::ast::*;
use super::lexer::*;
use super::object::*;
use super::parser::*;

#[test]
fn test_eval_integer_expression() {
    let tests = [("5", 5), ("10", 10)];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        test_integer_object(evaluated, tt.1);
    }
}

fn test_eval(input: &str) -> Option<Object> {
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    eval(Node::Program(program.unwrap()))
}

fn test_integer_object(obj: Option<Object>, expected: i64) {
    if let Some(Object::Integer(Integer { value })) = obj {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        assert!(false, "object is not Integer. got={:?}", obj);
    }
}
```

为了测试还需要在 lib.rs 中增加：

```rust,noplaypen
// src/lib.rs

mod evaluator_test;

```

测试失败，因为 eval 函数没有定义，并且 Object 对象还不支持输出，先解决 Object 的输出问题：

```rust,noplaypen
// src/object.rs

#[derive(Debug)]
pub enum Object {
// [...]
}

#[derive(Debug)]
pub struct Integer {
// [...]
}

#[derive(Debug)]
pub struct BooleanObj {
// [...]
}

#[derive(Debug)]
pub struct Null {}
```

加上 eval 函数

```rust,noplaypen
// src/evaluator.rs

use super::ast::*;
use super::object::*;

pub fn eval(node: Node) -> Option<Object> {
    match node {
        Node::Program(Program { statements }) => eval_statements(statements),
        Node::Statement(Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        })) => eval(Node::Expression(expression)),
        Node::Expression(Expression::IntegerLiteral(IntegerLiteral { token: _, value })) => {
            Some(Object::Integer(Integer { value: value }))
        }
        _ => None,
    }
}

fn eval_statements(stmts: Vec<Statement>) -> Option<Object> {
    let mut result: Option<Object> = None;
    for statement in stmts.iter() {
        result = eval(Node::Statement(statement.clone()));
    }
    result
}
```

这里由于需要在 eval_statements 中对 statement 执行 clone，需要把 ast.rs 中 Statement, Expression 相关枚举和结构体定义统一加上 Clone 属性，保证整个 AST 系统都能 Clone。

在lib.rs中加入
```rust,noplaypen
// src/lib.rs

pub mod evaluator;
```

在evaluator_test.rs中加入
```rust,noplaypen
// src/evaluator_test.rs

use super::evaluator::*;
```

测试通过！

## 补充 REPL

```rust,noplaypen
// src/repl.rs

use super::evaluator::*;
use super::object::*;
// [...]

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);

    loop {
// [...]
        if p.errors.len() != 0 {
            print_parser_errors(output, &p.errors);
            continue;
        }
        if program.is_some() {
            if let Some(evaluated) = eval(Node::Program(program.unwrap())) {
                writeln!(output, "{}", evaluated.inspect()).unwrap();
            }
        }
    }
}
```

执行 cargo run，如下：

```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> 5
5
>> 10
10
>> 999
999
>>
```

## 布尔字面量

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_eval_boolean_expression() {
    let tests = [("true", true), ("false", false)];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        test_boolean_object(evaluated, tt.1);
    }
}

fn test_boolean_object(obj: Option<Object>, expected: bool) {
    if let Some(Object::BooleanObj(BooleanObj { value })) = obj {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        assert!(false, "object is not Boolean. got={:?}", obj);
    }
}
```

测试结果

```
thread 'evaluator::tests::test_eval_boolean_Expression' panicked at 'object is not Boolean. got=None', src/evaluator_test.rs:83:13
```

加上求值代码

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::Boolean(Boolean { token: _, value })) => {
            Some(Object::BooleanObj(BooleanObj { value: value }))
        }
        _ => None,
    }
}
```

但是考虑到。。。

```rust,noplaypen
// src/evaluator.rs

pub const TRUE: BooleanObj = BooleanObj { value: true };
pub const FALSE: BooleanObj = BooleanObj { value: false };

pub fn native_bool_to_boolean_object(input: bool) -> Object {
    if input {
        TRUE
    } else {
        FALSE
    }
}

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::Boolean(Boolean { token: _, value })) => {
            Some(Object::BooleanObj(native_bool_to_boolean_object(value)))
        }
// [...]
}
```

## 空值

```rust,noplaypen
// src/evaluator.rs

pub const NULL: Null = Null {};
```

## 前缀表达式

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_bang_operator() {
    let tests = [
        ("!true", false),
        ("!false", true),
        ("!5", false),
        ("!!true", true),
        ("!!false", false),
        ("!!5", true),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        test_boolean_object(evaluated, tt.1);
    }
}
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::PrefixExpression(PrefixExpression {
            token: _,
            operator,
            right,
        })) => {
            let right_obj = eval(Node::Expression(*right));
            eval_prefix_expression(&operator, right_obj)
        }
// [...]
}

fn eval_prefix_expression(operator: &str, right: Option<Object>) -> Option<Object> {
    match operator {
        "!" => eval_bang_operator_expression(right),
        _ => None,
    }
}

fn eval_bang_operator_expression(right: Option<Object>) -> Option<Object> {
    match right {
        Some(Object::BooleanObj(TRUE)) => Some(Object::BooleanObj(FALSE)),
        Some(Object::BooleanObj(FALSE)) => Some(Object::BooleanObj(TRUE)),
        Some(Object::Null(NULL)) => Some(Object::BooleanObj(TRUE)),
        _ => Some(Object::BooleanObj(FALSE)),
    }
}
```

由于这里需要比较 Object，需要将Object及其子类型加上 PartialEq 和 Eq 两个属性。

测试通过！

下面再加

```rust,noplaypen
// src/evaluator_test.rs

fn test_eval_integer_expression() {
    let tests = [("5", 5), ("10", 10), ("-5", -5), ("-10", -10)];

// [...]
}
```

加上

```rust,noplaypen
// src/evaluator.rs

fn eval_prefix_expression(operator: &str, right: Option<Object>) -> Option<Object> {
// [...]
        "-" => eval_minus_prefix_operator_expression(right),
// [...]
}

fn eval_minus_prefix_operator_expression(right: Option<Object>) -> Option<Object> {
    if let Some(Object::Integer(Integer { value })) = right {
        Some(Object::Integer(Integer { value: -value }))
    } else {
        Some(Object::Null(NULL))
    }
}
```

测试通过！

用 cargo run 执行

```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> -5
-5
>> !true
false
>> !-5
false
>> !!-5
true
>> !!!!-5
true
>> -true
null
>>
```

## 中缀表达式

```rust,noplaypen
// src/evaluator_test.rs

fn test_eval_integer_expression() {
    let tests = [
        ("5", 5),
        ("10", 10),
        ("-5", -5),
        ("-10", -10),
        ("5 + 5 + 5 + 5 - 10", 10),
        ("2 * 2 * 2 * 2 * 2", 32),
        ("-50 + 100 + -50", 0),
        ("5 * 2 + 10", 20),
        ("5 + 2 * 10", 25),
        ("20 + 2 * -10", 0),
        ("50 / 2 * 2 + 10", 60),
        ("2 * (5 + 10)", 30),
        ("3 * 3 * 3 + 10", 37),
        ("3 * (3 * 3) + 10", 37),
        ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
    ];
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
// [...]
        Node::Expression(Expression::InfixExpression(InfixExpression {
            token: _,
            left,
            operator,
            right,
        })) => {
            let left_obj = eval(Node::Expression(*left));
            let right_obj = eval(Node::Expression(*right));
            eval_infix_expression(&operator, left_obj, right_obj)
        }
// [...]
}
```

```rust,noplaypen
// src/evaluator.rs

fn eval_infix_expression(
    operator: &str,
    left: Option<Object>,
    right: Option<Object>,
) -> Option<Object> {
    if let Some(Object::Integer(Integer { value })) = left {
        let left_val = value;
        if let Some(Object::Integer(Integer { value })) = right {
            let right_val = value;
            return eval_integer_infix_expression(operator, left_val, right_val);
        }
    }
    None
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Option<Object> {
    match operator {
        "+" => Some(Object::Integer(Integer {
            value: left + right,
        })),
        "-" => Some(Object::Integer(Integer {
            value: left - right,
        })),
        "*" => Some(Object::Integer(Integer {
            value: left * right,
        })),
        "/" => Some(Object::Integer(Integer {
            value: left / right,
        })),
        _ => Some(Object::Null(NULL)),
    }
}
```

测试通过！

```rust,noplaypen
// src/evaluator_test.rs


fn test_eval_boolean_expression() {
    let tests = [
        ("true", true),
        ("false", false),
        ("1 < 2", true),
        ("1 > 2", false),
        ("1 < 1", false),
        ("1 > 1", false),
        ("1 == 1", true),
        ("1 != 1", false),
        ("1 == 2", false),
        ("1 != 2", true),
    ];
// [...]
```

```rust,noplaypen
// src/evaluator.rs

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Option<Object> {
    match operator {
// [...]
        "<" => Some(Object::BooleanObj(native_bool_to_boolean_object(left < right))),
        ">" => Some(Object::BooleanObj(native_bool_to_boolean_object(left > right))),
        "==" => Some(Object::BooleanObj(native_bool_to_boolean_object(left == right))),
        "!=" => Some(Object::BooleanObj(native_bool_to_boolean_object(left != right))),
        _ => Some(Object::Null(NULL)),
    }
}
```

```rust,noplaypen
// src/evaluator_test.rs

fn test_eval_boolean_expression() {
// [...]
        ("true == true", true),
        ("false == false", true),
        ("true == false", false),
        ("true != false", true),
        ("false != true", true),
        ("(1 < 2) == true", true),
        ("(1 < 2) == false", false),
        ("(1 > 2) == true", false),
        ("(1 > 2) == false", true),
    ];
// [...]
```

测试结果如下：

```
thread 'evaluator_test::test_eval_boolean_expression' panicked at 'object is not Boolean. got=None', src/evaluator_test.rs:94:
```

```rust,noplaypen
// src/evaluator.rs

fn eval_infix_expression(
    operator: &str,
    left: Option<Object>,
    right: Option<Object>,
) -> Option<Object> {
    if let Some(Object::Integer(Integer { value })) = left {
        let left_val = value;
        if let Some(Object::Integer(Integer { value })) = right {
            let right_val = value;
            return eval_integer_infix_expression(operator, left_val, right_val);
        }
    }
    if let Some(left_obj) = left {
        if let Some(right_obj) = right {
            return match operator {
                "==" => Some(Object::BooleanObj(native_bool_to_boolean_object(left_obj == right_obj))),
                "!=" => Some(Object::BooleanObj(native_bool_to_boolean_object(left_obj != right_obj))),
                _ => Some(Object::Null(NULL)),
            };
        }
    }
    Some(Object::Null(NULL))
}
```

测试通过！

用 cargo run 执行

```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> 5 * 5 + 10
35
>> 3 + 4 * 5 == 3 * 1 + 4 * 5
true
>> 5 * 10 > 40 + 5
true
>> (10 + 2) * 30 == 300 + 20 * 3
true
>> (5 > 5 == true) != false
false
>> 500 / 2 != 250
false
>>
```
