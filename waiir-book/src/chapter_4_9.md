# 绑定与环境

需要用绑定来支持let语句，例如：
```js
let x = 5 + 5;
```
我们需要在求值后x的值为10。

测试用例如下：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

#[test]
fn test_let_statements() {
    let tests = [
        ("let a = 5; a;", 5),
        ("let a = 5 * 5; a;", 25),
        ("let a = 5; let b = a; b;", 5),
        ("let a = 5; let b = a; let c = a + b + 5; c;", 15),
    ];

    for tt in tests.iter() {
        test_integer_object(test_eval(tt.0), tt.1);
    }
}
```

还需要测试标识符没有值的情况；
```rust,noplaypen
// src/evaluator/evaluator_test.rs

fn test_error_handling() {
    let tests = [
// [...]
        ("foobar", "identifier not found: foobar"),
    ];
```

下面我们求值let语句：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node) -> Result<Object, String> {
    match node {
// [...]
        Node::Statement(Statement::LetStatement(LetStatement {
            token: _,
            name,
            value,
        })) => {
            let val = eval(Node::Expression(value))?;

            // 然后呢？
        }
        _ => Err(String::from("Unknown")),
    }
}
```
我们需要一个标识符名称到值的映射来保存已经求值出来的信息，这种映射称作环境（Environment），可以用一个哈希表（HashMap）来实现。

```rust,noplaypen
// src/environment/mod.rs

mod environment;
pub use environment::*;
```

```rust,noplaypen
// src/environment.rs

use crate::ast::*;
use crate::object::*;
use std::collections::*;

pub fn new_environment() -> Environment {
    Environment {
        store: HashMap::new(),
    }
}

pub struct Environment {
    pub store: HashMap<String, Object>,
}
impl Environment {
    pub fn get(&self, name: &str) -> Option<Object> {
        if let Some(v) = self.store.get(name) {
            return Some(v.clone());
        }
        None
    }
    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val.clone());
        val
    }
}

```
由于这里用到了Object的clone方法，需要将Object类型及其子类型都添加Clone属性。

修改main.rs：
```rust,noplaypen
// src/main.rs

mod environment;
```

扩展eval函数，增加Environment类型参数，这里使用引用计数来实现：
```rust,noplaypen
// src/evaluator/evaluator.rs

use crate::environment::*;
use std::cell::*;
use std::rc::*;

pub fn evaluate(program: Program, env: Rc<RefCell<Environment>>) -> Object {
    match eval(Node::Program(program), Rc::clone(&env)) {
        Ok(v) => v,
        Err(err) => Object::Null(Null {}),
    }
}

fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
// [...]
}
```
在Rust中，常用Rc和RefCell组合来实现引用计数，如上述代码中的env参数。

修改REPL，创建（最外层）env：
```rust,noplaypen
// src/repl/repl.rs

use crate::environment::*;
use std::cell::*;
use std::rc::*;

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);
    let env = Rc::new(RefCell::new(new_environment()));

    loop {
// [...]    
        match p.parse_program() {
            Ok(program) => {
                writeln!(output, "{}", evaluate(program, Rc::clone(&env)).inspect()).unwrap(),
            }
            Err(errors) => print_parser_errors(output, &errors),
        }
    }
}
```
这里需要修改的不仅仅是eval和start函数，很多求值函数都需要传递env做参数，请在命令行中执行：
```
$ cargo build
```
命令，根据编译器的报错修改相关代码，增加必要的env参数，所有env做实参的地方，都使用Rc::clone(&env)来增加对env的引用计数。重复上述步骤，直至没有此类错误。

之前的测试用例中也需要增加创建和使用env的代码：
```rust,noplaypen
// src/evaluator/evaluator_test.rs

use crate::environment::*;
use std::cell::*;
use std::rc::*;

fn test_eval(input: &str) -> Object {
    let env = Rc::new(RefCell::new(new_environment()));

    let l = Lexer::new(String::from(input));
    let mut p = Parser::new(l);
    match p.parse_program() {
        Ok(program) => evaluate(program, Rc::clone(&env)),
        Err(errors) => panic!("{:?}", errors),
    }
}
```

修改求值let语句的代码如下：
```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Object {
    match node {
// [...]
        Node::Statement(Statement::LetStatement(LetStatement {
            token: _,
            name,
            value,
        })) => {
            let val = eval(Node::Expression(value), Rc::clone(&env))?;
            Ok(env.borrow_mut().set(name.value, val))
        }
        _ => Err(String::from("Unknown")),
    }
}
```
这里使用borrow_mut方法从引用计数中借用可变对象，然后将标识符和值的映射保存在环境中。

```rust,noplaypen
// src/evaluator/evaluator.rs

fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Object {
    match node {
// [...]
        Node::Expression(Expression::Identifier(ident)) => eval_identifier(ident, Rc::clone(&env)),
        _ => Err(String::from("Unknown")),
    }
}

fn eval_identifier(node: Identifier, env: Rc<RefCell<Environment>>) -> Result<Object, String> {
    if let Some(val) = env.borrow().get(&node.value) {
        Ok(val)
    } else {
        Ok(new_error(format!("identifier not found: {}", node.value)))
    }
}
```
在使用标识符时，使用borrow方法借用env不可变对象，取得标识符对应的值。

测试通过！

执行cargo run：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let a = 5;
5
>> let b = a > 3;
true
>> let c = a * 99;
495
>> if (b) { 10 } else { 1 };
10
>> let d = if (c > a) { 99 } else { 100 };
99
>> d * c * a;
245025
>> 
```