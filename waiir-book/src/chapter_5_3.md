# 内置函数

内置函数不是用户提供的，也不是用Monkey代码编写的，这些函数内置于解释器和语言本身。

我们的内置函数是用Rust编写的。

下面是内置函数的Rust声明。

```rust,noplaypen
// src/object.rs

pub type BuiltinFunction = fn(&Vec<Option<Object>>) -> Option<Object>;
```

我们需要把这种内置函数封装到我们的对象系统中，定义如下：
```rust,noplaypen
// src/object.rs

pub struct Builtin {
    pub func: BuiltinFunction,
}
impl ObjectTrait for Builtin {
    fn get_type(&self) -> String {
        String::from("BUILTIN")
    }
    fn inspect(&self) -> String {
        String::from("builtin function")
    }
}
impl std::fmt::Debug for Builtin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Builtin")
    }
}
impl PartialEq for Builtin {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
impl Eq for Builtin {}
impl Clone for Builtin {
    fn clone(&self) -> Self {
        Builtin { func: self.func }
    }
}

pub enum Object {
// [...]
    Builtin(Builtin),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
// [...]
            Object::Builtin(b) => b.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
// [...]
            Object::Builtin(b) => b.inspect(),
        }
    }
}
```

## len函数

首先我们考虑实现的是len函数，功能如下：
```js
>> len("Hello World!") 12
>> len("") 0
>> len("Hey Bob, how ya doin?") 21
```

先写测试用例：
```rust,noplaypen
// src/evaluator_test.rs

#[test]
fn test_builtin_functions() {
    let tests: [(&str, Object); 5] = [
        (r#"len("")"#, Object::Integer(Integer { value: 0 })),
        (r#"len("four")"#, Object::Integer(Integer { value: 4 })),
        (
            r#"len("hello world")"#,
            Object::Integer(Integer { value: 11 }),
        ),
        (
            r#"len(1)"#,
            Object::ErrorObj(ErrorObj {
                message: String::from("argument to `len` not supported, got INTEGER"),
            }),
        ),
        (
            r#"len("one", "two")"#,
            Object::ErrorObj(ErrorObj {
                message: String::from("wrong number of arguments. got=2, want=1"),
            }),
        ),
    ];

    for tt in tests.iter() {
        let evaluated = test_eval(tt.0);
        if let Object::Integer(Integer { value }) = &tt.1 {
            test_integer_object(evaluated, *value);
        } else if let Object::ErrorObj(ErrorObj { message }) = &tt.1 {
            let expected_message = message;
            if let Some(Object::ErrorObj(ErrorObj { message })) = evaluated {
                assert!(
                    message == *expected_message,
                    "wrong error message. expected={:?}, got={:?}",
                    expected_message,
                    message
                );
            } else {
                assert!(false, "object is not Error. got={:?}", evaluated);
            }
        }
    }
}
```
测试失败输出如下：
```
thread 'evaluator::tests::test_builtin_functions' panicked at 'object is not Integer. got=Some(ErrorObj(ErrorObj { message: "identifier not found: len" }))', src/evaluator_test.rs:432:13
```

为了方便管理，我们单独使用一个文件来编写内置函数的实现：
```rust,noplaypen
// src/buildins.rs

use super::evaluator::*;
use super::object::*;

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => {
            let func: BuiltinFunction = |_| return Some(Object::Null(NULL));
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}
```
目前实现的是一个空的len函数。这里用到了Rust的闭包语法。

将builtins模块加入解释器中：
```rust,noplaypen
// src/lib.rs

pub mod builtins;
```

增加在求值过程中遇到内置函数情况的处理，这里返回内置函数对象：
```rust,noplaypen
// src/evaluator.rs

use super::builtins::*;

fn eval_identifier(node: Identifier, env: Rc<RefCell<Environment>>) -> Option<Object> {
    if let Some(val) = env.borrow().get(node.value.clone()) {
        val
    } else if let Some(builtin) = get_builtin(&node.value) {
        Some(builtin)
    } else {
        new_error(format!("identifier not found: {}", node.value))
    }
}
```
执行cargo run：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> len()
ERROR: not a function: "BUILTIN"
>> 
```

在对函数调用求值时，执行内置函数：
```rust,noplaypen
// src/evaluator.rs

fn apply_function(func: Option<Object>, args: Vec<Option<Object>>) -> Option<Object> {
    if let Some(Object::Function(function)) = func {
        let extended_env = Rc::new(RefCell::new(extend_function_env(function.clone(), args)));
        let evaluated = eval(
            Node::Statement(Statement::BlockStatement(function.body)),
            Rc::clone(&extended_env),
        );
        unwrap_return_value(evaluated)
    } else if let Some(Object::Builtin(Builtin { func })) = func {
        func(&args)
    } else {
        new_error(format!("not a function: {:?}", get_type(&func)))
    }
}
```
测试必然失败，现在的len函数返回为空：
```
thread 'evaluator::tests::test_builtin_functions' panicked at 'object is not Integer. got=Some(Null(Null))', src/evaluator_test.rs:446:13
```

编写len函数，计算字符串的长度，返回整数对象：
```rust,noplaypen
// src/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
        "len" => {
            let func: BuiltinFunction = |args| {
                if args.len() != 1 {
                    return new_error(format!(
                        "wrong number of arguments. got={}, want=1",
                        args.len()
                    ));
                }
                return match &args[0] {
                    Some(Object::StringObj(StringObj { value })) => {
                        Some(Object::Integer(Integer {
                            value: value.len() as i64,
                        }))
                    }
                    _ => new_error(format!(
                        "argument to `len` not supported, got {}",
                        get_type(&args[0])
                    )),
                };
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}
```
测试通过！

用cargo run执行：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> len("1234")
4
>> len("Hello World!")
12
>> len("Woooooohooo!", "len works!!")
ERROR: wrong number of arguments. got=2, want=1
>> len(12345)
ERROR: argument to `len` not supported, got INTEGER
>> 
```

至此，第一个内置函数已经能正常工作了。