// src/compiler.rs

use super::ast::*;
use super::code::*;
use super::object::*;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            instructions: Instructions::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile(&self, node: Node) -> Result<String, String> {
        return Ok(String::new());
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instuctions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }
}

pub struct Bytecode {
    pub instuctions: Instructions,
    pub constants: Vec<Object>,
}
