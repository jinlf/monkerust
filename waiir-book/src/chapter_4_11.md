# 谁在拿垃圾？

考虑当我们在解释器中运行以下Monkey代码片段时会发生什么：
```js
let counter = fn(x) { 
    if (x > 100) {
        return true; 
    } else {
        let foobar = 9999;
        counter(x + 1); 
    }
}; 

counter(0);
```

显然，它将在对counter函数体进行101次求值后返回“true”。但是，这期间发生了很多事。

第一件事是求值if-else表达式的条件：x>100。如果产生的值不为真，则求值if-else表达式的else分支。将整数字面量9999绑定到名称foobar，此名称将不再被引用，接着求值x+1，将调用eval的结果传递到另一个counter调用，然后重新开始，直到 x>100求值结果为真。

重点是：每次调用counter函数都会分配很多对象。用我们的eval函数和对象系统来表述就是：每次对counter函数体的求值都会导致分配和实例化很多object::Integer对象。除了整数字面量9999和x+1的结果，字面量100和1也会产生新的object::Integer对象。

仔细研究你会发现运行此小段代码会导致分配大约400个object::Integers对象。

创建如此多的对象会不会导致内存耗尽呢？并不会！

因为虽然Rust语言并不使用垃圾回收机制，但Rust有确定性析构，变量会在离开作用范围时释放。因此尽管我们Monkey代码段创建了很多对象，但求值的同时也在释放对象。

我们可以通过为object::Integer实现Drop trait来观察这个过程。

增加如下代码：
```rust,noplaypen
// src/object.rs

impl Drop for Integer {
    fn drop(&mut self) {
        println!("dropping Integer {}", self.inspect());
    }
}
```

用cargo run运行前面的Monkey代码（为了简少输出结果，修改条件为x>0）：
```
Hello, This is the Monkey programming language!
Feel free to type in commands
>> let counter = fn(x) { if (x > 0) { return true; } else { let foobar = 9999; counter(x + 1); } };
>> counter(0);
dropping Integer 0
dropping Integer 0
dropping Integer 0
dropping Integer 0
dropping Integer 1
dropping Integer 0
dropping Integer 1
dropping Integer 1
dropping Integer 0
dropping Integer 1
dropping Integer 1
dropping Integer 9999
dropping Integer 9999
dropping Integer 0
true
>>
```
就是说counter函数执行了两次，创建了很多Integer对象，在求值过程中，这些对象被适时释放，并不会耗光内存。看起来，用Rust开发解释器，不需要编写额外的垃圾回收机制。

下一章我们将扩展我们的解释器，让它更有用！