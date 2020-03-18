use super::ast::*;
use super::env::*;
use std::cell::*;
use std::fmt::*;
use std::rc::*;

pub trait ObjectTrait: Debug + Clone {
    fn get_type(&self) -> String;
    fn inspect(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Int { value: i64 },
    Bool { value: bool },
    Null {},
    ReturnValue { value: Box<Option<Object>> },
    Error { message: String },
    Func(Func),
    Str { value: String },
    Builtin(Builtin),
    Array { elements: Vec<Option<Object>> },
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Int { value: _ } => String::from("INTEGER"),
            Object::Bool { value: _ } => String::from("BOOLEAN"),
            Object::Null {} => String::from("NULL"),
            Object::ReturnValue { value: _ } => String::from("RETURN_VALUE"),
            Object::Error { message: _ } => String::from("ERROR"),
            Object::Func(_) => String::from("FUNCTION"),
            Object::Str { value: _ } => String::from("STRING"),
            Object::Builtin(_) => String::from("BUILTIN"),
            Object::Array { elements: _ } => String::from("ARRAY"),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Int { value } => String::from(format!("{}", value)),
            Object::Bool { value } => String::from(format!("{}", value)),
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
            Object::Str { value } => value.clone(),
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
