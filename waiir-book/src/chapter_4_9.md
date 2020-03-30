# 绑定与环境

需要用绑定来支持let语句，例如：
```js
let x = 5 + 5;
```
我们需要在求值后x的值为10。

测试用例如下：
```rust,noplaypen
// src/evaluator_test.rs

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
// src/evaluator_test.rs

fn test_error_handling() {
    let tests = [
// [...]
        ("foobar", "identifier not found: foobar"),
    ];
```

下面我们求值let语句：
```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node) -> Option<Object> {
    match node {
// [...]
        Node::Statement(Statement::LetStatement(LetStatement {
            token: _,
            name,
            value,
        })) => {
            let val = eval(Node::Expression(value));
            if is_error(&val) {
                return val;
            }

            // 然后呢？
        }
        _ => None,
    }
}
```
我们需要一个标识符名称到值的映射来保存已经求值出来的信息，这种映射称作环境（Environment），可以用一个哈希表（HashMap）即可。

```rust,noplaypen
// src/environment.rs

use super::ast::*;
use super::object::*;
use std::collections::*;

pub fn new_environment() -> Environment {
    Environment {
        store: HashMap::new(),
    }
}

pub struct Environment {
    pub store: HashMap<String, Option<Object>>,
}
impl Environment {
    pub fn get(&self, name: String) -> Option<Option<Object>> {
        if let Some(v) = self.store.get(&name) {
            if let Some(vv) = v {
                return Some(Some(vv.clone()));
            }
        }
        None
    }
    pub fn set(&mut self, name: String, val: Option<Object>) -> Option<Object> {
        if let Some(v) = val {
            self.store.insert(name, Some(v.clone()));
            Some(v)
        } else {
            self.store.insert(name, None);
            None
        }
    }
}

```
由于这里用到了Object的clone方法，需要将Object类型及其子类型都添加Clone属性。

在lib.rs中添加：
```rust,noplaypen
// src/lib.rs

pub mod environment;
```

扩展eval函数，增加Environment类型参数，这里使用引用计数来实现：
```rust,noplaypen
// src/evaluator.rs

use super::environment::*;
use std::cell::*;
use std::rc::*;

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
// [...]
}
```
在Rust中，常用Rc和RefCell组合来实现引用计数，如上述代码中的env参数。

修改REPL，创建（最外层）env：
```rust,noplaypen
// src/repl.rs

use super::environment::*;
use std::cell::*;
use std::rc::*;

pub fn start(input: &mut dyn Read, output: &mut dyn Write) {
    let mut scanner = BufReader::new(input);
    let env = Rc::new(RefCell::new(new_environment()));

    loop {
// [...]        
        if program.is_some() {
            if let Some(evaluated) = eval(Node::Program(program.unwrap()), Rc::clone(&env)) {
                writeln!(output, "{}", evaluated.inspect()).unwrap();
            }
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
// src/evaluator_test.rs

use super::environment::*;
use std::cell::*;
use std::rc::*;

fn test_eval(input: &str) -> Option<Object> {
    let env = Rc::new(RefCell::new(new_environment()));
    let l = Lexer::new(input);
    let mut p = Parser::new(l);
    let program = p.parse_program();
    eval(Node::Program(program.unwrap()), Rc::clone(&env))
}
```

修改求值let语句的代码如下：
```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Statement(Statement::LetStatement(LetStatement {
            token: _,
            name,
            value,
        })) => {
            let val = eval(Node::Expression(value), Rc::clone(&env));
            if is_error(&val) {
                return val;
            }

            env.borrow_mut().set(name.value, val)
        }
        _ => None,
    }
}
```
这里使用borrow_mut方法从引用计数中借用可变对象，然后将标识符和值的映射保存在环境中。

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::Identifier(ident)) => eval_identifier(ident, Rc::clone(&env)),
        _ => None,
    }
}

fn eval_identifier(node: Identifier, env: Rc<RefCell<Environment>>) -> Option<Object> {
    if let Some(val) = env.borrow().get(node.value.clone()) {
        val
    } else {
        new_error(format!("identifier not found: {}", node.value))
    }
}
```
在使用标识符时，使用borrow方法借用env对象，取得标识符对应的值。

测试通过！

用cargo run执行
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