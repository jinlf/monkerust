use super::ast::*;
use super::lexer::*;
use super::parser::*;
use std::fmt::*;

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    IntObj,
    BoolObj,
    NullObj,
}

pub trait ObjectTrait: Debug + Copy + Clone {
    fn get_type(&self) -> ObjectType;
    fn inspect(&self) -> String;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Object {
    Int { value: i64 },
    Bool { value: bool },
    Null {},
}
impl ObjectTrait for Object {
    fn get_type(&self) -> ObjectType {
        match self {
            Object::Int { value: _ } => ObjectType::IntObj,
            Object::Bool { value: _ } => ObjectType::BoolObj,
            Object::Null {} => ObjectType::NullObj,
        }
    }
    fn inspect(&self) -> String {
        match self {
            Object::Int { value } => String::from(format!("{}", value)),
            Object::Bool { value } => String::from(format!("{}", value)),
            Object::Null {} => String::from("null"),
        }
    }
}
