// src/frame.rs

use super::code::*;
use super::object::*;

#[derive(Clone)]
pub struct Frame {
    pub func: CompiledFunction,
    pub ip: i64,
}
impl Frame {
    pub fn new(func: CompiledFunction) -> Frame {
        Frame { func: func, ip: -1 }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.func.instructions
    }
}
