// src/vm.rs

use super::code::*;
use super::compiler::*;
use super::object::*;

const STACK_SIZE: usize = 2048;

pub struct Vm{
    pub constants: Vec<Object>,
    pub instructions: Instructions,
    pub stack: [Object;STACK_SIZE],
    pub sp: usize, // Always points to the next value. Top of stack is stack[sp-1]
}
impl Vm {
    pub fn new(bytecode: Bytecode) -> Vm {
        Vm {
            instructions: bytecode.instuctions,
            constants: bytecode.constants,

            stack: {
                let data: [std::mem::MaybeUninit<Object>; STACK_SIZE] = unsafe {
                    std::mem::MaybeUninit::uninit().assume_init()
                };            
                unsafe { std::mem::transmute::<_, [Object; STACK_SIZE]>(data) }
            },
            sp: 0,// Always points to the next value. Top of stack is stack[sp-1]
        }
    }

    pub fn stack_top(&self) -> Option<Object> {
        if self.sp == 0 {
            None
        } else {
            Some(self.stack[(self.sp - 1) as usize].clone())
        }
    }

    pub fn run(&mut self) -> Result<String, String> {
        let mut ip =0;  
        while ip < self.instructions.content.len() {
            let op = Opcode::from(self.instructions.content[ip]);
            match op {
                Opcode::OpConstant => {
                    let const_index = read_u16(&self.instructions.content[(ip+1)..(ip+3)]);
                    ip +=2;
                    match self.push(self.constants[const_index as usize].clone()) {
                        Ok(_) => {},
                        Err(err) => return Err(err)
                    }
                }
                _=>{}
            }

            ip+=1;
        }
        Ok(String::new())
    }

    pub fn push(&mut self, o: Object) -> Result<String, String> {
        if self.sp >= STACK_SIZE {
            return Err(String::from("stack overflow"));
        }
        self.stack[self.sp] = o;
        self.sp+=1;
        Ok(String::new())
    }
}

