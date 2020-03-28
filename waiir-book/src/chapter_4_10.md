# 函数与函数调用

```rust,noplaypen
// src/object.rs

use super::ast::*;
use super::environment::*;
use std::cell::*;
use std::rc::*;

#[derive(Debug, Clone)]
pub struct Function {
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Rc<RefCell<Environment>>,
}
impl ObjectTrait for Function {
    fn get_type(&self) -> String {
        String::from("FUNCTION")
    }
    fn inspect(&self) -> String {
        let mut params: Vec<String> = Vec::new();
        for p in self.parameters.iter() {
            params.push(p.string());
        }

        format!("fn({}) {{\n{}\n}}", params.join(", "), self.body.string())
    }
}
impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        let addr = self as *const Function as usize;
        let other_addr = other as *const Function as usize;
        addr == other_addr
    }
}
impl Eq for Function {}

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
// [...]
    Function(Function),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
// [...]
            Object::Function(f) => f.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
// [...]
            Object::Function(f) => f.inspect(),
        }
    }
}
```
其中需要Environment支持Debug属性。

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_function_object() {
    let input = "fn(x) { x + 2; };";
    let evaluated = test_eval(input);
    if let Some(Object::Function(Function {
        parameters,
        body,
        env: _,
    })) = evaluated
    {
        assert!(
            parameters.len() == 1,
            "function has wrong parameters. got={}",
            parameters.len()
        );

        assert!(
            parameters[0].string() == "x",
            "parameter is not 'x'. got={:?}",
            parameters[0]
        );

        let expected_body = "(x + 2)";
        assert!(
            body.string() == expected_body,
            "body is not {}, got={}",
            expected_body,
            body.string()
        );
    } else {
        assert!(false, "object is not Function. got={:?}", evaluated);
    }
}
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::FunctionLiteral(FunctionLiteral {
            token: _,
            parameters,
            body,
        })) => Some(Object::Function(Function {
            parameters: parameters,
            body: body,
            env: Rc::clone(&env),
        })),
        _ => None,
    }
}
```

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_function_application() {
    let tests = [
        ("let identity = fn(x) { x; }; identity(5);", 5),
        ("let identity = fn(x) { return x; }; identity(5);", 5),
        ("let double = fn(x) { x * 2; }; double(5);", 10),
        ("let add = fn(x, y) { x + y; }; add(5, 5);", 10),
        ("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));", 20),
        ("fn(x) { x; }(5)", 5),
    ];

    for tt in tests.iter() {
        test_integer_object(test_eval(tt.0), tt.1);
    }
}
```

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::CallExpression(CallExpression {
            token: _,
            function,
            arguments,
        })) => {
            let function_obj = eval(Node::Expression(*function), Rc::clone(&env));
            if is_error(&function_obj) {
                return function_obj;
            }
            let args = eval_expressions(arguments, Rc::clone(&env));
            if args.len() == 1 && is_error(&args[0]) {
                return args[0].clone();
            }
            // 然后呢？
        }
        _ => None,
    }
}

fn eval_expressions(exps: Vec<Expression>, env: Rc<RefCell<Environment>>) -> Vec<Option<Object>> {
    let mut result: Vec<Option<Object>> = Vec::new();
    for e in exps.iter() {
        let evaluated = eval(Node::Expression(e.clone()), Rc::clone(&env));
        if is_error(&evaluated) {
            return vec![evaluated];
        }
        result.push(evaluated);
    }
    result
}
```

```rust,noplaypen
// src/environment.rs

use std::cell::*;
use std::rc::*;

pub fn new_enclosed_environment(outer: Option<Rc<RefCell<Environment>>>) -> Environment {
    let mut env = new_environment();
    env.outer = outer;
    env
}

pub fn new_environment() -> Environment {
    Environment {
        store: HashMap::new(),
        outer: None,
    }
}

#[derive(Debug)]
pub struct Environment {
    pub store: HashMap<String, Option<Object>>,
    pub outer: Option<Rc<RefCell<Environment>>>,
}
impl Environment {
    pub fn get(&self, name: String) -> Option<Option<Object>> {
        if let Some(v) = self.store.get(&name) {
            if let Some(vv) = v {
                return Some(Some(vv.clone()));
            }
        } else if let Some(o) = &self.outer {
            return o.borrow().get(name);
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

```rust,noplaypen
// src/evaluator.rs

pub fn eval(node: Node, env: Rc<RefCell<Environment>>) -> Option<Object> {
    match node {
// [...]
        Node::Expression(Expression::CallExpression(CallExpression {
            token: _,
            function,
            arguments,
        })) => {
            let function_obj = eval(Node::Expression(*function), Rc::clone(&env));
            if is_error(&function_obj) {
                return function_obj;
            }
            let args = eval_expressions(arguments, Rc::clone(&env));
            if args.len() == 1 && is_error(&args[0]) {
                return args[0].clone();
            }
            apply_function(function_obj, args)
        }
        _ => None,
    }
}

fn apply_function(func: Option<Object>, args: Vec<Option<Object>>) -> Option<Object> {
    if let Some(Object::Function(function)) = func {
        let extended_env = Rc::new(RefCell::new(extend_function_env(function.clone(), args)));
        let evaluated = eval(
            Node::Statement(Statement::BlockStatement(function.body)),
            Rc::clone(&extended_env),
        );
        unwrap_return_value(evaluated)
    } else {
        new_error(format!("not a function: {:?}", get_type(&func)))
    }
}

fn extend_function_env(func: Function, args: Vec<Option<Object>>) -> Environment {
    let mut env = new_enclosed_environment(Some(func.env));
    for (param_idx, param) in func.parameters.iter().enumerate() {
        env.set(param.value.clone(), args[param_idx].clone());
    }
    env
}

fn unwrap_return_value(obj: Option<Object>) -> Option<Object> {
    if let Some(Object::ReturnValue(ReturnValue { value })) = obj {
        return Some(*value);
    }
    obj
}
```
测试通过！

执行cargo run
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let addTwo = fn(x) { x + 2; };
fn(x) {
(x + 2)
}
>> addTwo(2)
4
>> let multiply = fn(x, y) { x * y }; 
fn(x, y) {
(x * y)
}
>> multiply(50 / 2, 1 * 2)
50
>> fn(x) { x == 10 }(5)
false
>> fn(x) { x == 10 }(10)             
true
>>
```

```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_closures() {
    let input = "
let newAdder = fn(x) {
fn(y) { x + y };
};
let addTwo = newAdder(2);
addTwo(2);";
    test_integer_object(test_eval(input), 4);
}
```
测试通过！

用cargo run执行
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let newAdder = fn(x) { fn(y) { x + y } };
fn(x) {
fn (y) (x + y)
}
>> let addTwo = newAdder(2);
fn(y) {
(x + y)
}
>> addTwo(3);
5
>> let addThree = newAdder(3);
fn(y) {
(x + y)
}
>> addThree(10);
13
>> 
```

再次
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let newAdder = fn(x) {fn(y) { x + y } };
fn(x) {
fn (y) (x + y)
}
>> let addTwo = newAdder(2);
fn(y) {
(x + y)
}
>> x
ERROR: identifier not found: x
>> 
```

```
>> addTwo(3)
5
```

```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let add = fn(a, b) { a + b };
fn(a, b) {
(a + b)
}
>> let sub = fn(a, b) { a - b };
fn(a, b) {
(a - b)
}
>> let applyFunc = fn(a, b, func) { func(a, b) };
fn(a, b, func) {
func(a, b)
}
>> applyFunc(2, 2, add);
4
>> applyFunc(10, 2, sub); 
8
>> 
```
