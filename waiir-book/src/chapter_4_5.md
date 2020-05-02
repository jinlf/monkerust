# 表达式求值

下面是求值函数的声明。
```rust,noplaypen
fn eval(node: Node) -> Result<Object, String>
```

## 整数字面量

测试用例：

```rust,noplaypen
// src/evaluator/mod.rs

#[cfg(test)]
mod evaluator_test;
```

```rust,noplaypen
// src/evaluator/evaluator_test.rs

use crate::lexer::*;
use crate::object::*;
use crate::parser::*;

#[test]
fn test_eval_integer_expression() {
    let tests = [("5", 5), ("10", 10)];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        test_integer_object(evaluated, tt.1);
    }
}

fn test_eval(input: &str) -> Object {
    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(program) => evaluate(program),
        Err(errors) => panic!("{:?}", errors),
    }
}

fn test_integer_object(obj: Object, expected: i64) {
    if let Object::Integer(Integer { value }) = obj {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        panic!("object is not Integer. got={:?}", obj);
    }
}
```
在main.rs中加入：
```rust,noplaypen
// src/main.rs

mod evaluator;
mod object;
```

测试失败，因为evaluate函数没有定义，并且Object对象还不支持打印输出，先解决 Object的输出问题：

```rust,noplaypen
// src/object/object.rs

#[derive(Debug)]
pub enum Object {
// [...]
}

#[derive(Debug)]
pub struct Integer {
// [...]
}

#[derive(Debug)]
pub struct Boolean {
// [...]
}

#[derive(Debug)]
pub struct Null {} 
```

定义evaluate函数
```rust,noplaypen
// src/evaluator/mod.rs

mod evaluator;
pub use evaluator::*;
```

```rust,noplaypen
// src/evaluator/evaluator.rs

use crate::ast::*;
use crate::object::*;

pub fn evaluate(program: Program) -> Object {
    match eval(Node::Program(program)) {
        Ok(v) => v,
        Err(_) => Object::Null(Null {}),
    }
}

fn eval(node: Node) -> Result<Object, String> {
    match node {
        Node::Program(Program { statements }) => eval_statements(statements),
        Node::Statement(Statement::ExpressionStatement(ExpressionStatement {
            token: _,
            expression,
        })) => eval(Node::Expression(expression)),
        Node::Expression(Expression::IntegerLiteral(IntegerLiteral { token: _, value })) => {
            Ok(Object::Integer(Integer { value: value }))
        }
        _ => Err(String::from("Unknown")),
    }
}

fn eval_statements(stmts: Vec<Statement>) -> Result<Object, String> {
    let mut result: Result<Object, String> = Ok(Object::Null(Null {}));
    for statement in stmts.into_iter() {
        result = eval(Node::Statement(statement));
    }
    result
}
```

在evaluator_test.rs中加入
```rust,noplaypen
// src/evaluator/evaluator_test.rs

use crate::evaluator::*;
```

测试通过！

## 完成REPL

有了eval函数，我们可以将RPPL改成真正的REPL（读入-求值-打印-循环）。

```rust,noplaypen
// src/repl/repl.rs

use crate::evaluator::*;
use crate::object::*;
// [...]

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);

    loop {
// [...]
        match p.parse_program() {
            Ok(program) => writeln!(output, "{}", evaluate(program).inspect()).unwrap(),
            Err(errors) => print_parser_errors(output, &errors),
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

## 布尔值字面量

定义如下：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

#[test]
fn test_eval_boolean_expression() {
    let tests = [("true", true), ("false", false)];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        test_boolean_object(evaluated, tt.1);
    }
}

fn test_boolean_object(obj: Object, expected: bool) {
    if let Object::Boolean(Boolean { value }) = obj {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        panic!("object is not BooleanLiteral. got={:?}", obj);
    }
}
```

测试结果如下：

```
thread 'evaluator::evaluator_test::test_eval_boolean_expression' panicked at 'object is not BooleanLiteral. got=Null(Null)', src/evaluator/evaluator_test.rs:59:9
```

加上求值代码：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node) -> Result<Object, String> {
    match node {
// [...]
        Node::Expression(Expression::BooleanLiteral(BooleanLiteral { token: _, value })) => {
            Ok(Object::Boolean(Boolean { value: value }))
        }
        _ => Err(String::from("Unknown")),
    }
}
```

因为布尔值只有两个可能的值，我们可以引用它们而不是每次都创建新值：
```rust,noplaypen
// src/evaluator/evaluator.rs

pub const TRUE: Boolean = Boolean { value: true };
pub const FALSE: Boolean = Boolean { value: false };

pub fn native_bool_to_boolean_object(input: bool) -> Boolean {
    if input {
        TRUE
    } else {
        FALSE
    }
}

