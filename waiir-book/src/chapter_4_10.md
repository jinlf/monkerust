# 函数与函数调用

假设我们的Monkey语言已经支持函数和函数调用，那REPL将会是这样的：
```
>> let add = fn(a, b, c, d) { return a + b + c + d };
>> add(1, 2, 3, 4); 
10
>> let addThree = fn(x) { return x + 3 };
>> addThree(3); 
6
>> let max = fn(x, y) { if (x > y) { x } else { y } };
>> max(5, 10) 
10
>> let factorial = fn(n) { if (n == 0) { 1 } else { n * factorial(n - 1) } };
>> factorial(5) 
120
```

再如：
```
>> let callTwoTimes = fn(x, func) { func(func(x)) }; 
>> callTwoTimes(3, addThree);
9
>> callTwoTimes(3, fn(x) { x + 1 });
5
>> let newAdder = fn(x) { fn(n) { x + n } }; 
>> let addTwo = newAdder(2);
>> addTwo(2);
4
```

为此，需要做两件事：定义内部对象系统中函数的表示形式，并在eval中增加对函数调用的支持。

函数对象定义如下：
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
函数对象比较特殊，它内部包含了对AST中参数列表和函数体节点的引用。函数对象中的env记录了函数定义处的环境，后续我们会介绍它的用途。

这里定义比较函数对象的方法时，我采用了简单的内存地址比较方法。语句：
```rust,noplaypen
        let addr = self as *const Function as usize;
```
取得的是函数对象的内存地址。

这里还需要为Environment加上Debug属性：
```rust,noplaypen
// src/environment.rs

#[derive(Debug)]
pub struct Environment {
// [...]
```

测试用例如下：
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

在eval中支持函数字面量：
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

函数调用的测试用例：
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

在eval中增加函数调用的支持：
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
对函数调用的求值代码该如何完成呢？

这里需要使用嵌套环境，就是在求值函数体时创建一个新的内部环境，在函数体内取标识符的值时先在内部环境中查找，如果查不到，再到外部环境中查找，以此类推，如果直到找到最外层环境中仍然没有查到，则报错。

修改Environment代码支持嵌套环境：
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

求值函数调用时修改如下：
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
在extend_function_env方法中创建嵌套环境，其内部的outer成员指向函数对象中保存的外部环境。将每个参数的值设置到内部环境中，在apply_function中调用eval求值函数体时就使用内部环境作为实参。

测试通过！

执行cargo run：
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

自此Monkey也能够支持闭包了，测试代码如下：
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

用cargo run执行：
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

闭包是能够将定义处的环境封闭起来的函数。它们携带自己的环境，并且在每次被调用时都可以访问它。
分析上面输入，newAdder是一个高阶函数，addTwo就是一个闭包，因为他不仅能访问自己的参数y，还能访问调用newAdder(2)时绑定的x，即使这个绑定已经超出范围，也不在当前环境中。

测一下：
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
x在我们的最外层环境中没有绑定值。 但是addTwo仍然可以访问它：

```
>> addTwo(3)
5
```

函数也可以作为参数，例如：
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

至此，我们的解释器已经能够支持函数和函数调用了。