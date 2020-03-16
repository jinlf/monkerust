use super::ast::*;
use super::lexer::*;
use super::parser::*;
use std::fmt::*;

pub enum ObjectType {
    IntObj,
    BoolObj,
    NullObj,
}

pub trait ObjectTrait: Debug + Copy + Clone {
    fn get_type(&self) -> ObjectType;
    fn inspect(&self) -> String;
}

#[derive(Debug, Copy, Clone)]
pub enum Object {
    Integer { value: i64 },
    Boolean { value: bool },
    Null {},
}
impl ObjectTrait for Object {
    fn get_type(&self) -> ObjectType {
        match self {
            Object::Integer { value: _ } => ObjectType::IntObj,
            Object::Boolean { value: _ } => ObjectType::BoolObj,
            Object::Null {} => ObjectType::NullObj,
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Integer { value } => String::from(format!("{}", value)),
            Object::Boolean { value } => String::from(format!("{}", value)),
            Object::Null {} => String::from("null"),
        }
    }
}
