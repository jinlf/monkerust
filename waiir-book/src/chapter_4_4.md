# 表示对象

## 对象系统基础

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

## 整数

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

## 布尔值

```rust,noplaypen
// src/object.rs

pub struct BooleanObj {
    pub value: bool,
}
impl ObjectTrait for BooleanObj {
    fn get_type(&self) -> String {
        String::from("BOOLEAN")
    }
    fn inspect(&self) -> String {
        format!("{}", self.value)
    }
}

pub enum Object {
    Integer(Integer),
    BooleanObj(BooleanObj),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Integer(i) => i.get_type(),
            Object::BooleanObj(b) => b.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::BooleanObj(b) => b.inspect(),
        }
    }
}
```

## 空值

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
    BooleanObj(BooleanObj),
    Null(Null),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Integer(i) => i.get_type(),
            Object::BooleanObj(b) => b.get_type(),
            Object::Null(n) => n.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::BooleanObj(b) => b.inspect(),
            Object::Null(n) => n.inspect(),
        }
    }
}
```
