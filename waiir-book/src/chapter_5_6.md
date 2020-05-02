# 大结局

现在Monkey解释器功能比较丰富了。支持数学表达式、变量绑定、函数和调用、条件、return语句，甚至高阶函数和闭包。支持数据类型有：整数、布尔值、字符串、数组和哈希。

下面我们还有一件“非常重要”的事要做。

加入内置函数puts：
```rust,noplaypen
// src/evaluator/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]
        "puts" => {
            let func: BuiltinFunction = |args| {
                for arg in args.iter() {
                    println!("{}", arg.inspect());
                }
                return Ok(Object::Null(NULL));
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}
```

执行cargo run：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> puts("Hello World!")
Hello World!
null
>>
```

好吧，为了打印输出这句“Hello World！”我容易吗！！！