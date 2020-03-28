# 大结局

```rust,noplaypen
// src/builtins.rs

pub fn get_builtin(name: &str) -> Option<Object> {
    match name {
// [...]
        "puts" => {
            let func: BuiltinFunction = |args| {
                for arg in args.iter() {
                    println!("{}", arg.as_ref().unwrap().inspect());
                }
                return Some(Object::Null(NULL));
            };
            Some(Object::Builtin(Builtin { func: func }))
        }
        _ => None,
    }
}
```

用cargo run执行
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> puts("Hello World!")
Hello World!
null
>>
```