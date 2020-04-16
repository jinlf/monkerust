// src/frame.rs

use super::code::*;
use super::object::*;

#[derive(Clone)]
pub struct Frame {
    pub cl: Closure,
    pub ip: i64,
    pub base_pointer: usize,
}
impl Frame {
    pub fn new(cl: Closure, base_pointer: usize) -> Frame {
        Frame {
            cl: cl,
            ip: -1,
            base_pointer: base_pointer,
        }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.cl.func.instructions
    }
}
