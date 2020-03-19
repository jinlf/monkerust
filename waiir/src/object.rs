use super::ast::*;
use super::env::*;
use std::cell::*;
use std::collections::hash_map::*;
use std::fmt::*;
use std::hash::Hash as StdHash;
use std::hash::Hasher;
use std::rc::*;

pub trait ObjectTrait: Debug + Clone {
    fn get_type(&self) -> String;
    fn inspect(&self) -> String;
}

pub enum Hashtable {
    Int(Int),
    Bool(Bool),
    Str(Str),
}
impl Hashtable {
    pub fn hash_key(&self) -> HashKey {
        match self {
            Hashtable::Bool(Bool { value }) => {
                let mut v = 0;
                if *value {
                    v = 1;
                }
                HashKey {
                    obj_type: String::from("BOOLEAN"),
                    value: v,
                }
            }
            Hashtable::Int(Int { value }) => HashKey {
                obj_type: String::from("INTEGER"),
                value: *value as u64,
            },
            Hashtable::Str(Str { value }) => {
                let mut h = DefaultHasher::new();
                value.hash(&mut h);
                HashKey {
                    obj_type: String::from("STRING"),
                    value: h.finish(),
                }
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Int {
    pub value: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bool {
    pub value: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Str {
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Int(Int),
    Bool(Bool),
    Null {},
    ReturnValue { value: Box<Option<Object>> },
    Error { message: String },
    Func(Func),
    Str(Str),
    Builtin(Builtin),
    Array { elements: Vec<Option<Object>> },
    Hash(Hash),
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Int(_) => String::from("INTEGER"),
            Object::Bool(_) => String::from("BOOLEAN"),
            Object::Null {} => String::from("NULL"),
            Object::ReturnValue { value: _ } => String::from("RETURN_VALUE"),
            Object::Error { message: _ } => String::from("ERROR"),
            Object::Func(_) => String::from("FUNCTION"),
            Object::Str(_) => String::from("STRING"),
            Object::Builtin(_) => String::from("BUILTIN"),
            Object::Array { elements: _ } => String::from("ARRAY"),
            Object::Hash(_) => String::from("HASH"),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Int(Int { value }) => String::from(format!("{}", value)),
            Object::Bool(Bool { value }) => String::from(format!("{}", value)),
            Object::Null {} => String::from("null"),
            Object::ReturnValue { value } => value.as_ref().as_ref().unwrap().inspect(),
            Object::Error { message } => String::from(format!("ERROR: {}", message)),
            Object::Func(func) => {
                let mut out = String::new();
                let mut params: Vec<String> = Vec::new();
                for p in func.parameters.iter() {
                    params.push(p.string());
                }
                out.push_str("fn");
                out.push_str("(");
                out.push_str(&params.join(", "));
                out.push_str(") {\n");
                out.push_str(&func.body.string());
                out.push_str("\n}");
                out
            }
            Object::Str(Str { value }) => value.clone(),
            Object::Builtin(_) => String::from("builtin function"),
            Object::Array { elements } => {
                let mut out = String::new();
                let mut elems: Vec<String> = Vec::new();
                for e in elements.iter() {
                    elems.push(e.as_ref().unwrap().inspect());
                }

                out.push_str("[");
                out.push_str(&elems.join(", "));
                out.push_str("]");

                out
            }
            Object::Hash(Hash { pairs }) => {
                let mut out = String::new();
                let mut new_pairs: Vec<String> = Vec::new();
                for (_, pair) in pairs.iter() {
                    new_pairs.push(format!("{}: {}", pair.key.inspect(), pair.value.inspect()));
                }

                out.push_str("{");
                out.push_str(&new_pairs.join(", "));
                out.push_str("}");

                out
            }
        }
    }
}
impl Object {
    pub fn hash_key(&self) -> Option<HashKey> {
        match self {
            Object::Bool(Bool { value }) => {
                Some(Hashtable::Bool(Bool { value: *value }).hash_key())
            }
            Object::Int(Int { value }) => Some(Hashtable::Int(Int { value: *value }).hash_key()),
            Object::Str(Str { value }) => Some(
                Hashtable::Str(Str {
                    value: value.clone(),
                })
                .hash_key(),
            ),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct Func {
    pub parameters: Vec<Ident>,
    pub body: BlockStmt,
    pub env: Option<Rc<RefCell<Env>>>,
}
impl std::fmt::Debug for Func {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?} {:?}", self.parameters, self.body)
    }
}
impl PartialEq for Func {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl Eq for Func {}

pub struct Builtin {
    pub func: fn(args: &Vec<Option<Object>>) -> Option<Object>,
}
impl Debug for Builtin {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Builtin")
    }
}
impl Clone for Builtin {
    fn clone(&self) -> Self {
        Builtin { func: self.func }
    }
}
impl PartialEq for Builtin {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}
impl Eq for Builtin {}

#[derive(Debug, PartialEq, Eq, StdHash, Clone)]
pub struct HashKey {
    pub obj_type: String,
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HashPair {
    pub key: Object,
    pub value: Object,
}
impl HashPair {
    fn string(&self) -> String {
        let mut out = String::new();
        out.push_str("(");
        out.push_str(&self.key.inspect());
        out.push_str(", ");
        out.push_str(&self.value.inspect());
        out.push_str(")");
        out
    }
}

#[derive(PartialEq, Eq)]
pub struct Hash {
    pub pairs: HashMap<HashKey, HashPair>,
}
impl Debug for Hash {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut list: Vec<String> = Vec::new();
        for (_, value) in self.pairs.iter() {
            list.push(value.string());
        }
        write!(f, "Hash ({})", list.join(", "))
    }
}
impl StdHash for Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self as *const Hash as usize);
        state.finish();
    }
}
impl Clone for Hash {
    fn clone(&self) -> Self {
        let mut pairs: HashMap<HashKey, HashPair> = HashMap::new();
        pairs.clone_from(&self.pairs);
        Hash { pairs: pairs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_hash_key() {
        let hello1 = Hashtable::Str(Str {
            value: String::from("Hello World"),
        });
        let hello2 = Hashtable::Str(Str {
            value: String::from("Hello World"),
        });
        let diff1 = Hashtable::Str(Str {
            value: String::from("My name is johnny"),
        });
        let diff2 = Hashtable::Str(Str {
            value: String::from("My name is johnny"),
        });

        assert!(
            hello1.hash_key() == hello2.hash_key(),
            "strings with same content have different hash keys"
        );
        assert!(
            diff1.hash_key() == diff2.hash_key(),
            "strings with same content have different hash keys"
        );
        assert!(
            hello1.hash_key() != diff1.hash_key(),
            "strings with different content have same hash keys"
        );
    }
}