fn eval(node: Node) -> Result<Object, String> {
    match node {
// [...]
        Node::Expression(Expression::BooleanLiteral(BooleanLiteral { token: _, value })) => {
            Ok(Object::Boolean(native_bool_to_boolean_object(value)))
        }
// [...]
}
```

## 空值

跟布尔值类似，不需要每次都创建一个新的空值。

```rust,noplaypen
// src/evaluator/evaluator.rs

pub const NULL: Null = Null {};
```

现有的Null引用都换成NULL，包括，
```rust,noplaypen
// src/evaluator/evaluator.rs

pub fn evaluate(program: Program) -> Object {
    match eval(Node::Program(program)) {
        Ok(v) => v,
        Err(_) => Object::Null(NULL),
    }
}

fn eval_statements(stmts: Vec<Statement>) -> Result<Object, String> {
    let mut result: Result<Object, String> = Ok(Object::Null(NULL));
    for statement in stmts.into_iter() {
        result = eval(Node::Statement(statement));
    }
    result
}
```

现在，对象系统中包含了整数、布尔值和空值，可以求值操作符表达式了。

## 前缀表达式

Monkey语言有两个前缀操作符：“!”和“-”。

这里从“!”开始：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

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
true和false取反很好理解，Monkey语言中“!5”这种表达式结果会返回false，因为语言规定5是真值。

```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node) -> Result<Object, String> {
    match node {
// [...]
        Node::Expression(Expression::PrefixExpression(PrefixExpression {
            token: _,
            operator,
            right,
        })) => {
            let right_obj = eval(Node::Expression(*right))?;
            eval_prefix_expression(&operator, right_obj)
        }
// [...]
}

fn eval_prefix_expression(operator: &str, right: Object) -> Result<Object, String> {
    match operator {
        "!" => eval_bang_operator_expression(right),
        _ => Ok(Object::Null(NULL)),
    }
}

fn eval_bang_operator_expression(right: Object) -> Result<Object, String> {
    match right {
        Object::Boolean(TRUE) => Ok(Object::Boolean(FALSE)),
        Object::Boolean(FALSE) => Ok(Object::Boolean(TRUE)),
        Object::Null(NULL) => Ok(Object::Boolean(TRUE)),
        _ => Ok(Object::Boolean(FALSE)),
    }
}
```
由于这里需要比较 Object，需要将Object及其子类型加上 PartialEq 和 Eq 两个属性。

测试通过！


下面再加“-”操作符：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

fn test_eval_integer_expression() {
    let tests = [("5", 5), ("10", 10), ("-5", -5), ("-10", -10)];

// [...]
}
```

需要在eval_prefix_expression中增加一个分支：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_prefix_expression(operator: &str, right: Object) -> Result<Object, String> {
// [...]
        "-" => eval_minus_prefix_operator_expression(right),
// [...]
}

fn eval_minus_prefix_operator_expression(right: Object) -> Result<Object, String> {
    if let Object::Integer(Integer { value }) = right {
        Ok(Object::Integer(Integer { value: -value }))
    } else {
        Ok(Object::Null(NULL))
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

神奇吧！

## 中缀表达式

Monkey语言支持8种中缀操作符：
```js
5 + 5; 
5 - 5; 
5 * 5; 
5 / 5;

5 > 5; 
5 < 5; 
5 == 5; 
5 != 5;
```

先实现第一组数值操作符，测试用例如下：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

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

首先需要在eval中增加一个分支：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node) -> Result<Object, String> {
// [...]
        Node::Expression(Expression::InfixExpression(InfixExpression {
            token: _,
            left,
            operator,
            right,
        })) => {
            let left_obj = eval(Node::Expression(*left))?;
            let right_obj = eval(Node::Expression(*right))?;
            eval_infix_expression(&operator, left_obj, right_obj)
        }
// [...]
}
```
不管左子表达式和右子表达式具体什么Node类型，eval都能对其求值。

下面代码表示，当左右子表达式都是整数时根据运算符对其值进行加减乘除操作，否则返回空值。
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_infix_expression(
    operator: &str,
    left: Object,
    right: Object,
) -> Result<Object, String> {
    if let Object::Integer(Integer { value }) = left {
        let left_val = value;
        if let Object::Integer(Integer { value }) = right {
            let right_val = value;
            return eval_integer_infix_expression(operator, left_val, right_val);
        }
    }
    Ok(Object::Null(NULL))
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Result<Object, String> {
    match operator {
        "+" => Ok(Object::Integer(Integer {
            value: left + right,
        })),
        "-" => Ok(Object::Integer(Integer {
            value: left - right,
        })),
        "*" => Ok(Object::Integer(Integer {
            value: left * right,
        })),
        "/" => Ok(Object::Integer(Integer {
            value: left / right,
        })),
        _ => Ok(Object::Null(NULL)),
    }
}
```
测试通过！

下面考虑比较操作符：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

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
对整数左右子表达式进行比较。

```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Result<Object, String> {
    match operator {
// [...]
        "<" => Ok(Object::Boolean(native_bool_to_boolean_object(left < right))),
        ">" => Ok(Object::Boolean(native_bool_to_boolean_object(left > right))),
        "==" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left == right,
        ))),
        "!=" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left != right,
        ))),
        _ => Ok(Object::Null(NULL)),
    }
}
```
上面实现了对整数的四个比较操作符。

下面添加对布尔值的“==”和“!=”操作，先添加测试用例：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

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
thread 'evaluator::evaluator_test::test_eval_boolean_expression' panicked at 'object is not BooleanLiteral. got=Null(Null)', src/evaluator/evaluator_test.rs:95:9
```

```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval_infix_expression(operator: &str, left: Object, right: Object) -> Result<Object, String> {
    if let Object::Integer(Integer { value }) = left {
        let left_val = value;
        if let Object::Integer(Integer { value }) = right {
            let right_val = value;
            return eval_integer_infix_expression(operator, left_val, right_val);
        }
    }
    match operator {
        "==" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left == right,
        ))),
        "!=" => Ok(Object::Boolean(native_bool_to_boolean_object(
            left != right,
        ))),
        _ => Ok(Object::Null(NULL)),
    }
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

到现在为止，我们已经实现了一个“计算器”了。