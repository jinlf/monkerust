# 表达式求值

下面是求值函数的声明。
```rust,noplaypen
fn eval(node: Node) -> Result<Object, String>
```
它输入Node节点，输出Object节点，由于Rust语言不支持空值，考虑到求值的各种情况，这里用Option包装了Object。

## 整数字面量

测试用例：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

use crate::ast::*;
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
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(program) => eval(Node::Program(program), Rc::clone(&env)),
        Err(errors) => panic!("{:?}", errors),
    }
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

#[cfg(test)]
mod evaluator_test;
```

测试失败，因为eval函数没有定义，并且Object对象还不支持打印输出，先解决 Object的输出问题：

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

定义eval函数

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
这里由于需要在eval_statements中对statement执行clone方法，需要把ast.rs中的Statement, Expression相关枚举和结构体定义统一加上Clone属性，保证整个 AST系统都能clone。

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

## 完成REPL

有了eval函数，我们可以将RPPL改成真正的REPL（读入-求值-打印-循环）。

```rust,noplaypen
// src/repl/repl.rs

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

## 布尔值字面量

定义如下：
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
    if let Some(Object::Boolean(Boolean { value })) = obj {
        assert!(
            value == expected,
            "object has wrong value. got={}, want={}",
            value,
            expected
        );
    } else {
        assert!(false, "object is not BooleanLiteral. got={:?}", obj);
    }
}
```

测试结果如下：

```
thread 'evaluator::tests::test_eval_boolean_expression' panicked at 'object is not BooleanLiteral. got=None', src/evaluator_test.rs:83:13
```

加上求值代码：
```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::BooleanLiteral(BooleanLiteral { token: _, value })) => {
            Some(Object::Boolean(Boolean { value: value }))
        }
        _ => None,
    }
}
```

因为布尔值只有两个可能的值，我们可以引用它们而不是每次都创建新值：
```rust,noplaypen
// src/evaluator.rs

pub const TRUE: Boolean = Boolean { value: true };
pub const FALSE: Boolean = Boolean { value: false };

pub fn native_bool_to_boolean_object(input: bool) -> Boolean {
    if input {
        TRUE
    } else {
        FALSE
    }
}

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::BooleanLiteral(BooleanLiteral { token: _, value })) => {
            Some(Object::Boolean(native_bool_to_boolean_object(value)))
        }
// [...]
}
```

## 空值

跟布尔值类似，不需要每次都创建一个新的空值。

```rust,noplaypen
// src/evaluator.rs

pub const NULL: Null = Null {};
```

现在，对象系统中包含了整数、布尔值和空值，可以求值操作符表达式了。

## 前缀表达式

Monkey语言有两个前缀操作符：“!”和“-”。

这里从“!”开始：
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
true和false取反很好理解，Monkey语言中“!5”这种表达式结果会返回false，因为语言规定5是真值。

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
        _ => Some(Object::Null(NULL)),
    }
}
```
如果操作符无效，直接返回NULL值，这种做法比较简单，省去了错误判断。

```rust,noplaypen
// src/evaluator.rs

fn eval_bang_operator_expression(right: Option<Object>) -> Option<Object> {
    match right {
        Some(Object::Boolean(TRUE)) => Some(Object::Boolean(FALSE)),
        Some(Object::Boolean(FALSE)) => Some(Object::Boolean(TRUE)),
        Some(Object::Null(NULL)) => Some(Object::Boolean(TRUE)),
        _ => Some(Object::Boolean(FALSE)),
    }
}
```
由于这里需要比较 Object，需要将Object及其子类型加上 PartialEq 和 Eq 两个属性。

测试通过！


下面再加“-”操作符：
```rust,noplaypen
// src/evaluator_test.rs

fn test_eval_integer_expression() {
    let tests = [("5", 5), ("10", 10), ("-5", -5), ("-10", -10)];

// [...]
}
```

需要在eval_prefix_expression中增加一个分支：
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

首先需要在eval中增加一个分支：
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
不管左子表达式和右子表达式具体什么Node类型，eval都能对其求值。

下面代码表示，当左右子表达式都是整数时根据运算符对其值进行加减乘除操作，否则返回空值。
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

下面考虑比较操作符：
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
对整数左右子表达式进行比较。

```rust,noplaypen
// src/evaluator.rs

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Option<Object> {
    match operator {
// [...]
        "<" => Some(Object::Boolean(native_bool_to_boolean_object(left < right))),
        ">" => Some(Object::Boolean(native_bool_to_boolean_object(left > right))),
        "==" => Some(Object::Boolean(native_bool_to_boolean_object(
            left == right,
        ))),
        "!=" => Some(Object::Boolean(native_bool_to_boolean_object(
            left != right,
        ))),
        _ => Some(Object::Null(NULL)),
    }
}
```
上面实现了对整数的四个比较操作符。

下面添加对布尔值的“==”和“!=”操作，先添加测试用例：
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
thread 'evaluator_test::test_eval_boolean_expression' panicked at 'object is not BooleanLiteral. got=None', src/evaluator_test.rs:94:
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
                "==" => Some(Object::Boolean(native_bool_to_boolean_object(
                    left_obj == right_obj,
                ))),
                "!=" => Some(Object::Boolean(native_bool_to_boolean_object(
                    left_obj != right_obj,
                ))),
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

到现在为止，我们已经实现了一个“计算器”了。