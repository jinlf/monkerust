// src/vm.rs

use super::code::*;
use super::compiler::*;
use super::object::*;

const STACK_SIZE: usize = 2048;

pub struct Vm {
    pub constants: Vec<Object>,
    pub instructions: Instructions,
    pub stack: [Option<Object>; STACK_SIZE],
    pub sp: usize, // Always points to the next value. Top of stack is stack[sp-1]
}
impl Vm {
    pub fn new(bytecode: Bytecode) -> Vm {
        Vm {
            instructions: bytecode.instuctions,
            constants: bytecode.constants,

            stack: {
                let mut data: [std::mem::MaybeUninit<Option<Object>>; STACK_SIZE] =
                    unsafe { std::mem::MaybeUninit::uninit().assume_init() };
                for elem in &mut data[..] {
                    unsafe {
                        std::ptr::write(elem.as_mut_ptr(), None);
                    }
                }
                unsafe { std::mem::transmute::<_, [Option<Object>; STACK_SIZE]>(data) }
            },
            sp: 0, // Always points to the next value. Top of stack is stack[sp-1]
        }
    }

    pub fn stack_top(&self) -> Option<Object> {
        if self.sp == 0 {
            None
        } else {
            self.stack[(self.sp - 1) as usize].clone()
        }
    }

    pub fn run(&mut self) -> Result<String, String> {
        let mut ip = 0;
        while ip < self.instructions.0.len() {
            let op = Opcode::from(self.instructions.0[ip]);
            match op {
                Opcode::OpConstant => {
                    let const_index = read_u16(&self.instructions.0[(ip + 1)..(ip + 3)]);
                    ip += 2;
                    match self.push(self.constants[const_index as usize].clone()) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
                }
                Opcode::OpAdd | Opcode::OpSub | Opcode::OpMul | Opcode::OpDiv => {
                    match self.execute_binary_operation(op) {
                        Ok(_) => {}
                        Err(err) => return Err(err),
                    }
                }
                Opcode::OpPop => {
                    self.pop();
                }
                _ => {}
            }

            ip += 1;
        }
        Ok(String::new())
    }

    pub fn push(&mut self, o: Object) -> Result<String, String> {
        if self.sp >= STACK_SIZE {
            return Err(String::from("stack overflow"));
        }
        self.stack[self.sp] = Some(o);
        self.sp += 1;
        Ok(String::new())
    }

    pub fn pop(&mut self) -> Option<Object> {
        let o = self.stack[self.sp - 1].clone();
        self.sp -= 1;
        o
    }

    pub fn last_popped_stack_elem(&self) -> Option<Object> {
        self.stack[self.sp].clone()
    }

    fn execute_binary_operation(&mut self, op: Opcode) -> Result<String, String> {
        let right = self.pop();
        let left = self.pop();
        if let Some(Object::Integer(Integer { value })) = left {
            let left_value = value;
            if let Some(Object::Integer(Integer { value })) = right {
                let right_value = value;
                return self.execute_binary_integer_operation(op, left_value, right_value);
            } else {
                return Err(String::from("error"));
            }
        } else {
            return Err(String::from("error"));
        }
    }

    fn execute_binary_integer_operation(
        &mut self,
        op: Opcode,
        left_value: i64,
        right_value: i64,
    ) -> Result<String, String> {
        let result = match op {
            Opcode::OpAdd => left_value + right_value,
            Opcode::OpSub => left_value - right_value,
            Opcode::OpMul => left_value * right_value,
            Opcode::OpDiv => left_value / right_value,
            _ => return Err(format!("unknown integer operator: {:?}", op)),
        };
        match self.push(Object::Integer(Integer { value: result })) {
            Ok(_) => {}
            Err(err) => return Err(err),
        }
        Ok(String::new())
    }
}
