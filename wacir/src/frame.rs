// src/frame.rs

use super::code::*;
use super::object::*;

#[derive(Clone)]
pub struct Frame {
    pub func: CompiledFunction,
    pub ip: i64,
    pub base_pointer: usize,
}
impl Frame {
    pub fn new(func: CompiledFunction, base_pointer: usize) -> Frame {
        Frame {
            func: func,
            ip: -1,
            base_pointer: base_pointer,
        }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.func.instructions
    }
}
