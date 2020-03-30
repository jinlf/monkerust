# 表示对象

我们需要一个对象系统，或称值系统，用来表示AST的值或表示在内存中求值AST时生成的值。
注意：这个对象系统与面向对象编程没有关系。

例如：
```js
let a = 5;
// [...]
a + a;
```
我们首先将整数字面量5绑定到名称a，我们在求值a + a表达式时需要得到这个5。

使用解释语言构建值的内部表示时，有很多不同的选择。
有些使用宿主语言的本机类型（整数，布尔值等）来表示解释语言中的值，而不用任何形式包装。在其他语言中，值/对象仅表示为指针，而在某些编程语言中，本机类型和指针混合在一起。

产生这些不同的原因是：
1. 宿主语言不同；
2. 所解释的语言特性不同；
3. 执行速度和内存消耗需求不同；

另外，还存在如何向解释语言的用户公开这些值及其表示的问题。

## 对象系统基础

由于不考虑性能，我们实现Monkey解释器时选择了最简单的实现方式：定义Rust枚举Object，将每种求值得到的结构体作为Object的一种枚举成员类型。

定义如下：
```rust,noplaypen
// src/object.rs

pub trait ObjectTrait {
    fn get_type(&self) -> String;
    fn inspect(&self) -> String;
}
pub enum Object {}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        String::new() //TODO, be replaced
    }
    fn inspect(&self) -> String {
        String::new() //TODO, be replaced
    }
}
```

再在 lib.rs 中增加

```rust,noplaypen
// src/lib.rs

pub mod object;
```

让我们从最简单的整数、布尔值和空值开始。

## 整数

定义如下：
```rust,noplaypen
// src/object.rs

pub struct Integer {
    pub value: i64,
}
impl ObjectTrait for Integer {
    fn get_type(&self) -> String {
        String::from("INTEGER")
    }
    fn inspect(&self) -> String {
        format!("{}", self.value)
    }
}

pub enum Object {
    Integer(Integer),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Integer(i) => i.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
        }
    }
}
```
当我们在源代码中遇到整型字面量时，先生成对应的AST节点，对其求值得到Integer对象。整数的值被包装在Rust的i64类型变量中。

## 布尔值

定义如下：
```rust,noplaypen
// src/object.rs

pub struct Boolean {
    pub value: bool,
}
impl ObjectTrait for Boolean {
    fn get_type(&self) -> String {
        String::from("BOOLEAN")
    }
    fn inspect(&self) -> String {
        format!("{}", self.value)
    }
}

pub enum Object {
    Integer(Integer),
    Boolean(Boolean),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Integer(i) => i.get_type(),
            Object::Boolean(b) => b.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::Boolean(b) => b.inspect(),
        }
    }
}
```
布尔值的值被包装在Rust的bool类型变量中。

## 空值

编程语言中包含空值会带来很多问题。但出于某些考虑，原作者在设计Monkey语言时仍然保留了空值。

定义如下：
```rust,noplaypen
// src/object.rs

pub struct Null {}
impl ObjectTrait for Null {
    fn get_type(&self) -> String {
        String::from("NULL")
    }
    fn inspect(&self) -> String {
        String::from("null")
    }
}

pub enum Object {
    Integer(Integer),
    Boolean(Boolean),
    Null(Null),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Integer(i) => i.get_type(),
            Object::Boolean(b) => b.get_type(),
            Object::Null(n) => n.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::Boolean(b) => b.inspect(),
            Object::Null(n) => n.inspect(),
        }
    }
}
```

空值是个与布尔值或整数类似的结构体，唯一的不同是没有包装任何值。

有了这个最基本的对象系统，我们可以开始实现eval函数了。
