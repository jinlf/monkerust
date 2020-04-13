// src/object.rs

use super::ast::*;
use super::code::*;
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
    Boolean(Boolean),
    Null(Null),
    ReturnValue(ReturnValue),
    ErrorObj(ErrorObj),
    Function(Function),
    StringObj(StringObj),
    Builtin(Builtin),
    Array(Array),
    Hash(Hash),
    CompiledFunction(CompiledFunction),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Integer(i) => i.get_type(),
            Object::Boolean(b) => b.get_type(),
            Object::Null(n) => n.get_type(),
            Object::ReturnValue(rv) => rv.get_type(),
            Object::ErrorObj(e) => e.get_type(),
            Object::Function(f) => f.get_type(),
            Object::StringObj(s) => s.get_type(),
            Object::Builtin(b) => b.get_type(),
            Object::Array(a) => a.get_type(),
            Object::Hash(h) => h.get_type(),
            Object::CompiledFunction(cf) => cf.get_type(),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::Boolean(b) => b.inspect(),
            Object::Null(n) => n.inspect(),
            Object::ReturnValue(rv) => rv.inspect(),
            Object::ErrorObj(e) => e.inspect(),
            Object::Function(f) => f.inspect(),
            Object::StringObj(s) => s.inspect(),
            Object::Builtin(b) => b.inspect(),
            Object::Array(a) => a.inspect(),
            Object::Hash(h) => h.inspect(),
            Object::CompiledFunction(cf) => cf.inspect(),
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
        format!(
            "fn({}) {{\n{}\n}}",
            self.parameters
                .iter()
                .map(|x| x.string())
                .collect::<Vec<String>>()
                .join(", "),
            self.body.string()
        )
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

// impl Drop for Integer {
//     fn drop(&mut self) {
//         println!("dropping Integer {}", self.inspect());
//     }
// }

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

#[derive(PartialEq, Eq, StdHash, Clone)]
pub enum HashKey {
    Integer(Integer),
    Boolean(Boolean),
    StringObj(StringObj),
}
impl HashKey {
    fn inspect(&self) -> String {
        match self {
            HashKey::Integer(i) => i.inspect(),
            HashKey::Boolean(b) => b.inspect(),
            HashKey::StringObj(s) => s.inspect(),
        }
    }
}

impl StdHash for Integer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_type().hash(state);
        state.write_i64(self.value);
        state.finish();
    }
}
impl StdHash for Boolean {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_type().hash(state);
        if self.value {
            state.write_u8(1);
        } else {
            state.write_u8(0);
        }
        state.finish();
    }
}
impl StdHash for StringObj {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_type().hash(state);
        self.value.hash(state);
        state.finish();
    }
}

#[derive(Clone)]
pub struct Hash {
    pub pairs: HashMap<HashKey, Object>,
}
impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Hash")
    }
}
impl Eq for Hash {}
impl PartialEq for Hash {
    fn eq(&self, _other: &Self) -> bool {
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
                .map(|(key, value)| { format!("{}: {}", key.inspect(), value.inspect()) })
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
            Object::Boolean(b) => Some(b),
            Object::StringObj(s) => Some(s),
            _ => None,
        }
    }
}

pub trait Hashable {
    fn hash_key(&self) -> HashKey;
}
impl Hashable for Boolean {
    fn hash_key(&self) -> HashKey {
        HashKey::Boolean(self.clone())
    }
}
impl Hashable for Integer {
    fn hash_key(&self) -> HashKey {
        HashKey::Integer(self.clone())
    }
}
impl Hashable for StringObj {
    fn hash_key(&self) -> HashKey {
        HashKey::StringObj(self.clone())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CompiledFunction {
    pub instructions: Instructions,
}
impl ObjectTrait for CompiledFunction {
    fn get_type(&self) -> String {
        String::from("COMPILED_FUNCTION")
    }
    fn inspect(&self) -> String {
        format!(
            "CompiledFunction[{}]",
            self as *const CompiledFunction as usize
        )
    }
}
