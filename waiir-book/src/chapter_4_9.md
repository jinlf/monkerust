# 绑定与环境

测试用例
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

fn test_error_handling() {
    let tests = [
// [...]
        ("foobar", "identifier not found: foobar"),
    ];
```

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

扩展eval函数，增加Environment类型参数，这里添加用引用计数来实现
```rust,noplaypen
// src/evaluator.rs

use super::environment::*;
use std::cell::*;
use std::rc::*;

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
// [...]
}
```
所有需要env做参数的地方，都使用Rc::clone(&env)。

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