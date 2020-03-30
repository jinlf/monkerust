// src/object.rs

use super::ast::*;
use super::environment::*;
use std::cell::*;
use std::collections::hash_map::*;
use std::fmt::*;
use std::hash::Hash as StdHash;
use std::hash::Hasher;
use std::rc::*;

pub trait ObjectTrait {
    fn get_type(&self) -> String;
    fn inspect(&self) -> String;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Object {
    Integer(Integer),
    BooleanObj(BooleanObj),
    Null(Null),
    ReturnValue(ReturnValue),
    ErrorObj(ErrorObj),
    Function(Function),
    StringObj(StringObj),
    Builtin(Builtin),
    Array(Array),
    Hash(Hash),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Integer(i) => i.get_type(),
            Object::BooleanObj(b) => b.get_type(),
            Object::Null(n) => n.get_type(),
            Object::ReturnValue(rv) => rv.get_type(),
            Object::ErrorObj(e) => e.get_type(),
            Object::Function(f) => f.get_type(),
            Object::StringObj(s) => s.get_type(),
            Object::Builtin(b) => b.get_type(),
            Object::Array(a) => a.get_type(),
            Object::Hash(h) => h.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::BooleanObj(b) => b.inspect(),
            Object::Null(n) => n.inspect(),
            Object::ReturnValue(rv) => rv.inspect(),
            Object::ErrorObj(e) => e.inspect(),
            Object::Function(f) => f.inspect(),
            Object::StringObj(s) => s.inspect(),
            Object::Builtin(b) => b.inspect(),
            Object::Array(a) => a.inspect(),
            Object::Hash(h) => h.inspect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Null {}
impl ObjectTrait for Null {
    fn get_type(&self) -> String {
        String::from("NULL")
    }
    fn inspect(&self) -> String {
        String::from("null")
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReturnValue {
    pub value: Box<Object>,
}
impl ObjectTrait for ReturnValue {
    fn get_type(&self) -> String {
        String::from("RETURN_VALUE")
    }
    fn inspect(&self) -> String {
        self.value.inspect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorObj {
    pub message: String,
}
impl ObjectTrait for ErrorObj {
    fn get_type(&self) -> String {
        String::from("ERROR")
    }
    fn inspect(&self) -> String {
        format!("ERROR: {}", self.message)
    }
}

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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StringObj {
    pub value: String,
}
impl ObjectTrait for StringObj {
    fn get_type(&self) -> String {
        String::from("STRING")
    }
    fn inspect(&self) -> String {
        self.value.clone()
    }
}

pub type BuiltinFunction = fn(&Vec<Option<Object>>) -> Option<Object>;

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
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
impl Eq for Builtin {}
impl Clone for Builtin {
    fn clone(&self) -> Self {
        Builtin { func: self.func }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Array {
    pub elements: Vec<Object>,
}
impl ObjectTrait for Array {
    fn get_type(&self) -> String {
        String::from("ARRAY")
    }
    fn inspect(&self) -> String {
        format!(
            "[{}]",
            self.elements
                .iter()
                .map(|x| x.inspect())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, PartialEq, Eq, StdHash)]
pub struct HashKey {
    pub obj_type: String,
    pub value: u64,
}
pub trait Hashable {
    fn hash_key(&self) -> HashKey;
}
impl Hashable for BooleanObj {
    fn hash_key(&self) -> HashKey {
        let mut value: u64 = 0;
        if self.value {
            value = 1;
        }
        HashKey {
            obj_type: self.get_type(),
            value: value,
        }
    }
}
impl Hashable for Integer {
    fn hash_key(&self) -> HashKey {
        HashKey {
            obj_type: self.get_type(),
            value: self.value as u64,
        }
    }
}
impl Hashable for StringObj {
    fn hash_key(&self) -> HashKey {
        let mut h = DefaultHasher::new();
        self.value.hash(&mut h);
        HashKey {
            obj_type: self.get_type(),
            value: h.finish(),
        }
    }
}

#[derive(Clone)]
pub struct HashPair {
    pub key: Object,
    pub value: Object,
}
#[derive(Clone)]
pub struct Hash {
    pub pairs: HashMap<HashKey, HashPair>,
}
impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Hash")
    }
}
impl Eq for Hash {}
impl PartialEq for Hash {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
impl ObjectTrait for Hash {
    fn get_type(&self) -> String {
        String::from("HASH")
    }
    fn inspect(&self) -> String {
        format!(
            "{{{}}}",
            self.pairs
                .iter()
                .map(|(_, pair)| { format!("{}: {}", pair.key.inspect(), pair.value.inspect()) })
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub trait AsHashable {
    fn as_hashable(&self) -> Option<&dyn Hashable>;
}
impl AsHashable for Object {
    fn as_hashable(&self) -> Option<&dyn Hashable> {
        match self {
            Object::Integer(i) => Some(i),
            Object::BooleanObj(b) => Some(b),
            Object::StringObj(s) => Some(s),
            _ => None,
        }
    }
}
