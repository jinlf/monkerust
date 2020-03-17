use std::fmt::*;

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    IntObj,
    BoolObj,
    NullObj,
    ReturnObj,
    ErrorObj,
}

pub trait ObjectTrait: Debug + Clone {
    fn get_type(&self) -> String;
    fn inspect(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Int { value: i64 },
    Bool { value: bool },
    Null {},
    ReturnValue { value: Box<Object> },
    Error { message: String },
}
impl ObjectTrait for Object {
    fn get_type(&self) -> String {
        match self {
            Object::Int { value: _ } => String::from("INTEGER"),
            Object::Bool { value: _ } => String::from("BOOLEAN"),
            Object::Null {} => String::from("NULL"),
            Object::ReturnValue { value: _ } => String::from("RETURN_VALUE"),
            Object::Error { message: _ } => String::from("ERROR"),
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Int { value } => String::from(format!("{}", value)),
            Object::Bool { value } => String::from(format!("{}", value)),
            Object::Null {} => String::from("null"),
            Object::ReturnValue { value } => value.inspect(),
            Object::Error { message } => String::from(format!("ERROR: {}", message)),
        }
    }
}
